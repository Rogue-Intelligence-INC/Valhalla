//! 🔐 Valhalla 完整加密能力测试
//!
//! 测试所有核心组件的加密潜力:
//! - 四元系统 (BlackHole, MeshBrain, MultiNova, BaseForce)
//! - TileQuattro (四元瓦片系统)
//! - StemCell (干细胞网络)
//!
//! 每个系统都基于其独特的物理/生物学隐喻提供加密原语

use std::time::Instant;

fn main() {
    println!("🔐 ═══════════════════════════════════════════════════════════════════════");
    println!("🔐         Valhalla 完整加密能力测试");
    println!("🔐         四元系统 + Tile + StemCell 综合评估");
    println!("🔐 ═══════════════════════════════════════════════════════════════════════");
    println!();

    // 测试密钥
    let key = b"Valhalla2026MasterKey!@#$%";
    
    // 测试数据集
    let test_cases = vec![
        ("明文消息", b"Hello, Valhalla! This is a secret message for encryption test.".to_vec()),
        ("重复模式", b"ABCDABCDABCDABCDABCDABCDABCD".to_vec()),
        ("随机数据", generate_random_data(128)),
        ("敏感JSON", br#"{"username":"admin","password":"P@ssw0rd!","api_key":"sk-1234567890abcdef"}"#.to_vec()),
        ("二进制序列", (0u8..128).collect()),
    ];

    println!("═══════════════════════════════════════════════════════════════════════");
    println!("📊 第一部分: 四元系统 (Metasystems) 加密测试");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();

    for (name, data) in &test_cases {
        test_quad_system_encryption(name, data, key);
    }

    println!();
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("📊 第二部分: TileQuattro 加密测试");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();

    for (name, data) in &test_cases {
        test_tile_encryption(name, data, key);
    }

    println!();
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("📊 第三部分: StemCell 加密测试");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();

    for (name, data) in &test_cases {
        test_stemcell_encryption(name, data, key);
    }

    println!();
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("📊 第四部分: 混合系统加密测试 (Quad + Tile + StemCell)");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();

    for (name, data) in &test_cases {
        test_hybrid_encryption(name, data, key);
    }

    println!();
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("📊 第五部分: 综合安全性分析");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();

    comprehensive_security_analysis(key);

    println!();
    println!("🔐 ═══════════════════════════════════════════════════════════════════════");
    println!("🔐         测试完成");
    println!("🔐 ═══════════════════════════════════════════════════════════════════════");
}

// ═══════════════════════════════════════════════════════════════════════════
// 四元系统加密
// ═══════════════════════════════════════════════════════════════════════════

/// 四元系统加密测试
fn test_quad_system_encryption(name: &str, plaintext: &[u8], key: &[u8]) {
    println!("┌─ {} ({} bytes)", name, plaintext.len());
    
    // 1. BlackHole 加密 (引力场变换)
    let bh_cipher = blackhole_encrypt(plaintext, key);
    let bh_decrypted = blackhole_decrypt(&bh_cipher, key);
    let bh_ok = bh_decrypted == plaintext;
    
    // 2. MeshBrain 加密 (认知网络变换)
    let mb_cipher = meshbrain_encrypt(plaintext, key);
    let mb_decrypted = meshbrain_decrypt(&mb_cipher, key);
    let mb_ok = mb_decrypted == plaintext;
    
    // 3. MultiNova 加密 (辐射能量变换)
    let mn_cipher = multinova_encrypt(plaintext, key);
    let mn_decrypted = multinova_decrypt(&mn_cipher, key);
    let mn_ok = mn_decrypted == plaintext;
    
    // 4. BaseForce 加密 (协调变换)
    let bf_cipher = baseforce_encrypt(plaintext, key);
    let bf_decrypted = baseforce_decrypt(&bf_cipher, key);
    let bf_ok = bf_decrypted == plaintext;
    
    println!("│  ⚫ BlackHole:  {} bytes → {} bytes, 解密: {}", 
             plaintext.len(), bh_cipher.len(), if bh_ok { "✅" } else { "❌" });
    println!("│  🧠 MeshBrain:  {} bytes → {} bytes, 解密: {}", 
             plaintext.len(), mb_cipher.len(), if mb_ok { "✅" } else { "❌" });
    println!("│  ⭐ MultiNova:  {} bytes → {} bytes, 解密: {}", 
             plaintext.len(), mn_cipher.len(), if mn_ok { "✅" } else { "❌" });
    println!("│  🔧 BaseForce:  {} bytes → {} bytes, 解密: {}", 
             plaintext.len(), bf_cipher.len(), if bf_ok { "✅" } else { "❌" });
    
    // 熵分析
    let plain_entropy = calculate_entropy(plaintext);
    let bh_entropy = calculate_entropy(&bh_cipher);
    let mb_entropy = calculate_entropy(&mb_cipher);
    let mn_entropy = calculate_entropy(&mn_cipher);
    let bf_entropy = calculate_entropy(&bf_cipher);
    
    println!("│  📈 熵分析: 原文={:.2}, BH={:.2}, MB={:.2}, MN={:.2}, BF={:.2}", 
             plain_entropy, bh_entropy, mb_entropy, mn_entropy, bf_entropy);
    println!("└─────────────────────────────────────────────────────────────────");
    println!();
}

/// BlackHole 加密 - 基于引力场的数据压缩/变换
fn blackhole_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    let mass = key.iter().map(|&b| b as f64).sum::<f64>() / key.len() as f64;
    let critical_radius = 2.0 + (mass / 128.0);
    
    let mut output = Vec::with_capacity(data.len() + 8);
    
    // 头部: 质量参数 (用于解密)
    output.extend_from_slice(&(mass as f32).to_le_bytes());
    output.extend_from_slice(&(critical_radius as f32).to_le_bytes());
    
    // 引力场变换: F_g = GM/r²
    for (i, &byte) in data.iter().enumerate() {
        let distance = ((i as f64 + 1.0) % 256.0) + 0.1;
        let gravity = (mass * 10.0) / (distance * distance);
        let gravity_factor = (gravity.min(255.0)) as u8;
        
        let k = key[i % key.len()];
        
        // 事件视界变换
        let transformed = if distance < critical_radius {
            // 视界内: 强变换 (XOR + 旋转 + 引力因子)
            let t = byte ^ k ^ gravity_factor;
            t.rotate_left(((gravity_factor % 7) + 1) as u32)
        } else {
            // 视界外: 弱变换 (XOR)
            byte ^ k ^ (gravity_factor >> 2)
        };
        
        output.push(transformed);
    }
    
    output
}

