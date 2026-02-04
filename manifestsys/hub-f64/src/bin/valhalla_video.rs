//! 🌌 Valhalla 视频编解码器
//!
//! 用 Valhalla 的方式压缩和播放视频
//! 编码: 视频 → Valhalla 内部表示
//! 解码: Valhalla 表示 → 可播放视频

use std::fs::{self, File};
use std::io::{self, Write, Read, BufWriter, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

/// Valhalla 视频帧 - 压缩表示
#[derive(Clone)]
struct ValhallaFrame {
    /// 帧宽度
    width: u16,
    /// 帧高度
    height: u16,
    /// 压缩后的像素数据 (每个 8x8 块压缩为一个向量)
    blocks: Vec<CompressedBlock>,
}

/// 压缩块 (8x8 像素 → 压缩表示)
#[derive(Clone, Copy)]
struct CompressedBlock {
    /// 块的平均颜色 (YUV)
    y_avg: u8,
    u_avg: u8,
    v_avg: u8,
    /// 块的纹理特征 (Valhalla direction)
    texture: [i8; 4],  // 4 字节纹理向量
    /// 能量/复杂度
    energy: u8,
}

// 每个块 = 3 + 4 + 1 = 8 字节
// 原始 8x8 RGB = 192 字节
// 压缩率: 96%

/// 帧间差异块 (P帧用)
#[derive(Clone, Copy)]
struct DeltaBlock {
    /// 是否与前一帧相同 (skip)
    skip: bool,
    /// 差异值 (如果不 skip)
    delta_y: i8,
    delta_u: i8,
    delta_v: i8,
}

impl DeltaBlock {
    fn is_similar(a: &CompressedBlock, b: &CompressedBlock, threshold: u8) -> bool {
        let dy = (a.y_avg as i16 - b.y_avg as i16).abs();
        let du = (a.u_avg as i16 - b.u_avg as i16).abs();
        let dv = (a.v_avg as i16 - b.v_avg as i16).abs();
        dy < threshold as i16 && du < threshold as i16 && dv < threshold as i16
    }
}

impl CompressedBlock {
    /// 内部迭代压缩 - 迭代 iterations 次优化表示
    fn from_pixels_iterative(pixels: &[u8], stride: usize, x: usize, y: usize, iterations: usize) -> Self {
        // 第一次: 提取原始特征
        let mut block = Self::from_pixels_single(pixels, stride, x, y);
        
        if iterations <= 1 {
            return block;
        }
        
        // 提取原始像素用于比较
        let mut original = [[0u8; 3]; 64];
        for dy in 0..8 {
            for dx in 0..8 {
                let idx = ((y + dy) * stride + (x + dx)) * 3;
                if idx + 2 < pixels.len() {
                    original[dy * 8 + dx] = [pixels[idx], pixels[idx + 1], pixels[idx + 2]];
                }
            }
        }
        
        // 迭代优化
        for iter in 1..iterations {
            // 解码当前块
            let decoded = block.to_pixels();
            
            // 计算误差并调整
            let mut error_y = 0i32;
            let mut error_u = 0i32;
            let mut error_v = 0i32;
            let mut h_error = 0i32;
            let mut v_error = 0i32;
            
            for i in 0..64 {
                let orig_r = original[i][0] as i32;
                let orig_g = original[i][1] as i32;
                let orig_b = original[i][2] as i32;
                let dec_r = decoded[i * 3] as i32;
                let dec_g = decoded[i * 3 + 1] as i32;
                let dec_b = decoded[i * 3 + 2] as i32;
                
                // 计算 YUV 误差
                let orig_y = ((66 * orig_r + 129 * orig_g + 25 * orig_b + 128) >> 8) + 16;
                let dec_y = ((66 * dec_r + 129 * dec_g + 25 * dec_b + 128) >> 8) + 16;
                error_y += orig_y - dec_y;
                
                let orig_u = ((-38 * orig_r - 74 * orig_g + 112 * orig_b + 128) >> 8) + 128;
                let dec_u = ((-38 * dec_r - 74 * dec_g + 112 * dec_b + 128) >> 8) + 128;
                error_u += orig_u - dec_u;
                
                let orig_v = ((112 * orig_r - 94 * orig_g - 18 * orig_b + 128) >> 8) + 128;
                let dec_v = ((112 * dec_r - 94 * dec_g - 18 * dec_b + 128) >> 8) + 128;
                error_v += orig_v - dec_v;
                
                // 纹理误差
                let dx = i % 8;
                let dy = i / 8;
                if dx > 0 {
                    h_error += (orig_r - original[i-1][0] as i32).abs() - 
                              (dec_r - decoded[(i-1) * 3] as i32).abs();
                }
                if dy > 0 {
                    v_error += (orig_r - original[i-8][0] as i32).abs() - 
                              (dec_r - decoded[(i-8) * 3] as i32).abs();
                }
            }
            
            // 学习率随迭代递减
            let lr = 1.0 / (iter as f64 + 1.0);
            
            // 调整参数
            let y_adj = ((error_y as f64 / 64.0) * lr) as i32;
            let u_adj = ((error_u as f64 / 64.0) * lr) as i32;
            let v_adj = ((error_v as f64 / 64.0) * lr) as i32;
            
            block.y_avg = (block.y_avg as i32 + y_adj).max(0).min(255) as u8;
            block.u_avg = (block.u_avg as i32 + u_adj).max(0).min(255) as u8;
            block.v_avg = (block.v_avg as i32 + v_adj).max(0).min(255) as u8;
            
            // 调整纹理
            let h_adj = ((h_error as f64 / 56.0) * lr) as i32;
            let v_adj = ((v_error as f64 / 56.0) * lr) as i32;
            
            block.texture[0] = (block.texture[0] as i32 + h_adj / 4).max(-127).min(127) as i8;
            block.texture[1] = (block.texture[1] as i32 + v_adj / 4).max(-127).min(127) as i8;
        }
        
        block
    }
    
    fn from_pixels(pixels: &[u8], stride: usize, x: usize, y: usize) -> Self {
        // 默认 10 次迭代
        Self::from_pixels_iterative(pixels, stride, x, y, 10)
    }
    
    fn from_pixels_single(pixels: &[u8], stride: usize, x: usize, y: usize) -> Self {
        let mut y_sum = 0u32;
        let mut u_sum = 0u32;
        let mut v_sum = 0u32;
        let mut h_diff = 0i32;
        let mut v_diff = 0i32;
        
        // 遍历 8x8 块
        for dy in 0..8 {
            for dx in 0..8 {
                let idx = ((y + dy) * stride + (x + dx)) * 3;
                if idx + 2 < pixels.len() {
                    let r = pixels[idx] as u32;
                    let g = pixels[idx + 1] as u32;
                    let b = pixels[idx + 2] as u32;
                    
                    // RGB → YUV
                    let y_val = ((66 * r + 129 * g + 25 * b + 128) >> 8) + 16;
                    let u_val = ((-38 * r as i32 - 74 * g as i32 + 112 * b as i32 + 128) >> 8) + 128;
                    let v_val = ((112 * r as i32 - 94 * g as i32 - 18 * b as i32 + 128) >> 8) + 128;
                    
                    y_sum += y_val.min(255);
                    u_sum += u_val.max(0).min(255) as u32;
                    v_sum += v_val.max(0).min(255) as u32;
                    
                    // 计算梯度
                    if dx > 0 {
                        let prev_idx = ((y + dy) * stride + (x + dx - 1)) * 3;
                        if prev_idx + 2 < pixels.len() {
                            h_diff += (pixels[idx] as i32 - pixels[prev_idx] as i32).abs();
                        }
                    }
                    if dy > 0 {
                        let prev_idx = ((y + dy - 1) * stride + (x + dx)) * 3;
                        if prev_idx + 2 < pixels.len() {
                            v_diff += (pixels[idx] as i32 - pixels[prev_idx] as i32).abs();
                        }
                    }
                }
            }
        }
        
        let y_avg = (y_sum / 64).min(255) as u8;
        let u_avg = (u_sum / 64).min(255) as u8;
        let v_avg = (v_sum / 64).min(255) as u8;
        
        // 纹理向量 (方向特征)
        let texture = [
            ((h_diff / 64).min(127) as i8),  // 水平纹理
            ((v_diff / 64).min(127) as i8),  // 垂直纹理
            (((h_diff - v_diff) / 128).max(-128).min(127) as i8),  // 对角特征
            ((y_avg as i32 - 128) / 2).max(-128).min(127) as i8,   // 亮度偏移
        ];
        
        // 能量 = 纹理总强度
        let energy = ((h_diff + v_diff) / 128).min(255) as u8;
        
        Self {
            y_avg,
            u_avg,
            v_avg,
            texture,
            energy,
        }
    }
    
    fn to_pixels(&self) -> [u8; 192] {  // 8x8 * 3 = 192
        let mut pixels = [0u8; 192];
        
        // YUV → RGB
        let y = self.y_avg as i32 - 16;
        let u = self.u_avg as i32 - 128;
        let v = self.v_avg as i32 - 128;
        
        let r = ((298 * y + 409 * v + 128) >> 8).max(0).min(255) as u8;
        let g = ((298 * y - 100 * u - 208 * v + 128) >> 8).max(0).min(255) as u8;
        let b = ((298 * y + 516 * u + 128) >> 8).max(0).min(255) as u8;
        
        // 使用纹理向量添加细节
        for dy in 0..8 {
            for dx in 0..8 {
                let idx = (dy * 8 + dx) * 3;
                
                // 基于纹理向量添加变化
                let h_var = (self.texture[0] as i32 * (dx as i32 - 4)) / 32;
                let v_var = (self.texture[1] as i32 * (dy as i32 - 4)) / 32;
                let d_var = (self.texture[2] as i32 * ((dx + dy) as i32 - 7)) / 64;
                
                let variation = h_var + v_var + d_var;
                
                pixels[idx] = (r as i32 + variation).max(0).min(255) as u8;
                pixels[idx + 1] = (g as i32 + variation).max(0).min(255) as u8;
                pixels[idx + 2] = (b as i32 + variation).max(0).min(255) as u8;
            }
        }
        
        pixels
    }
    
    fn to_bytes(&self) -> [u8; 8] {
        [
            self.y_avg,
            self.u_avg,
            self.v_avg,
            self.texture[0] as u8,
            self.texture[1] as u8,
            self.texture[2] as u8,
            self.texture[3] as u8,
            self.energy,
        ]
    }
    
    fn from_bytes(bytes: &[u8; 8]) -> Self {
        Self {
            y_avg: bytes[0],
            u_avg: bytes[1],
            v_avg: bytes[2],
            texture: [
                bytes[3] as i8,
                bytes[4] as i8,
                bytes[5] as i8,
                bytes[6] as i8,
            ],
            energy: bytes[7],
        }
    }
}

/// Valhalla 视频文件格式
struct ValhallaVideo {
    magic: [u8; 8],      // "VALHALLA"
    version: u8,         // 1
    width: u16,
    height: u16,
    fps: u8,
    frame_count: u32,
    // 然后是帧数据...
}

/// 迭代学习后的紧凑表示
#[derive(Clone)]
struct LearnedBlock {
    /// 学习后的紧凑编码 (4字节而非8字节)
    code: u32,
}

impl LearnedBlock {
    /// 从迭代优化后的块创建紧凑表示
    fn from_block(block: &CompressedBlock) -> Self {
        // 量化到更小的范围
        let y_q = (block.y_avg / 4) as u32;      // 6 bits (0-63)
        let u_q = (block.u_avg / 8) as u32;      // 5 bits (0-31)
        let v_q = (block.v_avg / 8) as u32;      // 5 bits (0-31)
        let t0 = ((block.texture[0] as i32 + 128) / 16) as u32;  // 4 bits
        let t1 = ((block.texture[1] as i32 + 128) / 16) as u32;  // 4 bits
        let e_q = (block.energy / 32) as u32;   // 3 bits
        
        // 打包: 6 + 5 + 5 + 4 + 4 + 3 = 27 bits → 4 bytes
        let code = (y_q << 21) | (u_q << 16) | (v_q << 11) | (t0 << 7) | (t1 << 3) | e_q;
        
        Self { code }
    }
    
    fn to_block(&self) -> CompressedBlock {
        let y_q = ((self.code >> 21) & 0x3F) as u8;
        let u_q = ((self.code >> 16) & 0x1F) as u8;
        let v_q = ((self.code >> 11) & 0x1F) as u8;
        let t0 = ((self.code >> 7) & 0xF) as u8;
        let t1 = ((self.code >> 3) & 0xF) as u8;
        let e_q = (self.code & 0x7) as u8;
        
        CompressedBlock {
            y_avg: y_q * 4 + 2,
            u_avg: u_q * 8 + 4,
            v_avg: v_q * 8 + 4,
            texture: [
                (t0 as i32 * 16 - 128 + 8) as i8,
                (t1 as i32 * 16 - 128 + 8) as i8,
                0,
                0,
            ],
            energy: e_q * 32 + 16,
        }
    }
    
    fn to_bytes(&self) -> [u8; 4] {
        self.code.to_le_bytes()
    }
    
    fn from_bytes(bytes: &[u8; 4]) -> Self {
        Self { code: u32::from_le_bytes(*bytes) }
    }
}

/// V3 编码 - 迭代学习 + 紧凑表示
fn encode_video_v3(input_path: &str, output_path: &str, iterations: usize) -> io::Result<()> {
    println!("📥 输入: {}", input_path);
    println!("📦 输出: {} (V3 迭代学习)", output_path);
    println!("🔄 内部迭代: {} 次", iterations);
    
    let probe = Command::new("ffprobe")
        .args(&["-v", "error", "-select_streams", "v:0",
                "-show_entries", "stream=width,height,r_frame_rate",
                "-of", "csv=p=0", input_path])
        .output()?;
    
    let info = String::from_utf8_lossy(&probe.stdout);
    let parts: Vec<&str> = info.trim().split(',').collect();
    
    let width: u16 = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(1920);
    let height: u16 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1080);
    let fps: u8 = parts.get(2).map(|s| {
        if s.contains('/') {
            let p: Vec<&str> = s.split('/').collect();
            (p[0].parse::<f32>().unwrap_or(24.0) / p[1].parse::<f32>().unwrap_or(1.0)).round() as u8
        } else { s.parse().unwrap_or(24) }
    }).unwrap_or(24);
    
    println!("📐 视频: {}x{} @ {} fps", width, height, fps);
    
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    
    // V3 头部
    writer.write_all(b"VALHALV3")?;
    writer.write_all(&[3])?;
    writer.write_all(&width.to_le_bytes())?;
    writer.write_all(&height.to_le_bytes())?;
    writer.write_all(&[fps])?;
    writer.write_all(&(iterations as u8).to_le_bytes())?;
    writer.write_all(&[0u8; 4])?;  // frame_count
    
    let blocks_w = (width as usize + 7) / 8;
    let blocks_h = (height as usize + 7) / 8;
    let frame_size = width as usize * height as usize * 3;
    
    println!("🔄 开始 V3 编码...");
    
    let mut ffmpeg = Command::new("ffmpeg")
        .args(&["-i", input_path, "-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    
    let mut stdout = ffmpeg.stdout.take().unwrap();
    let mut frame_buffer = vec![0u8; frame_size];
    let mut prev_blocks: Vec<LearnedBlock> = Vec::new();
    let mut frame_count = 0u32;
    let mut total_skipped = 0u64;
    let mut total_blocks = 0u64;
    
    loop {
        match stdout.read_exact(&mut frame_buffer) {
            Ok(_) => {},
            Err(_) => break,
        }
        
        let is_i_frame = frame_count % 30 == 0;
        writer.write_all(&[if is_i_frame { b'I' } else { b'P' }])?;
        
        let mut current_blocks: Vec<LearnedBlock> = Vec::with_capacity(blocks_w * blocks_h);
        
        for by in 0..blocks_h {
            for bx in 0..blocks_w {
                // 迭代优化
                let block = CompressedBlock::from_pixels_iterative(
                    &frame_buffer, width as usize, bx * 8, by * 8, iterations
                );
                current_blocks.push(LearnedBlock::from_block(&block));
            }
        }
        
        if is_i_frame || prev_blocks.is_empty() {
            for block in &current_blocks {
                writer.write_all(&block.to_bytes())?;  // 4 bytes vs 8 bytes
            }
        } else {
            // P 帧: RLE 压缩
            let mut skip_count = 0u16;
            for (i, block) in current_blocks.iter().enumerate() {
                let similar = (block.code ^ prev_blocks[i].code) < 256;  // 相似度阈值
                
                if similar {
                    skip_count += 1;
                    total_skipped += 1;
                } else {
                    if skip_count > 0 {
                        writer.write_all(&[0xFF, 0xFF])?;
                        writer.write_all(&skip_count.to_le_bytes())?;
                        skip_count = 0;
                    }
                    writer.write_all(&block.to_bytes())?;
                }
                total_blocks += 1;
            }
            if skip_count > 0 {
                writer.write_all(&[0xFF, 0xFF])?;
                writer.write_all(&skip_count.to_le_bytes())?;
            }
            writer.write_all(&[0xFE, 0xFE])?;  // 帧结束
        }
        
        prev_blocks = current_blocks;
        frame_count += 1;
        
        if frame_count % 50 == 0 {
            print!("\r   {} 帧, skip: {:.1}%", frame_count, 
                   total_skipped as f64 / total_blocks.max(1) as f64 * 100.0);
            io::stdout().flush()?;
        }
    }
    
    writer.flush()?;
    drop(writer);
    
    let mut file = fs::OpenOptions::new().write(true).open(output_path)?;
    use std::io::Seek;
    file.seek(std::io::SeekFrom::Start(15))?;
    file.write_all(&frame_count.to_le_bytes())?;
    
    let out_size = fs::metadata(output_path)?.len();
    let in_size = fs::metadata(input_path)?.len();
    
    println!("\n✅ V3 编码完成!");
    println!("   帧数: {}", frame_count);
    println!("   原始: {:.2} MB → 压缩: {:.2} MB", in_size as f64/1e6, out_size as f64/1e6);
    if out_size < in_size {
        println!("   压缩率: {:.1}%", (1.0 - out_size as f64/in_size as f64) * 100.0);
    } else {
        println!("   膨胀: {:.1}x", out_size as f64/in_size as f64);
    }
    
    Ok(())
}

/// V3 解码
fn decode_video_v3(input_path: &str, output_path: &str) -> io::Result<()> {
    println!("📥 输入: {}", input_path);
    
    let mut reader = BufReader::new(File::open(input_path)?);
    
    let mut magic = [0u8; 8];
    reader.read_exact(&mut magic)?;
    if &magic != b"VALHALV3" {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "不是 V3 格式"));
    }
    
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;  // version
    
    let mut w = [0u8; 2];
    let mut h = [0u8; 2];
    reader.read_exact(&mut w)?;
    reader.read_exact(&mut h)?;
    let width = u16::from_le_bytes(w) as usize;
    let height = u16::from_le_bytes(h) as usize;
    
    reader.read_exact(&mut buf)?;
    let fps = buf[0];
    reader.read_exact(&mut buf)?;  // iterations (ignored for decode)
    
    let mut fc = [0u8; 4];
    reader.read_exact(&mut fc)?;
    let frame_count = u32::from_le_bytes(fc);
    
    let blocks_w = (width + 7) / 8;
    let blocks_h = (height + 7) / 8;
    let blocks_per_frame = blocks_w * blocks_h;
    
    println!("📐 {}x{} @ {} fps, {} 帧", width, height, fps, frame_count);
    println!("🔄 解码中...");
    
    let out_w = blocks_w * 8;
    let out_h = blocks_h * 8;
    
    let mut ffmpeg = Command::new("ffmpeg")
        .args(&["-y", "-f", "rawvideo", "-pix_fmt", "rgb24",
                "-s", &format!("{}x{}", out_w, out_h),
                "-r", &fps.to_string(), "-i", "-",
                "-c:v", "libx264", "-preset", "fast", "-crf", "23",
                output_path])
        .stdin(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    
    let mut stdin = ffmpeg.stdin.take().unwrap();
    let frame_size = out_w * out_h * 3;
    let mut frame = vec![0u8; frame_size];
    let mut prev_blocks: Vec<LearnedBlock> = vec![LearnedBlock { code: 0 }; blocks_per_frame];
    
    for f in 0..frame_count {
        let mut frame_type = [0u8; 1];
        reader.read_exact(&mut frame_type)?;
        
        let mut current_blocks = prev_blocks.clone();
        
        if frame_type[0] == b'I' {
            for i in 0..blocks_per_frame {
                let mut bytes = [0u8; 4];
                reader.read_exact(&mut bytes)?;
                current_blocks[i] = LearnedBlock::from_bytes(&bytes);
            }
        } else {
            // P 帧: 读取直到帧结束标记
            let mut i = 0;
            loop {
                let mut marker = [0u8; 2];
                reader.read_exact(&mut marker)?;
                
                if marker[0] == 0xFE && marker[1] == 0xFE {
                    break;  // 帧结束
                } else if marker[0] == 0xFF && marker[1] == 0xFF {
                    // skip marker - 读取数量
                    let mut skip_bytes = [0u8; 2];
                    reader.read_exact(&mut skip_bytes)?;
                    let skip = u16::from_le_bytes(skip_bytes) as usize;
                    i += skip;
                } else {
                    // 普通块: 读取剩余2字节
                    let mut rest = [0u8; 2];
                    reader.read_exact(&mut rest)?;
                    let bytes = [marker[0], marker[1], rest[0], rest[1]];
                    if i < blocks_per_frame {
                        current_blocks[i] = LearnedBlock::from_bytes(&bytes);
                        i += 1;
                    }
                }
            }
        }
        
        // 渲染帧
        for by in 0..blocks_h {
            for bx in 0..blocks_w {
                let block = current_blocks[by * blocks_w + bx].to_block();
                let pixels = block.to_pixels();
                
                for dy in 0..8 {
                    for dx in 0..8 {
                        let src = (dy * 8 + dx) * 3;
                        let dst = ((by * 8 + dy) * out_w + (bx * 8 + dx)) * 3;
                        if dst + 2 < frame.len() {
                            frame[dst] = pixels[src];
                            frame[dst + 1] = pixels[src + 1];
                            frame[dst + 2] = pixels[src + 2];
                        }
                    }
                }
            }
        }
        
        stdin.write_all(&frame)?;
        prev_blocks = current_blocks;
        
        if (f + 1) % 100 == 0 {
            print!("\r   {}/{} 帧", f + 1, frame_count);
            io::stdout().flush()?;
        }
    }
    
    drop(stdin);
    ffmpeg.wait()?;
    println!("\n✅ 解码完成: {}", output_path);
    
    Ok(())
}

fn encode_video_v2(input_path: &str, output_path: &str) -> io::Result<()> {
    println!("📥 输入: {}", input_path);
    println!("📦 输出: {} (V2 帧间压缩)", output_path);
    
    // 获取视频信息
    let probe = Command::new("ffprobe")
        .args(&["-v", "error", "-select_streams", "v:0",
                "-show_entries", "stream=width,height,r_frame_rate",
                "-of", "csv=p=0", input_path])
        .output()?;
    
    let info = String::from_utf8_lossy(&probe.stdout);
    let parts: Vec<&str> = info.trim().split(',').collect();
    
    if parts.len() < 3 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "无法读取视频信息"));
    }
    
    let width: u16 = parts[0].parse().unwrap_or(1920);
    let height: u16 = parts[1].parse().unwrap_or(1080);
    let fps_str = parts[2];
    let fps: u8 = if fps_str.contains('/') {
        let fps_parts: Vec<&str> = fps_str.split('/').collect();
        let num: f32 = fps_parts[0].parse().unwrap_or(24.0);
        let den: f32 = fps_parts[1].parse().unwrap_or(1.0);
        (num / den).round() as u8
    } else {
        fps_str.parse().unwrap_or(24)
    };
    
    println!("📐 视频: {}x{} @ {} fps", width, height, fps);
    
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    
    // 写入头部 V2
    writer.write_all(b"VALHALV2")?;  // V2 magic
    writer.write_all(&[2])?;          // version 2
    writer.write_all(&width.to_le_bytes())?;
    writer.write_all(&height.to_le_bytes())?;
    writer.write_all(&[fps])?;
    writer.write_all(&[0u8; 4])?;  // frame_count placeholder
    
    let blocks_per_row = (width as usize + 7) / 8;
    let blocks_per_col = (height as usize + 7) / 8;
    let blocks_per_frame = blocks_per_row * blocks_per_col;
    let frame_size = width as usize * height as usize * 3;
    
    println!("🔄 开始 V2 编码 (帧间压缩)...");
    
    let mut ffmpeg = Command::new("ffmpeg")
        .args(&["-i", input_path, "-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    
    let mut stdout = ffmpeg.stdout.take().unwrap();
    let mut frame_buffer = vec![0u8; frame_size];
    let mut prev_blocks: Vec<CompressedBlock> = Vec::new();
    let mut frame_count = 0u32;
    let mut total_skipped = 0u64;
    let mut total_blocks = 0u64;
    let gop_size = 30;  // 每 30 帧一个 I 帧
    let threshold = 8u8;  // 相似度阈值
    
    loop {
        match stdout.read_exact(&mut frame_buffer) {
            Ok(_) => {},
            Err(_) => break,
        }
        
        let is_i_frame = frame_count % gop_size == 0;
        
        // 写入帧类型标记
        writer.write_all(&[if is_i_frame { b'I' } else { b'P' }])?;
        
        let mut current_blocks: Vec<CompressedBlock> = Vec::with_capacity(blocks_per_frame);
        
        for by in 0..blocks_per_col {
            for bx in 0..blocks_per_row {
                let block = CompressedBlock::from_pixels(&frame_buffer, width as usize, bx * 8, by * 8);
                current_blocks.push(block);
            }
        }
        
        if is_i_frame || prev_blocks.is_empty() {
            // I 帧: 写入完整数据
            for block in &current_blocks {
                writer.write_all(&block.to_bytes())?;
            }
        } else {
            // P 帧: 使用 RLE 压缩 skip 块
            let mut skip_count = 0u16;
            
            for (i, block) in current_blocks.iter().enumerate() {
                let prev = &prev_blocks[i];
                
                if DeltaBlock::is_similar(block, prev, threshold) {
                    skip_count += 1;
                    total_skipped += 1;
                } else {
                    // 写入之前的 skip count
                    if skip_count > 0 {
                        writer.write_all(&[0xFF])?;  // skip 标记
                        writer.write_all(&skip_count.to_le_bytes())?;
                        skip_count = 0;
                    }
                    // 写入变化的块
                    writer.write_all(&block.to_bytes())?;
                }
                total_blocks += 1;
            }
            
            // 处理末尾的 skip
            if skip_count > 0 {
                writer.write_all(&[0xFF])?;
                writer.write_all(&skip_count.to_le_bytes())?;
            }
            writer.write_all(&[0xFE])?;  // 帧结束标记
        }
        
        prev_blocks = current_blocks;
        frame_count += 1;
        
        if frame_count % 100 == 0 {
            let skip_ratio = total_skipped as f64 / total_blocks as f64 * 100.0;
            print!("\r   已编码 {} 帧, skip率: {:.1}%...", frame_count, skip_ratio);
            io::stdout().flush()?;
        }
    }
    
    writer.flush()?;
    drop(writer);
    
    // 更新 frame_count
    let mut file = fs::OpenOptions::new().write(true).open(output_path)?;
    use std::io::Seek;
    file.seek(std::io::SeekFrom::Start(14))?;
    file.write_all(&frame_count.to_le_bytes())?;
    
    let output_size = fs::metadata(output_path)?.len();
    let input_size = fs::metadata(input_path)?.len();
    let skip_ratio = total_skipped as f64 / total_blocks as f64 * 100.0;
    
    println!("\n✅ V2 编码完成!");
    println!("   帧数: {} (I帧: {}, P帧: {})", frame_count, frame_count / gop_size + 1, frame_count - frame_count / gop_size - 1);
    println!("   跳过块: {:.1}%", skip_ratio);
    println!("   原始: {:.2} MB", input_size as f64 / 1024.0 / 1024.0);
    println!("   压缩: {:.2} MB", output_size as f64 / 1024.0 / 1024.0);
    
    if output_size < input_size {
        println!("   压缩率: {:.1}%", (1.0 - output_size as f64 / input_size as f64) * 100.0);
    } else {
        println!("   膨胀: {:.1}x", output_size as f64 / input_size as f64);
    }
    
    Ok(())
}

fn encode_video(input_path: &str, output_path: &str) -> io::Result<()> {
    println!("📥 输入: {}", input_path);
    println!("📦 输出: {}", output_path);
    
    // 获取视频信息
    let probe = Command::new("ffprobe")
        .args(&["-v", "error", "-select_streams", "v:0",
                "-show_entries", "stream=width,height,r_frame_rate,nb_frames",
                "-of", "csv=p=0", input_path])
        .output()?;
    
    let info = String::from_utf8_lossy(&probe.stdout);
    let parts: Vec<&str> = info.trim().split(',').collect();
    
    if parts.len() < 3 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "无法读取视频信息"));
    }
    
    let width: u16 = parts[0].parse().unwrap_or(1920);
    let height: u16 = parts[1].parse().unwrap_or(1080);
    let fps_str = parts[2];
    let fps: u8 = if fps_str.contains('/') {
        let fps_parts: Vec<&str> = fps_str.split('/').collect();
        let num: f32 = fps_parts[0].parse().unwrap_or(24.0);
        let den: f32 = fps_parts[1].parse().unwrap_or(1.0);
        (num / den).round() as u8
    } else {
        fps_str.parse().unwrap_or(24)
    };
    
    println!("📐 视频: {}x{} @ {} fps", width, height, fps);
    
    // 创建输出文件
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    
    // 写入头部
    writer.write_all(b"VALHALLA")?;  // 8 bytes magic
    writer.write_all(&[1])?;          // version
    writer.write_all(&width.to_le_bytes())?;
    writer.write_all(&height.to_le_bytes())?;
    writer.write_all(&[fps])?;
    
    // 占位符 for frame_count (后面更新)
    let frame_count_pos = 8 + 1 + 2 + 2 + 1;
    writer.write_all(&[0u8; 4])?;
    
    // 使用 ffmpeg 提取帧并处理
    let blocks_per_row = (width as usize + 7) / 8;
    let blocks_per_col = (height as usize + 7) / 8;
    let frame_size = width as usize * height as usize * 3;
    
    println!("🔄 开始编码 (每帧 {} 个块)...", blocks_per_row * blocks_per_col);
    
    let mut ffmpeg = Command::new("ffmpeg")
        .args(&["-i", input_path,
                "-f", "rawvideo",
                "-pix_fmt", "rgb24",
                "-"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    
    let mut stdout = ffmpeg.stdout.take().unwrap();
    let mut frame_buffer = vec![0u8; frame_size];
    let mut frame_count = 0u32;
    
    loop {
        // 读取一帧
        match stdout.read_exact(&mut frame_buffer) {
            Ok(_) => {},
            Err(_) => break,
        }
        
        // 压缩帧
        for by in 0..blocks_per_col {
            for bx in 0..blocks_per_row {
                let block = CompressedBlock::from_pixels(
                    &frame_buffer,
                    width as usize,
                    bx * 8,
                    by * 8
                );
                writer.write_all(&block.to_bytes())?;
            }
        }
        
        frame_count += 1;
        if frame_count % 100 == 0 {
            print!("\r   已编码 {} 帧...", frame_count);
            io::stdout().flush()?;
        }
    }
    
    writer.flush()?;
    drop(writer);
    
    // 更新 frame_count
    let mut file = fs::OpenOptions::new().write(true).open(output_path)?;
    use std::io::Seek;
    file.seek(std::io::SeekFrom::Start(frame_count_pos as u64))?;
    file.write_all(&frame_count.to_le_bytes())?;
    
    let output_size = fs::metadata(output_path)?.len();
    let input_size = fs::metadata(input_path)?.len();
    let compression_ratio = (1.0 - output_size as f64 / input_size as f64) * 100.0;
    
    println!("\n✅ 编码完成!");
    println!("   帧数: {}", frame_count);
    println!("   原始: {:.2} MB", input_size as f64 / 1024.0 / 1024.0);
    println!("   压缩: {:.2} MB", output_size as f64 / 1024.0 / 1024.0);
    println!("   压缩率: {:.1}%", compression_ratio);
    
    Ok(())
}

fn decode_video(input_path: &str, output_path: &str) -> io::Result<()> {
    println!("📥 输入: {}", input_path);
    println!("📦 输出: {}", output_path);
    
    let input_file = File::open(input_path)?;
    let mut reader = BufReader::new(input_file);
    
    // 读取头部
    let mut magic = [0u8; 8];
    reader.read_exact(&mut magic)?;
    if &magic != b"VALHALLA" {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "不是 Valhalla 视频文件"));
    }
    
    let mut version = [0u8; 1];
    reader.read_exact(&mut version)?;
    
    let mut width_bytes = [0u8; 2];
    reader.read_exact(&mut width_bytes)?;
    let width = u16::from_le_bytes(width_bytes);
    
    let mut height_bytes = [0u8; 2];
    reader.read_exact(&mut height_bytes)?;
    let height = u16::from_le_bytes(height_bytes);
    
    let mut fps_bytes = [0u8; 1];
    reader.read_exact(&mut fps_bytes)?;
    let fps = fps_bytes[0];
    
    let mut frame_count_bytes = [0u8; 4];
    reader.read_exact(&mut frame_count_bytes)?;
    let frame_count = u32::from_le_bytes(frame_count_bytes);
    
    println!("📐 视频: {}x{} @ {} fps, {} 帧", width, height, fps, frame_count);
    
    let blocks_per_row = (width as usize + 7) / 8;
    let blocks_per_col = (height as usize + 7) / 8;
    let blocks_per_frame = blocks_per_row * blocks_per_col;
    
    println!("🔄 开始解码...");
    
    // 使用 ffmpeg 编码输出
    let mut ffmpeg = Command::new("ffmpeg")
        .args(&["-y",
                "-f", "rawvideo",
                "-pix_fmt", "rgb24",
                "-s", &format!("{}x{}", blocks_per_row * 8, blocks_per_col * 8),
                "-r", &fps.to_string(),
                "-i", "-",
                "-c:v", "libx264",
                "-preset", "fast",
                "-crf", "23",
                output_path])
        .stdin(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    
    let mut stdin = ffmpeg.stdin.take().unwrap();
    let frame_pixel_size = blocks_per_row * 8 * blocks_per_col * 8 * 3;
    let mut frame_pixels = vec![0u8; frame_pixel_size];
    let mut block_bytes = [0u8; 8];
    
    for f in 0..frame_count {
        // 解码每一帧
        for by in 0..blocks_per_col {
            for bx in 0..blocks_per_row {
                reader.read_exact(&mut block_bytes)?;
                let block = CompressedBlock::from_bytes(&block_bytes);
                let pixels = block.to_pixels();
                
                // 写入帧缓冲
                for dy in 0..8 {
                    for dx in 0..8 {
                        let src_idx = (dy * 8 + dx) * 3;
                        let dst_idx = ((by * 8 + dy) * blocks_per_row * 8 + (bx * 8 + dx)) * 3;
                        if dst_idx + 2 < frame_pixels.len() {
                            frame_pixels[dst_idx] = pixels[src_idx];
                            frame_pixels[dst_idx + 1] = pixels[src_idx + 1];
                            frame_pixels[dst_idx + 2] = pixels[src_idx + 2];
                        }
                    }
                }
            }
        }
        
        stdin.write_all(&frame_pixels)?;
        
        if (f + 1) % 100 == 0 {
            print!("\r   已解码 {}/{} 帧...", f + 1, frame_count);
            io::stdout().flush()?;
        }
    }
    
    drop(stdin);
    ffmpeg.wait()?;
    
    println!("\n✅ 解码完成!");
    
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🌌 Valhalla 视频编解码器                                   ║");
    println!("║     用 Valhalla 的方式压缩和播放视频                           ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    if args.len() < 3 {
        println!("用法:");
        println!("  编码: {} encode <input.mp4> <output.valhalla>", args[0]);
        println!("  解码: {} decode <input.valhalla> <output.mp4>", args[0]);
        println!("");
        println!("示例:");
        println!("  {} encode movie.mp4 movie.valhalla", args[0]);
        println!("  {} decode movie.valhalla movie_decoded.mp4", args[0]);
        return Ok(());
    }
    
    let command = &args[1];
    let input = &args[2];
    let output = if args.len() > 3 { &args[3] } else { 
        if command == "encode" { "output.valhalla" } else { "output.mp4" }
    };
    
    match command.as_str() {
        "encode" => encode_video_v3(input, output, 10)?,  // V3: 10次内部迭代
        "encode_v2" => encode_video_v2(input, output)?,   // V2: 帧间压缩
        "encode_v1" => encode_video(input, output)?,      // V1: 基础
        "decode" => {
            // 自动检测版本
            let mut f = File::open(input)?;
            let mut magic = [0u8; 8];
            f.read_exact(&mut magic)?;
            drop(f);
            
            if &magic == b"VALHALV3" {
                decode_video_v3(input, output)?;
            } else {
                decode_video(input, output)?;
            }
        },
        _ => {
            println!("未知命令: {}", command);
            println!("使用 'encode' 或 'decode'");
        }
    }
    
    Ok(())
}
