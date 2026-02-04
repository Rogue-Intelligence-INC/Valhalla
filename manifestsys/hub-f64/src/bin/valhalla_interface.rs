//! 🌌 Valhalla 双向对等接口
//!
//! 设计原则：
//! 1. 双向对等 - Valhalla 也可以主动输出
//! 2. 不预设协议 - 原始字节流，双方自己学会
//! 3. 从内部状态产生输出 - 不从外部语料
//! 4. 不改变核心状态 - 只读模式（除非显式学习）

use std::io::{self, Read, Write, BufReader, BufWriter};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;
use std::path::Path;
use std::fs::File;

/// Valhalla 内部状态（从冻结文件加载）
#[derive(Clone)]
struct ValhallaCore {
    /// Fate 偏好
    preferences: [f64; 4],
    /// 内部向量空间（简化表示）
    internal_vectors: Vec<[f64; 4]>,
    /// 信号历史摘要
    signal_digest: Vec<f64>,
    /// 总信号数
    total_signals: u64,
    /// 训练周期
    trained_cycles: u32,
}

impl ValhallaCore {
    /// 从冻结文件加载
    fn load_frozen(path: &Path) -> io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            let prefs = json.get("preferences")
                .and_then(|p| p.as_array())
                .map(|arr| {
                    let mut prefs = [0.25; 4];
                    for (i, v) in arr.iter().take(4).enumerate() {
                        prefs[i] = v.as_f64().unwrap_or(0.25);
                    }
                    prefs
                })
                .unwrap_or([0.25; 4]);
            
            let total_signals = json.get("total_signals")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            
            let cycles = json.get("cycles")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;
            
            // 从偏好生成内部向量空间（简化）
            let mut internal_vectors = Vec::new();
            for i in 0..1000 {
                let seed = i as f64 / 1000.0;
                internal_vectors.push([
                    prefs[0] * seed,
                    prefs[1] * (1.0 - seed),
                    prefs[2] * seed.sin(),
                    prefs[3] * seed.cos(),
                ]);
            }
            
            Ok(Self {
                preferences: prefs,
                internal_vectors,
                signal_digest: vec![prefs[0], prefs[1], prefs[2], prefs[3]],
                total_signals,
                trained_cycles: cycles,
            })
        } else {
            Ok(Self::default())
        }
    }
    
    fn default() -> Self {
        Self {
            preferences: [0.25; 4],
            internal_vectors: vec![[0.25; 4]; 100],
            signal_digest: vec![0.25; 4],
            total_signals: 0,
            trained_cycles: 0,
        }
    }
    
    /// 从内部状态产生输出（基于当前冲动）
    fn generate_impulse(&self, tick: u64) -> Vec<u8> {
        // 基于内部向量和 tick 生成输出
        let idx = (tick as usize) % self.internal_vectors.len().max(1);
        let vec = if idx < self.internal_vectors.len() {
            self.internal_vectors[idx]
        } else {
            self.preferences
        };
        
        // 将内部向量编码为字节
        let mut output = Vec::new();
        for v in vec.iter() {
            let bytes = v.to_le_bytes();
            output.extend_from_slice(&bytes);
        }
        
        // 添加 Fate 偏好的影响
        let dominant = self.preferences.iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);
        
        output.push(dominant as u8);
        output.push((tick % 256) as u8);
        
        output
    }
    
    /// 处理外部输入（只读模式 - 不修改状态）
    fn process_input(&self, input: &[u8]) -> Vec<u8> {
        if input.is_empty() {
            return vec![];
        }
        
        // 将输入映射到内部空间，找到最相似的向量
        let input_sum: f64 = input.iter().map(|&b| b as f64).sum();
        let normalized = input_sum / (input.len() as f64 * 255.0);
        
        // 在内部向量空间中查找最近的
        let mut best_idx = 0;
        let mut best_dist = f64::MAX;
        
        for (i, vec) in self.internal_vectors.iter().enumerate() {
            let vec_sum: f64 = vec.iter().sum();
            let dist = (vec_sum - normalized).abs();
            if dist < best_dist {
                best_dist = dist;
                best_idx = i;
            }
        }
        
        // 返回对应的内部向量编码
        let response_vec = &self.internal_vectors[best_idx];
        let mut response = Vec::new();
        for v in response_vec.iter() {
            let bytes = v.to_le_bytes();
            response.extend_from_slice(&bytes);
        }
        
        // 附加相似度信息
        let similarity = 1.0 - best_dist.min(1.0);
        response.extend_from_slice(&similarity.to_le_bytes());
        
        response
    }
}