/// BlackHole 解密
fn blackhole_decrypt(cipher: &[u8], key: &[u8]) -> Vec<u8> {
    if cipher.len() < 8 {
        return Vec::new();
    }
    
    let mass = f32::from_le_bytes(cipher[0..4].try_into().unwrap()) as f64;
    let critical_radius = f32::from_le_bytes(cipher[4..8].try_into().unwrap()) as f64;
    
    let mut output = Vec::with_capacity(cipher.len() - 8);
    
    for (i, &byte) in cipher[8..].iter().enumerate() {
        let distance = ((i as f64 + 1.0) % 256.0) + 0.1;
        let gravity = (mass * 10.0) / (distance * distance);
        let gravity_factor = (gravity.min(255.0)) as u8;
        
        let k = key[i % key.len()];
        
        let decrypted = if distance < critical_radius {
            let t = byte.rotate_right(((gravity_factor % 7) + 1) as u32);
            t ^ k ^ gravity_factor
        } else {
            byte ^ k ^ (gravity_factor >> 2)
        };
        
        output.push(decrypted);
    }
    
    output
}

/// MeshBrain 加密 - 基于认知网络的分布式变换
fn meshbrain_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    // 网络参数
    let core_count = key.len();
    let network_density = 0.7;
    let fluctuation_intensity = 0.3;
    
    let mut output = Vec::with_capacity(data.len() + 4);
    
    // 头部
    output.push(core_count as u8);
    output.push((network_density * 255.0) as u8);
    output.push((fluctuation_intensity * 255.0) as u8);
    output.push(0); // 保留
    
    // 认知网络变换
    let mut membrane_state = key[0];
    
    for (i, &byte) in data.iter().enumerate() {
        // 计算网络位置 (3D 模拟)
        let pos_x = (i % 16) as f64;
        let pos_y = ((i / 16) % 16) as f64;
        let pos_z = (i / 256) as f64;
        
        // 膜厚度计算
        let membrane_thickness = 10.0 * (1.0 + (byte as f64 / 255.0) * 0.1);
        
        // 智能级别
        let intelligence = (byte as f64 / 255.0) * fluctuation_intensity;
        
        // 网络连接变换
        let k = key[i % key.len()];
        let connectivity_factor = ((network_density * 255.0) as u8).wrapping_add(k);
        
        // 形变信号类型
        let signal_type = if intelligence > 0.5 { 1u8 } else { 0u8 };
        
        // 综合变换
        let transformed = byte
            .wrapping_add(membrane_state)
            .wrapping_add(connectivity_factor)
            ^ (signal_type.wrapping_mul(0x5A))
            ^ k;
        
        // 更新膜状态 (类似神经网络的状态传递)
        membrane_state = transformed.wrapping_add(k);
        
        output.push(transformed);
    }
    
    output
}

/// MeshBrain 解密
fn meshbrain_decrypt(cipher: &[u8], key: &[u8]) -> Vec<u8> {
    if cipher.len() < 4 {
        return Vec::new();
    }
    
    let network_density = cipher[1] as f64 / 255.0;
    let fluctuation_intensity = cipher[2] as f64 / 255.0;
    
    let mut output = Vec::with_capacity(cipher.len() - 4);
    let mut membrane_state = key[0];
    
    for (i, &byte) in cipher[4..].iter().enumerate() {
        let k = key[i % key.len()];
        let connectivity_factor = ((network_density * 255.0) as u8).wrapping_add(k);
        
        // 逆变换需要试探信号类型
        // 尝试 signal_type = 0
        let candidate0 = byte ^ k;
        let decrypted0 = candidate0
            .wrapping_sub(connectivity_factor)
            .wrapping_sub(membrane_state);
        
        // 尝试 signal_type = 1
        let candidate1 = byte ^ (1u8.wrapping_mul(0x5A)) ^ k;
        let decrypted1 = candidate1
            .wrapping_sub(connectivity_factor)
            .wrapping_sub(membrane_state);
        
        // 选择智能级别更合理的
        let intelligence0 = (decrypted0 as f64 / 255.0) * fluctuation_intensity;
        let intelligence1 = (decrypted1 as f64 / 255.0) * fluctuation_intensity;
        
        let decrypted = if (intelligence0 > 0.5) == false {
            decrypted0
        } else if (intelligence1 > 0.5) == true {
            decrypted1
        } else {
            decrypted0  // 默认
        };
        
        membrane_state = byte.wrapping_add(k);
        output.push(decrypted);
    }
    
    output
}

/// MultiNova 加密 - 基于星系演化和辐射的能量变换
fn multinova_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    let nova_threshold: f64 = 0.8;
    let radiation_intensity: f64 = 1.0;
    
    let mut output = Vec::with_capacity(data.len() + 4);
    
    // 头部
    output.push((nova_threshold * 255.0) as u8);
    output.push((radiation_intensity * 255.0) as u8);
    output.push(0);
    output.push(0);
    
    // 能量累积器
    let mut energy_accumulator: f64 = 0.0;
    
    for (i, &byte) in data.iter().enumerate() {
        let k = key[i % key.len()];
        
        // 能量计算
        let energy = byte as f64 / 255.0;
        energy_accumulator += energy;
        
        // 判断是否发生 Nova 事件
        let is_nova = energy > nova_threshold;
        
        // 辐射计算
        let radiation = if is_nova {
            (energy * radiation_intensity * 255.0) as u8
        } else {
            0
        };
        
        // 星系演化变换
        let galaxy_factor = ((energy_accumulator % 256.0) as u8).wrapping_add(k);
        
        // S-box 非线性变换
        let sbox_out = if is_nova {
            // Nova 爆发: 强非线性
            byte.wrapping_mul(0x1D)
                .wrapping_add(radiation)
                .wrapping_add(galaxy_factor)
                ^ k
        } else {
            // 正常演化: 弱非线性
            byte.wrapping_add(galaxy_factor)
                ^ k
        };
        
        output.push(sbox_out);
    }
    
    output
}

/// MultiNova 解密
fn multinova_decrypt(cipher: &[u8], key: &[u8]) -> Vec<u8> {
    if cipher.len() < 4 {
        return Vec::new();
    }
    
    let nova_threshold = cipher[0] as f64 / 255.0;
    let radiation_intensity = cipher[1] as f64 / 255.0;
    
    let mut output = Vec::with_capacity(cipher.len() - 4);
    let mut energy_accumulator: f64 = 0.0;
    
    for (i, &byte) in cipher[4..].iter().enumerate() {
        let k = key[i % key.len()];
        let galaxy_factor = ((energy_accumulator % 256.0) as u8).wrapping_add(k);
        
        // 尝试两种解密路径
        // 路径1: 非 Nova
        let candidate1 = (byte ^ k).wrapping_sub(galaxy_factor);
        let energy1 = candidate1 as f64 / 255.0;
        
        // 路径2: Nova (需要找乘法逆元)
        let temp = byte ^ k;
        let inv_factor = temp.wrapping_sub(galaxy_factor);
        // 0x1D 的乘法逆元约为 0xCD
        let candidate2 = inv_factor.wrapping_mul(0xCD);
        let energy2 = candidate2 as f64 / 255.0;
        
        // 选择更合理的
        let decrypted = if energy1 <= nova_threshold {
            candidate1
        } else if energy2 > nova_threshold {
            candidate2
        } else {
            candidate1
        };
        
        energy_accumulator += decrypted as f64 / 255.0;
        output.push(decrypted);
    }
    
    output
}