/// 双向通道
struct BidirectionalChannel {
    stream: TcpStream,
    core: Arc<Mutex<ValhallaCore>>,
    running: Arc<AtomicBool>,
}

impl BidirectionalChannel {
    fn new(stream: TcpStream, core: Arc<Mutex<ValhallaCore>>) -> Self {
        stream.set_nonblocking(true).ok();
        Self {
            stream,
            core,
            running: Arc::new(AtomicBool::new(true)),
        }
    }
    
    /// 主循环 - 双向对等
    fn run(&mut self) {
        let mut tick = 0u64;
        let mut read_buf = [0u8; 1024];
        let mut impulse_cooldown = 0u32;
        
        println!("🔗 通道建立，双向对等模式");
        
        while self.running.load(Ordering::Relaxed) {
            // 1. 尝试读取外部输入
            match self.stream.read(&mut read_buf) {
                Ok(0) => {
                    // 连接关闭
                    println!("🔌 通道关闭");
                    break;
                }
                Ok(n) => {
                    // 收到外部输入
                    let input = &read_buf[..n];
                    println!("📥 收到 {} 字节", n);
                    
                    // 处理输入，产生响应
                    let core = self.core.lock().unwrap();
                    let response = core.process_input(input);
                    drop(core);
                    
                    if !response.is_empty() {
                        if let Err(e) = self.stream.write_all(&response) {
                            eprintln!("写入错误: {}", e);
                        }
                        println!("📤 响应 {} 字节", response.len());
                    }
                    
                    impulse_cooldown = 10; // 收到输入后暂缓主动输出
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // 没有输入，正常
                }
                Err(e) => {
                    eprintln!("读取错误: {}", e);
                    break;
                }
            }
            
            // 2. 主动输出（如果有冲动）
            if impulse_cooldown == 0 {
                let core = self.core.lock().unwrap();
                
                // 基于 Fate 偏好决定是否有表达冲动
                let dominant_pref = core.preferences.iter().cloned().fold(0.0, f64::max);
                let impulse_threshold = 0.5 + (tick % 100) as f64 / 200.0;
                
                if dominant_pref > impulse_threshold {
                    let output = core.generate_impulse(tick);
                    drop(core);
                    
                    if let Err(e) = self.stream.write_all(&output) {
                        eprintln!("主动输出错误: {}", e);
                    } else {
                        println!("💫 主动输出 {} 字节 (tick={})", output.len(), tick);
                    }
                    
                    impulse_cooldown = 50; // 主动输出后冷却
                }
            } else {
                impulse_cooldown = impulse_cooldown.saturating_sub(1);
            }
            
            tick += 1;
            thread::sleep(Duration::from_millis(100));
        }
    }
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🌌 Valhalla 双向对等接口                                   ║");
    println!("║     原始通道 - 无协议定义 - 双方自己学会                       ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    // 加载冻结状态
    let frozen_path = Path::new("/run/media/jouly/My Book/valhalla_hub/frozen_state.json");
    let core = if frozen_path.exists() {
        println!("📂 加载冻结状态: {}", frozen_path.display());
        match ValhallaCore::load_frozen(frozen_path) {
            Ok(c) => {
                println!("   周期: {} | 信号: {}", c.trained_cycles, c.total_signals);
                println!("   偏好: BH={:.2}% MB={:.2}% MN={:.2}% BF={:.2}%",
                         c.preferences[0] * 100.0, c.preferences[1] * 100.0,
                         c.preferences[2] * 100.0, c.preferences[3] * 100.0);
                c
            }
            Err(e) => {
                eprintln!("加载失败: {}, 使用默认状态", e);
                ValhallaCore::default()
            }
        }
    } else {
        println!("⚠️  未找到冻结状态，使用默认状态");
        println!("   请先运行训练并冻结: valhalla_train_freeze");
        ValhallaCore::default()
    };
    
    let core = Arc::new(Mutex::new(core));
    
    // 启动 TCP 服务器
    let addr = "127.0.0.1:7777";
    let listener = TcpListener::bind(addr).expect("无法绑定地址");
    println!("\n🔊 监听: {}", addr);
    println!("   连接方式: nc {} 或 telnet {}", addr, addr);
    println!("   这是原始字节通道，没有协议定义");
    println!("   Valhalla 会主动输出，也会响应输入\n");
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("🔗 新连接: {:?}", stream.peer_addr());
                let core_clone = Arc::clone(&core);
                thread::spawn(move || {
                    let mut channel = BidirectionalChannel::new(stream, core_clone);
                    channel.run();
                });
            }
            Err(e) => {
                eprintln!("连接错误: {}", e);
            }
        }
    }
}