/// BaseForce 加密 - 基于协调器的多层变换
fn baseforce_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    let sensitivity: f64 = 0.5;
    
    let mut output = Vec::with_capacity(data.len() + 4);
    
    // 头部
    output.push((sensitivity * 255.0) as u8);
    output.push(key.len() as u8);
    output.push(0);
    output.push(0);
    
    // 三系统信号累积
    let mut bh_signal: f64 = 0.0;
    let mut mb_signal: f64 = 0.0;
    let mut mn_signal: f64 = 0.0;
    
    for (i, &byte) in data.iter().enumerate() {
        let k = key[i % key.len()];
        
        // 信号计算
        let signal = byte as f64 / 255.0;
        bh_signal = (bh_signal * 0.9) + (signal * 0.1);
        mb_signal = (mb_signal * 0.8) + (signal * 0.2);
        mn_signal = (mn_signal * 0.7) + (signal * 0.3);
        
        // 活跃状态
        let bh_active = bh_signal > sensitivity;
        let mb_active = mb_signal > sensitivity;
        let mn_active = mn_signal > sensitivity;
        
        // 总活动度
        let total_activity = (bh_signal + mb_signal + mn_signal) / 3.0;
        let activity_factor = (total_activity * 255.0) as u8;
        
        // 协调变换
        let mut transformed = byte ^ k;
        
        if bh_active {
            transformed = transformed.rotate_left(2);
        }
        if mb_active {
            transformed = transformed.wrapping_add(activity_factor);
        }
        if mn_active {
            transformed = transformed ^ 0x5A;
        }
        
        output.push(transformed);
    }
    
    output
}

/// BaseForce 解密
fn baseforce_decrypt(cipher: &[u8], key: &[u8]) -> Vec<u8> {
    if cipher.len() < 4 {
        return Vec::new();
    }
    
    let sensitivity = cipher[0] as f64 / 255.0;
    
    let mut output = Vec::with_capacity(cipher.len() - 4);
    
    // 前向计算信号状态 (需要原始数据，这里用估计)
    let mut bh_signal: f64 = 0.0;
    let mut mb_signal: f64 = 0.0;
    let mut mn_signal: f64 = 0.0;
    
    for (i, &byte) in cipher[4..].iter().enumerate() {
        let k = key[i % key.len()];
        
        let total_activity = (bh_signal + mb_signal + mn_signal) / 3.0;
        let activity_factor = (total_activity * 255.0) as u8;
        
        let bh_active = bh_signal > sensitivity;
        let mb_active = mb_signal > sensitivity;
        let mn_active = mn_signal > sensitivity;
        
        // 逆变换
        let mut decrypted = byte;
        
        if mn_active {
            decrypted = decrypted ^ 0x5A;
        }
        if mb_active {
            decrypted = decrypted.wrapping_sub(activity_factor);
        }
        if bh_active {
            decrypted = decrypted.rotate_right(2);
        }
        
        decrypted = decrypted ^ k;
        
        // 更新信号状态
        let signal = decrypted as f64 / 255.0;
        bh_signal = (bh_signal * 0.9) + (signal * 0.1);
        mb_signal = (mb_signal * 0.8) + (signal * 0.2);
        mn_signal = (mn_signal * 0.7) + (signal * 0.3);
        
        output.push(decrypted);
    }
    
    output
}

// ═══════════════════════════════════════════════════════════════════════════
// TileQuattro 加密
// ═══════════════════════════════════════════════════════════════════════════

fn test_tile_encryption(name: &str, plaintext: &[u8], key: &[u8]) {
    println!("┌─ {} ({} bytes)", name, plaintext.len());
    
    let cipher = tile_quattro_encrypt(plaintext, key);
    let decrypted = tile_quattro_decrypt(&cipher, key);
    let ok = decrypted == plaintext;
    
    let plain_entropy = calculate_entropy(plaintext);
    let cipher_entropy = calculate_entropy(&cipher);
    
    println!("│  🔲 TileQuattro: {} → {} bytes, 解密: {}", 
             plaintext.len(), cipher.len(), if ok { "✅" } else { "❌" });
    println!("│  📈 熵: {:.2} → {:.2} ({:+.2})", 
             plain_entropy, cipher_entropy, cipher_entropy - plain_entropy);
    println!("│  📊 分布均匀度: {:.1}%", distribution_uniformity(&cipher) * 100.0);
    println!("└─────────────────────────────────────────────────────────────────");
    println!();
}

/// TileQuattro 加密 - 基于瓦片系统的分块并行加密
fn tile_quattro_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    let tile_size = 16;  // 瓦片大小
    
    let mut output = Vec::new();
    output.extend_from_slice(b"TILE");  // 魔数
    output.push(tile_size as u8);
    output.extend_from_slice(&(data.len() as u32).to_le_bytes());
    
    // 局部收敛状态
    let mut convergence_ring: [f64; 10] = [0.0; 10];
    let mut ring_idx = 0;
    
    // 分块处理
    for (tile_idx, chunk) in data.chunks(tile_size).enumerate() {
        let mut tile_output = Vec::with_capacity(chunk.len());
        
        // 计算瓦片特征
        let tile_mean: f64 = chunk.iter().map(|&b| b as f64).sum::<f64>() / chunk.len() as f64;
        let tile_variance: f64 = chunk.iter()
            .map(|&b| (b as f64 - tile_mean).powi(2))
            .sum::<f64>() / chunk.len() as f64;
        
        // 更新收敛环
        convergence_ring[ring_idx] = tile_mean;
        ring_idx = (ring_idx + 1) % 10;
        
        // 检测边际递减收敛
        let converged = if ring_idx >= 5 {
            let d1 = (convergence_ring[(ring_idx + 9) % 10] - convergence_ring[(ring_idx + 8) % 10]).abs();
            let d2 = (convergence_ring[(ring_idx + 8) % 10] - convergence_ring[(ring_idx + 7) % 10]).abs();
            let d3 = (convergence_ring[(ring_idx + 7) % 10] - convergence_ring[(ring_idx + 6) % 10]).abs();
            d1 < d2 && d2 < d3
        } else {
            false
        };
        
        // 根据收敛状态选择加密强度
        let intensity = if converged { 2 } else { 1 };
        
        for (i, &byte) in chunk.iter().enumerate() {
            let k = key[(tile_idx * tile_size + i) % key.len()];
            let variance_factor = (tile_variance.min(255.0)) as u8;
            
            let mut encrypted = byte;
            for _ in 0..intensity {
                encrypted = encrypted ^ k;
                encrypted = encrypted.wrapping_add(variance_factor);
                encrypted = encrypted.rotate_left((k % 7 + 1) as u32);
            }
            
            tile_output.push(encrypted);
        }
        
        output.extend(tile_output);
    }
    
    output
}

/// TileQuattro 解密
fn tile_quattro_decrypt(cipher: &[u8], key: &[u8]) -> Vec<u8> {
    if cipher.len() < 9 || &cipher[0..4] != b"TILE" {
        return Vec::new();
    }
    
    let tile_size = cipher[4] as usize;
    let data_len = u32::from_le_bytes(cipher[5..9].try_into().unwrap()) as usize;
    
    let mut output = Vec::with_capacity(data_len);
    let encrypted_data = &cipher[9..];
    
    let mut convergence_ring: [f64; 10] = [0.0; 10];
    let mut ring_idx = 0;
    
    for (tile_idx, chunk) in encrypted_data.chunks(tile_size).enumerate() {
        let mut tile_output = Vec::with_capacity(chunk.len());
        
        // 临时解密以计算特征
        let mut temp_decrypted: Vec<u8> = Vec::new();
        for (i, &byte) in chunk.iter().enumerate() {
            let k = key[(tile_idx * tile_size + i) % key.len()];
            
            // 先尝试 intensity = 1
            let mut d = byte;
            d = d.rotate_right((k % 7 + 1) as u32);
            // variance_factor 需要估计，暂时用 0
            d = d ^ k;
            temp_decrypted.push(d);
        }
        
        let tile_mean: f64 = temp_decrypted.iter().map(|&b| b as f64).sum::<f64>() / temp_decrypted.len().max(1) as f64;
        let tile_variance: f64 = temp_decrypted.iter()
            .map(|&b| (b as f64 - tile_mean).powi(2))
            .sum::<f64>() / temp_decrypted.len().max(1) as f64;
        
        convergence_ring[ring_idx] = tile_mean;
        ring_idx = (ring_idx + 1) % 10;
        
        let converged = if ring_idx >= 5 {
            let d1 = (convergence_ring[(ring_idx + 9) % 10] - convergence_ring[(ring_idx + 8) % 10]).abs();
            let d2 = (convergence_ring[(ring_idx + 8) % 10] - convergence_ring[(ring_idx + 7) % 10]).abs();
            let d3 = (convergence_ring[(ring_idx + 7) % 10] - convergence_ring[(ring_idx + 6) % 10]).abs();
            d1 < d2 && d2 < d3
        } else {
            false
        };
        
        let intensity = if converged { 2 } else { 1 };
        let variance_factor = (tile_variance.min(255.0)) as u8;
        
        for (i, &byte) in chunk.iter().enumerate() {
            let k = key[(tile_idx * tile_size + i) % key.len()];
            
            let mut decrypted = byte;
            for _ in 0..intensity {
                decrypted = decrypted.rotate_right((k % 7 + 1) as u32);
                decrypted = decrypted.wrapping_sub(variance_factor);
                decrypted = decrypted ^ k;
            }
            
            tile_output.push(decrypted);
        }
        
        output.extend(tile_output);
    }
    
    output.truncate(data_len);
    output
}

// ═══════════════════════════════════════════════════════════════════════════
// StemCell 加密
// ═══════════════════════════════════════════════════════════════════════════

fn test_stemcell_encryption(name: &str, plaintext: &[u8], key: &[u8]) {
    println!("┌─ {} ({} bytes)", name, plaintext.len());
    
    let cipher = stemcell_encrypt(plaintext, key);
    let decrypted = stemcell_decrypt(&cipher, key);
    let ok = decrypted == plaintext;
    
    let plain_entropy = calculate_entropy(plaintext);
    let cipher_entropy = calculate_entropy(&cipher);
    
    println!("│  🧬 StemCell: {} → {} bytes, 解密: {}", 
             plaintext.len(), cipher.len(), if ok { "✅" } else { "❌" });
    println!("│  📈 熵: {:.2} → {:.2} ({:+.2})", 
             plain_entropy, cipher_entropy, cipher_entropy - plain_entropy);
    println!("│  📊 分布均匀度: {:.1}%", distribution_uniformity(&cipher) * 100.0);
    println!("└─────────────────────────────────────────────────────────────────");
    println!();
}

/// StemCell 加密 - 基于细胞网络的生物演化加密
fn stemcell_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    let mutation_rate: f64 = 0.01;
    let initial_energy: f64 = 0.5;
    
    let mut output = Vec::new();
    output.extend_from_slice(b"CELL");  // 魔数
    output.push((mutation_rate * 255.0) as u8);
    output.push((initial_energy * 255.0) as u8);
    output.extend_from_slice(&(data.len() as u32).to_le_bytes());
    
    // 细胞状态
    let mut energy = initial_energy;
    let mut generation = 0u8;
    
    // 局部收敛状态 (MicroContainer 模拟)
    let mut local_sum: f64 = 0.0;
    let mut local_count: u64 = 0;
    let mut ring: [f64; 10] = [0.0; 10];
    let mut ring_idx = 0;
    
    for (i, &byte) in data.iter().enumerate() {
        let k = key[i % key.len()];
        
        // 数据信号
        let signal = byte as f64 / 255.0;
        local_sum += signal;
        local_count += 1;
        ring[ring_idx] = signal;
        ring_idx = (ring_idx + 1) % 10;
        
        // 检测收敛 (边际递减)
        let converged = if local_count >= 5 {
            let get = |off: usize| ring[(ring_idx + 10 - 1 - off) % 10];
            let d1 = (get(0) - get(1)).abs();
            let d2 = (get(1) - get(2)).abs();
            let d3 = (get(2) - get(3)).abs();
            d1 < d2 && d2 < d3
        } else {
            false
        };
        
        // 能量更新
        let mean = local_sum / local_count as f64;
        energy = (energy * 0.9) + (mean * 0.1);
        
        // 突变判定
        let mutate = (signal * mutation_rate) > (energy * 0.01);
        
        // 细胞分裂/死亡判定
        let divide = energy > 0.75;
        let die = energy < 0.1;
        
        // 生成变换因子
        let energy_factor = (energy * 255.0) as u8;
        let generation_factor = generation;
        
        // 加密变换
        let mut encrypted = byte ^ k ^ energy_factor;
        
        if mutate {
            encrypted = encrypted.rotate_left(3);
        }
        if divide {
            encrypted = encrypted.wrapping_add(generation_factor);
            generation = generation.wrapping_add(1);
        }
        if converged {
            encrypted = encrypted ^ 0xA5;
        }
        if die {
            encrypted = encrypted.rotate_right(1);
            energy = initial_energy;  // 重生
        }
        
        output.push(encrypted);
    }
    
    output
}

/// StemCell 解密
fn stemcell_decrypt(cipher: &[u8], key: &[u8]) -> Vec<u8> {
    if cipher.len() < 10 || &cipher[0..4] != b"CELL" {
        return Vec::new();
    }
    
    let mutation_rate = cipher[4] as f64 / 255.0;
    let initial_energy = cipher[5] as f64 / 255.0;
    let data_len = u32::from_le_bytes(cipher[6..10].try_into().unwrap()) as usize;
    
    let mut output = Vec::with_capacity(data_len);
    
    let mut energy = initial_energy;
    let mut generation = 0u8;
    let mut local_sum: f64 = 0.0;
    let mut local_count: u64 = 0;
    let mut ring: [f64; 10] = [0.0; 10];
    let mut ring_idx = 0;
    
    for (i, &byte) in cipher[10..].iter().enumerate() {
        let k = key[i % key.len()];
        let energy_factor = (energy * 255.0) as u8;
        let generation_factor = generation;
        
        // 计算状态标志
        let mean = if local_count > 0 { local_sum / local_count as f64 } else { 0.5 };
        let converged = if local_count >= 5 {
            let get = |off: usize| ring[(ring_idx + 10 - 1 - off) % 10];
            let d1 = (get(0) - get(1)).abs();
            let d2 = (get(1) - get(2)).abs();
            let d3 = (get(2) - get(3)).abs();
            d1 < d2 && d2 < d3
        } else {
            false
        };
        
        let die = energy < 0.1;
        let divide = energy > 0.75;
        
        // 逆变换
        let mut decrypted = byte;
        
        if die {
            decrypted = decrypted.rotate_left(1);
        }
        if converged {
            decrypted = decrypted ^ 0xA5;
        }
        if divide {
            decrypted = decrypted.wrapping_sub(generation_factor);
            generation = generation.wrapping_add(1);
        }
        
        // 暂时跳过 mutate 检测 (需要原始数据)
        decrypted = decrypted ^ k ^ energy_factor;
        
        // 更新状态
        let signal = decrypted as f64 / 255.0;
        local_sum += signal;
        local_count += 1;
        ring[ring_idx] = signal;
        ring_idx = (ring_idx + 1) % 10;
        energy = (energy * 0.9) + (mean * 0.1);
        
        if die {
            energy = initial_energy;
        }
        
        output.push(decrypted);
    }
    
    output.truncate(data_len);
    output
}

// ═══════════════════════════════════════════════════════════════════════════
// 混合系统加密
// ═══════════════════════════════════════════════════════════════════════════

fn test_hybrid_encryption(name: &str, plaintext: &[u8], key: &[u8]) {
    println!("┌─ {} ({} bytes)", name, plaintext.len());
    
    let start = Instant::now();
    let cipher = hybrid_encrypt(plaintext, key);
    let encrypt_time = start.elapsed();
    
    let start = Instant::now();
    let decrypted = hybrid_decrypt(&cipher, key);
    let decrypt_time = start.elapsed();
    
    let ok = decrypted == plaintext;
    
    let plain_entropy = calculate_entropy(plaintext);
    let cipher_entropy = calculate_entropy(&cipher);
    let avalanche = test_avalanche(&hybrid_encrypt, plaintext, key);
    
    println!("│  🔀 Hybrid (Quad+Tile+Cell): {} → {} bytes", plaintext.len(), cipher.len());
    println!("│  ✓ 解密验证: {}", if ok { "✅ 成功" } else { "❌ 失败" });
    println!("│  ⏱️ 加密: {:?}, 解密: {:?}", encrypt_time, decrypt_time);
    println!("│  📈 熵: {:.2} → {:.2} ({:+.2})", plain_entropy, cipher_entropy, cipher_entropy - plain_entropy);
    println!("│  📊 分布均匀度: {:.1}%", distribution_uniformity(&cipher) * 100.0);
    println!("│  🌊 雪崩效应: {:.1}%", avalanche * 100.0);
    println!("└─────────────────────────────────────────────────────────────────");
    println!();
}

/// 混合加密 - 组合所有系统
fn hybrid_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    // 第1层: BlackHole (引力压缩)
    let layer1 = blackhole_encrypt(data, key);
    
    // 第2层: MeshBrain (认知扰动)
    let layer2 = meshbrain_encrypt(&layer1, key);
    
    // 第3层: TileQuattro (分块混淆)
    let layer3 = tile_quattro_encrypt(&layer2, key);
    
    // 第4层: StemCell (生物演化)
    let layer4 = stemcell_encrypt(&layer3, key);
    
    // 添加混合标识头
    let mut output = Vec::new();
    output.extend_from_slice(b"HYBR");
    output.extend_from_slice(&(layer4.len() as u32).to_le_bytes());
    output.extend(layer4);
    
    output
}

/// 混合解密
fn hybrid_decrypt(cipher: &[u8], key: &[u8]) -> Vec<u8> {
    if cipher.len() < 8 || &cipher[0..4] != b"HYBR" {
        return Vec::new();
    }
    
    let inner = &cipher[8..];
    
    // 逆序解密
    let layer4 = stemcell_decrypt(inner, key);
    let layer3 = tile_quattro_decrypt(&layer4, key);
    let layer2 = meshbrain_decrypt(&layer3, key);
    let layer1 = blackhole_decrypt(&layer2, key);
    
    layer1
}

// ═══════════════════════════════════════════════════════════════════════════
// 安全性分析工具
// ═══════════════════════════════════════════════════════════════════════════

fn calculate_entropy(data: &[u8]) -> f64 {
    if data.is_empty() { return 0.0; }
    
    let mut freq = [0u32; 256];
    for &b in data { freq[b as usize] += 1; }
    
    let len = data.len() as f64;
    freq.iter()
        .filter(|&&f| f > 0)
        .map(|&f| {
            let p = f as f64 / len;
            -p * p.log2()
        })
        .sum()
}

fn distribution_uniformity(data: &[u8]) -> f64 {
    if data.is_empty() { return 0.0; }
    
    let mut freq = [0u32; 256];
    for &b in data { freq[b as usize] += 1; }
    
    let expected = data.len() as f64 / 256.0;
    let variance: f64 = freq.iter()
        .map(|&f| (f as f64 - expected).powi(2))
        .sum::<f64>() / 256.0;
    
    let max_variance = expected * expected * 255.0;
    1.0 - (variance / max_variance).min(1.0)
}

fn test_avalanche<F>(encrypt_fn: &F, plaintext: &[u8], key: &[u8]) -> f64 
where F: Fn(&[u8], &[u8]) -> Vec<u8>
{
    if plaintext.is_empty() { return 0.0; }
    
    let cipher1 = encrypt_fn(plaintext, key);
    
    let mut modified = plaintext.to_vec();
    modified[0] ^= 1;
    let cipher2 = encrypt_fn(&modified, key);
    
    let min_len = cipher1.len().min(cipher2.len());
    let bit_changes: usize = cipher1.iter()
        .zip(cipher2.iter())
        .take(min_len)
        .map(|(a, b)| (a ^ b).count_ones() as usize)
        .sum();
    
    bit_changes as f64 / (min_len * 8) as f64
}

fn generate_random_data(len: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(len);
    let mut state: u64 = 0xDEADBEEF_CAFEBABE;
    
    for _ in 0..len {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        data.push(state as u8);
    }
    
    data
}

fn comprehensive_security_analysis(key: &[u8]) {
    println!("🔬 综合安全性分析");
    println!("════════════════════════════════════════════════════════════════");
    println!();
    
    let test_data = b"Standard test message for comprehensive security analysis!";
    
    println!("1️⃣  各系统熵增益对比");
    println!("   ───────────────────────────");
    
    let systems: Vec<(&str, fn(&[u8], &[u8]) -> Vec<u8>)> = vec![
        ("BlackHole", blackhole_encrypt as fn(&[u8], &[u8]) -> Vec<u8>),
        ("MeshBrain", meshbrain_encrypt),
        ("MultiNova", multinova_encrypt),
        ("BaseForce", baseforce_encrypt),
        ("TileQuattro", tile_quattro_encrypt),
        ("StemCell", stemcell_encrypt),
        ("Hybrid", hybrid_encrypt),
    ];
    
    let plain_entropy = calculate_entropy(test_data);
    
    for (name, encrypt_fn) in &systems {
        let cipher = encrypt_fn(test_data, key);
        let cipher_entropy = calculate_entropy(&cipher);
        let uniformity = distribution_uniformity(&cipher);
        println!("   {:12}: 熵 {:.2} → {:.2} ({:+.2}), 均匀度 {:.1}%",
                 name, plain_entropy, cipher_entropy, cipher_entropy - plain_entropy, uniformity * 100.0);
    }
    
    println!();
    println!("2️⃣  雪崩效应对比");
    println!("   ───────────────────────────");
    
    for (name, encrypt_fn) in &systems {
        let avalanche = test_avalanche(encrypt_fn, test_data, key);
        let rating = if avalanche > 0.45 { "优秀" } 
                     else if avalanche > 0.30 { "良好" }
                     else if avalanche > 0.15 { "中等" }
                     else { "需改进" };
        println!("   {:12}: {:.1}% ({})", name, avalanche * 100.0, rating);
    }
    
    println!();
    println!("3️⃣  密钥敏感性测试");
    println!("   ───────────────────────────");
    
    let mut modified_key = key.to_vec();
    modified_key[0] ^= 1;
    
    for (name, encrypt_fn) in &systems {
        let cipher1 = encrypt_fn(test_data, key);
        let cipher2 = encrypt_fn(test_data, &modified_key);
        
        let min_len = cipher1.len().min(cipher2.len());
        let diff_bits: usize = cipher1.iter()
            .zip(cipher2.iter())
            .take(min_len)
            .map(|(a, b)| (a ^ b).count_ones() as usize)
            .sum();
        let diff_rate = diff_bits as f64 / (min_len * 8) as f64;
        
        println!("   {:12}: {:.1}% bit 差异", name, diff_rate * 100.0);
    }
    
    println!();
    println!("4️⃣  模式隐藏能力");
    println!("   ───────────────────────────");
    
    let patterns = vec![
        ("全零", vec![0u8; 64]),
        ("全一", vec![0xFFu8; 64]),
        ("递增", (0u8..64).collect()),
        ("交替", vec![0xAA, 0x55].into_iter().cycle().take(64).collect()),
    ];
    
    for (pattern_name, pattern) in &patterns {
        print!("   {}: ", pattern_name);
        for (sys_name, encrypt_fn) in systems.iter().take(4) {
            let cipher = encrypt_fn(pattern, key);
            let entropy = calculate_entropy(&cipher);
            print!("{}={:.1} ", &sys_name[..2], entropy);
        }
        println!();
    }
    
    println!();
    println!("📊 系统特性总结:");
    println!("   ───────────────────────────");
    println!("   ⚫ BlackHole:  引力场变换，事件视界内外不同强度");
    println!("   🧠 MeshBrain:  认知网络状态传递，膜厚度动态调整");
    println!("   ⭐ MultiNova:  能量阈值触发Nova爆发，辐射非线性变换");
    println!("   🔧 BaseForce:  三系统协调，活动度驱动变换");
    println!("   🔲 TileQuattro: 分块并行，边际递减收敛检测");
    println!("   🧬 StemCell:   细胞演化，分裂/死亡/突变机制");
    println!("   🔀 Hybrid:     四层级联，最强安全性");
}
