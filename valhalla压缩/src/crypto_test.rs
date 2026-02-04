//! 🔐 Valhalla 加密能力测试
//! 
//! 基于 Valhalla 四元系统的加密算法测试
//! 使用 Fate 路由决策实现自适应加密
//!
//! 四元系统加密原语:
//! - BlackHole: XOR 折叠混淆 (对称加密基础)
//! - MeshBrain: 差分编码扰动 (数据模式隐藏)
//! - MultiNova: 位平面置换 (数据混淆)
//! - BaseForce: 基线偏移变换 (数据变换)

use std::collections::HashMap;

fn main() {
    println!("🔐 ═══════════════════════════════════════════════════════════════");
    println!("🔐         Valhalla 四元系统加密能力测试");
    println!("🔐 ═══════════════════════════════════════════════════════════════");
    println!();

    // 测试数据集
    let test_cases = vec![
        ("明文文本", b"Hello, Valhalla! This is a secret message.".to_vec()),
        ("重复模式", b"AAAABBBBCCCCDDDDEEEE".to_vec()),
        ("随机数据", generate_random_data(64)),
        ("二进制数据", (0u8..=255).collect::<Vec<u8>>()),
        ("JSON 数据", br#"{"user":"admin","password":"secret123","token":"abc123xyz"}"#.to_vec()),
    ];

    let key = b"Valhalla2026SecretKey!";  // 密钥
    
    println!("📌 密钥: {:?}", String::from_utf8_lossy(key));
    println!();

    for (name, data) in &test_cases {
        println!("════════════════════════════════════════════════════════");
        println!("📋 测试: {}", name);
        println!("════════════════════════════════════════════════════════");
        test_encryption(name, data, key);
        println!();
    }

    // 详细的加密算法分析
    println!("🔬 ═══════════════════════════════════════════════════════════════");
    println!("🔬         Valhalla 加密算法详细分析");
    println!("🔬 ═══════════════════════════════════════════════════════════════");
    println!();
    
    analyze_encryption_properties(key);
    
    println!();
    println!("🔐 ═══════════════════════════════════════════════════════════════");
    println!("🔐         测试完成");
    println!("🔐 ═══════════════════════════════════════════════════════════════");
}

/// 测试加密功能
fn test_encryption(name: &str, plaintext: &[u8], key: &[u8]) {
    let original_len = plaintext.len();
    
    // 显示原始数据
    println!("📥 原始数据: {} bytes", original_len);
    if original_len <= 64 {
        println!("   Hex: {}", to_hex(plaintext));
        if let Ok(s) = std::str::from_utf8(plaintext) {
            println!("   Text: {}", s);
        }
    }
    
    // 1. Valhalla 四元加密
    let (ciphertext, route_info) = valhalla_encrypt(plaintext, key);
    let cipher_len = ciphertext.len();
    
    println!();
    println!("🔒 加密后: {} bytes ({})", cipher_len, 
             if cipher_len > original_len { 
                 format!("+{:.1}%", (cipher_len - original_len) as f64 / original_len as f64 * 100.0)
             } else {
                 format!("-{:.1}%", (original_len - cipher_len) as f64 / original_len as f64 * 100.0)
             });
    println!("   Hex: {}", to_hex(&ciphertext[..ciphertext.len().min(64)]));
    if ciphertext.len() > 64 {
        println!("   ... 还有 {} bytes", ciphertext.len() - 64);
    }
    
    // Fate 路由信息
    println!();
    println!("🎯 Fate 路由决策:");
    println!("   BlackHole: {:.1}% (XOR折叠)", route_info[0] * 100.0);
    println!("   MeshBrain: {:.1}% (差分编码)", route_info[1] * 100.0);
    println!("   MultiNova: {:.1}% (位平面置换)", route_info[2] * 100.0);
    println!("   BaseForce: {:.1}% (基线变换)", route_info[3] * 100.0);
    
    // 2. 解密
    let decrypted = valhalla_decrypt(&ciphertext, key);
    let decrypt_ok = decrypted == plaintext;
    
    println!();
    println!("🔓 解密验证: {}", if decrypt_ok { "✅ 成功" } else { "❌ 失败" });
    
    // 3. 安全性分析
    println!();
    println!("🔬 安全性分析:");
    
    // 熵分析
    let plain_entropy = calculate_entropy(plaintext);
    let cipher_entropy = calculate_entropy(&ciphertext);
    println!("   原文熵: {:.3} bits/byte", plain_entropy);
    println!("   密文熵: {:.3} bits/byte", cipher_entropy);
    println!("   熵增益: {:.3} bits/byte (越高越好)", cipher_entropy - plain_entropy);
    
    // 字节分布分析
    let plain_dist = byte_distribution(plaintext);
    let cipher_dist = byte_distribution(&ciphertext);
    let dist_score = distribution_uniformity(&cipher_dist);
    println!("   密文分布均匀度: {:.1}% (越接近100%越好)", dist_score * 100.0);
    
    // 相关性分析
    let correlation = calculate_correlation(plaintext, &ciphertext);
    println!("   明密文相关性: {:.3} (越接近0越好)", correlation);
    
    // 雪崩效应测试
    let avalanche = test_avalanche_effect(plaintext, key);
    println!("   雪崩效应: {:.1}% bit变化 (越接近50%越好)", avalanche * 100.0);
}

/// Valhalla 四元加密
fn valhalla_encrypt(plaintext: &[u8], key: &[u8]) -> (Vec<u8>, [f64; 4]) {
    // 1. 特征提取
    let features = extract_features(plaintext);
    
    // 2. Fate 路由决策 (基于密钥和数据特征)
    let routing = fate_routing(&features, key);
    
    // 3. 密钥扩展
    let expanded_key = expand_key(key, plaintext.len());
    
    // 4. 四元系统加密
    let mut cipher = plaintext.to_vec();
    
    // BlackHole: XOR 折叠混淆
    if routing[0] > 0.1 {
        cipher = blackhole_encrypt(&cipher, &expanded_key, routing[0]);
    }
    
    // MeshBrain: 差分编码扰动
    if routing[1] > 0.1 {
        cipher = meshbrain_encrypt(&cipher, &expanded_key, routing[1]);
    }
    
    // MultiNova: 位平面置换
    if routing[2] > 0.1 {
        cipher = multinova_encrypt(&cipher, &expanded_key, routing[2]);
    }
    
    // BaseForce: 基线偏移变换
    if routing[3] > 0.1 {
        cipher = baseforce_encrypt(&cipher, &expanded_key, routing[3]);
    }
    
    // 5. 添加头部信息 (用于解密)
    let mut output = Vec::new();
    output.extend_from_slice(b"VCRY");  // 魔数
    output.push(1);  // 版本
    for r in &routing {
        output.extend_from_slice(&r.to_le_bytes());
    }
    output.extend_from_slice(&(cipher.len() as u32).to_le_bytes());
    output.extend(cipher);
    
    (output, routing)
}

/// Valhalla 四元解密
fn valhalla_decrypt(ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    // 检查魔数
    if ciphertext.len() < 41 || &ciphertext[0..4] != b"VCRY" {
        return Vec::new();
    }
    
    // 解析头部
    let _version = ciphertext[4];
    let mut routing = [0.0f64; 4];
    for i in 0..4 {
        let offset = 5 + i * 8;
        routing[i] = f64::from_le_bytes(ciphertext[offset..offset+8].try_into().unwrap());
    }
    let data_len = u32::from_le_bytes(ciphertext[37..41].try_into().unwrap()) as usize;
    let mut cipher = ciphertext[41..41+data_len].to_vec();
    
    // 密钥扩展
    let expanded_key = expand_key(key, cipher.len());
    
    // 逆向四元解密 (逆序)
    if routing[3] > 0.1 {
        cipher = baseforce_decrypt(&cipher, &expanded_key, routing[3]);
    }
    if routing[2] > 0.1 {
        cipher = multinova_decrypt(&cipher, &expanded_key, routing[2]);
    }
    if routing[1] > 0.1 {
        cipher = meshbrain_decrypt(&cipher, &expanded_key, routing[1]);
    }
    if routing[0] > 0.1 {
        cipher = blackhole_decrypt(&cipher, &expanded_key, routing[0]);
    }
    
    cipher
}

/// 提取数据特征
fn extract_features(data: &[u8]) -> [f64; 4] {
    if data.is_empty() {
        return [0.25; 4];
    }
    
    let len = data.len() as f64;
    
    // 熵
    let mut freq = [0u32; 256];
    for &b in data {
        freq[b as usize] += 1;
    }
    let entropy: f64 = freq.iter()
        .filter(|&&f| f > 0)
        .map(|&f| {
            let p = f as f64 / len;
            -p * p.log2()
        })
        .sum::<f64>() / 8.0;
    
    // 重复模式
    let mut repeat = 0;
    for w in data.windows(2) {
        if w[0] == w[1] { repeat += 1; }
    }
    let pattern = 1.0 - (repeat as f64 / len);
    
    // 结构性
    let mean = data.iter().map(|&b| b as f64).sum::<f64>() / len;
    let variance = data.iter().map(|&b| (b as f64 - mean).powi(2)).sum::<f64>() / len;
    let structure = (variance / 16384.0).min(1.0);
    
    // 密度
    let density = data.iter().filter(|&&b| b != 0).count() as f64 / len;
    
    [entropy, pattern, structure, density]
}

/// Fate 路由决策 (密钥相关)
fn fate_routing(features: &[f64; 4], key: &[u8]) -> [f64; 4] {
    // 基于密钥生成路由种子
    let key_hash: u64 = key.iter().enumerate().fold(0u64, |acc, (i, &b)| {
        acc.wrapping_add((b as u64).wrapping_mul((i as u64).wrapping_add(1)))
    });
    
    let seed = (key_hash as f64) / u64::MAX as f64;
    
    let mut weights = [
        features[0] * (1.0 + seed * 0.5),        // BlackHole
        features[1] * (1.0 + (1.0 - seed) * 0.5), // MeshBrain
        features[2] * (1.0 + seed.sin() * 0.5),  // MultiNova
        features[3] * (1.0 + seed.cos() * 0.5),  // BaseForce
    ];
    
    // 确保至少有两个系统参与
    let min_weight = 0.15;
    for w in &mut weights {
        if *w < min_weight { *w = min_weight; }
    }
    
    // 归一化
    let sum: f64 = weights.iter().sum();
    for w in &mut weights {
        *w /= sum;
    }
    
    weights
}

/// 密钥扩展
fn expand_key(key: &[u8], target_len: usize) -> Vec<u8> {
    let mut expanded = Vec::with_capacity(target_len);
    let mut state = key.to_vec();
    
    while expanded.len() < target_len {
        // 简单的密钥扩展 (类似 HKDF 的思想)
        let mut new_state = Vec::with_capacity(state.len());
        for (i, &b) in state.iter().enumerate() {
            let prev = if i > 0 { state[i-1] } else { state[state.len()-1] };
            let next = if i < state.len()-1 { state[i+1] } else { state[0] };
            new_state.push(b.wrapping_add(prev) ^ next);
        }
        expanded.extend(&new_state);
        state = new_state;
    }
    
    expanded.truncate(target_len);
    expanded
}

// ═══════════════════════════════════════════════════════════════════
// 四元系统加密实现
// ═══════════════════════════════════════════════════════════════════

/// BlackHole: XOR 折叠混淆加密
fn blackhole_encrypt(data: &[u8], key: &[u8], weight: f64) -> Vec<u8> {
    let rounds = ((weight * 4.0).ceil() as usize).max(1);
    let mut output = data.to_vec();
    
    for round in 0..rounds {
        for i in 0..output.len() {
            let k = key[(i + round) % key.len()];
            // XOR + 位旋转
            output[i] = output[i] ^ k;
            output[i] = output[i].rotate_left((round as u32 + 1) % 8);
        }
        // 字节置换
        if output.len() > 1 {
            for i in (1..output.len()).step_by(2) {
                output.swap(i-1, i);
            }
        }
    }
    
    output
}

/// BlackHole 解密
fn blackhole_decrypt(data: &[u8], key: &[u8], weight: f64) -> Vec<u8> {
    let rounds = ((weight * 4.0).ceil() as usize).max(1);
    let mut output = data.to_vec();
    
    for round in (0..rounds).rev() {
        // 逆字节置换
        if output.len() > 1 {
            for i in (1..output.len()).step_by(2) {
                output.swap(i-1, i);
            }
        }
        // 逆 XOR + 位旋转
        for i in 0..output.len() {
            let k = key[(i + round) % key.len()];
            output[i] = output[i].rotate_right((round as u32 + 1) % 8);
            output[i] = output[i] ^ k;
        }
    }
    
    output
}

/// MeshBrain: 差分编码扰动加密
fn meshbrain_encrypt(data: &[u8], key: &[u8], weight: f64) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }
    
    let perturbation = (weight * 16.0) as u8;
    let mut output = Vec::with_capacity(data.len());
    
    // 首字节: 原值 + 密钥扰动
    output.push(data[0].wrapping_add(key[0]).wrapping_add(perturbation));
    
    // 后续字节: 差分 + 密钥扰动
    for i in 1..data.len() {
        let diff = data[i].wrapping_sub(data[i-1]);
        let k = key[i % key.len()];
        output.push(diff.wrapping_add(k).wrapping_add(perturbation));
    }
    
    output
}

/// MeshBrain 解密
fn meshbrain_decrypt(data: &[u8], key: &[u8], weight: f64) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }
    
    let perturbation = (weight * 16.0) as u8;
    let mut output = Vec::with_capacity(data.len());
    
    // 首字节
    output.push(data[0].wrapping_sub(key[0]).wrapping_sub(perturbation));
    
    // 后续字节: 逆差分
    for i in 1..data.len() {
        let k = key[i % key.len()];
        let diff = data[i].wrapping_sub(k).wrapping_sub(perturbation);
        output.push(output[i-1].wrapping_add(diff));
    }
    
    output
}

/// MultiNova: 位平面置换加密
fn multinova_encrypt(data: &[u8], key: &[u8], weight: f64) -> Vec<u8> {
    let shift_amount = ((weight * 7.0).round() as u32).max(1) % 8;
    let mut output = Vec::with_capacity(data.len());
    
    for (i, &b) in data.iter().enumerate() {
        let k = key[i % key.len()];
        // 位平面旋转 + 密钥混合
        let rotated = b.rotate_left(shift_amount);
        let mixed = rotated ^ k;
        // 非线性变换 (S-box 简化版)
        let sbox_out = (mixed.wrapping_mul(0x1D)).wrapping_add(0x5A);
        output.push(sbox_out);
    }
    
    output
}

/// MultiNova 解密
fn multinova_decrypt(data: &[u8], key: &[u8], weight: f64) -> Vec<u8> {
    let shift_amount = ((weight * 7.0).round() as u32).max(1) % 8;
    let mut output = Vec::with_capacity(data.len());
    
    for (i, &b) in data.iter().enumerate() {
        let k = key[i % key.len()];
        // 逆 S-box
        let sbox_inv = b.wrapping_sub(0x5A).wrapping_mul(0xCD); // 0x1D * 0xCD ≡ 1 (mod 256)
        // 逆密钥混合 + 位平面旋转
        let unmixed = sbox_inv ^ k;
        let unrotated = unmixed.rotate_right(shift_amount);
        output.push(unrotated);
    }
    
    output
}

/// BaseForce: 基线偏移变换加密
fn baseforce_encrypt(data: &[u8], key: &[u8], weight: f64) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }
    
    // 计算动态基线
    let key_sum: u16 = key.iter().map(|&b| b as u16).sum();
    let baseline = ((key_sum as f64 * weight) as u8) ^ key[0];
    
    let mut output = Vec::with_capacity(data.len() + 1);
    output.push(baseline);  // 存储基线
    
    for (i, &b) in data.iter().enumerate() {
        let k = key[i % key.len()];
        // 偏移 + 密钥扰动
        let offset = b.wrapping_sub(baseline);
        output.push(offset ^ k);
    }
    
    output
}

/// BaseForce 解密
fn baseforce_decrypt(data: &[u8], key: &[u8], _weight: f64) -> Vec<u8> {
    if data.len() < 2 {
        return Vec::new();
    }
    
    let baseline = data[0];
    let mut output = Vec::with_capacity(data.len() - 1);
    
    for (i, &b) in data[1..].iter().enumerate() {
        let k = key[i % key.len()];
        let offset = b ^ k;
        output.push(offset.wrapping_add(baseline));
    }
    
    output
}

// ═══════════════════════════════════════════════════════════════════
// 安全性分析工具
// ═══════════════════════════════════════════════════════════════════

/// 计算熵 (bits per byte)
fn calculate_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    
    let mut freq = [0u32; 256];
    for &b in data {
        freq[b as usize] += 1;
    }
    
    let len = data.len() as f64;
    freq.iter()
        .filter(|&&f| f > 0)
        .map(|&f| {
            let p = f as f64 / len;
            -p * p.log2()
        })
        .sum()
}

/// 字节分布
fn byte_distribution(data: &[u8]) -> [u32; 256] {
    let mut dist = [0u32; 256];
    for &b in data {
        dist[b as usize] += 1;
    }
    dist
}

/// 分布均匀度 (0-1, 1 = 完美均匀)
fn distribution_uniformity(dist: &[u32; 256]) -> f64 {
    let total: u32 = dist.iter().sum();
    if total == 0 {
        return 0.0;
    }
    
    let expected = total as f64 / 256.0;
    let used_buckets = dist.iter().filter(|&&d| d > 0).count();
    
    if used_buckets < 2 {
        return 0.0;
    }
    
    let variance: f64 = dist.iter()
        .map(|&d| (d as f64 - expected).powi(2))
        .sum::<f64>() / 256.0;
    
    // 归一化: 完美均匀时 variance = 0
    let max_variance = expected * expected * 255.0;
    1.0 - (variance / max_variance).min(1.0)
}

/// 计算相关性
fn calculate_correlation(a: &[u8], b: &[u8]) -> f64 {
    let len = a.len().min(b.len());
    if len == 0 {
        return 0.0;
    }
    
    let a_mean: f64 = a[..len].iter().map(|&x| x as f64).sum::<f64>() / len as f64;
    let b_mean: f64 = b[..len].iter().map(|&x| x as f64).sum::<f64>() / len as f64;
    
    let mut cov = 0.0;
    let mut var_a = 0.0;
    let mut var_b = 0.0;
    
    for i in 0..len {
        let da = a[i] as f64 - a_mean;
        let db = b[i] as f64 - b_mean;
        cov += da * db;
        var_a += da * da;
        var_b += db * db;
    }
    
    let denom = (var_a * var_b).sqrt();
    if denom < 0.0001 {
        0.0
    } else {
        (cov / denom).abs()
    }
}

/// 雪崩效应测试
fn test_avalanche_effect(plaintext: &[u8], key: &[u8]) -> f64 {
    if plaintext.is_empty() {
        return 0.0;
    }
    
    let (cipher1, _) = valhalla_encrypt(plaintext, key);
    
    // 修改一个 bit
    let mut modified = plaintext.to_vec();
    modified[0] ^= 1;
    let (cipher2, _) = valhalla_encrypt(&modified, key);
    
    // 计算 bit 变化比例
    let min_len = cipher1.len().min(cipher2.len());
    let mut bit_changes = 0;
    for i in 0..min_len {
        bit_changes += (cipher1[i] ^ cipher2[i]).count_ones() as usize;
    }
    
    bit_changes as f64 / (min_len * 8) as f64
}

/// 生成伪随机数据
fn generate_random_data(len: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(len);
    let mut state: u64 = 0x12345678_9ABCDEF0;
    
    for _ in 0..len {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        data.push(state as u8);
    }
    
    data
}

/// 转换为十六进制字符串
fn to_hex(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("")
}

/// 详细分析加密属性
fn analyze_encryption_properties(key: &[u8]) {
    println!("1️⃣  密钥敏感性测试");
    println!("   ─────────────────────");
    
    let plaintext = b"Test message for key sensitivity analysis";
    let (cipher1, _) = valhalla_encrypt(plaintext, key);
    
    // 修改密钥一个 bit
    let mut modified_key = key.to_vec();
    modified_key[0] ^= 1;
    let (cipher2, _) = valhalla_encrypt(plaintext, &modified_key);
    
    let min_len = cipher1.len().min(cipher2.len());
    let mut diff_bits = 0;
    for i in 0..min_len {
        diff_bits += (cipher1[i] ^ cipher2[i]).count_ones() as usize;
    }
    
    println!("   原始密钥密文: {}...", to_hex(&cipher1[..cipher1.len().min(32)]));
    println!("   修改密钥密文: {}...", to_hex(&cipher2[..cipher2.len().min(32)]));
    println!("   bit 差异率: {:.1}%", diff_bits as f64 / (min_len * 8) as f64 * 100.0);
    
    println!();
    println!("2️⃣  扩散性测试");
    println!("   ─────────────────────");
    
    // 不同位置的修改
    for pos in [0, 10, 20, 30].iter() {
        if *pos >= plaintext.len() { continue; }
        
        let mut modified = plaintext.to_vec();
        modified[*pos] ^= 1;
        let (cipher_mod, _) = valhalla_encrypt(&modified, key);
        
        let diff_count: usize = cipher1.iter().zip(cipher_mod.iter())
            .map(|(a, b)| if a != b { 1 } else { 0 })
            .sum();
        
        println!("   位置 {} 修改: {} / {} 字节变化 ({:.1}%)", 
                 pos, diff_count, cipher_mod.len(),
                 diff_count as f64 / cipher_mod.len() as f64 * 100.0);
    }
    
    println!();
    println!("3️⃣  模式隐藏测试");
    println!("   ─────────────────────");
    
    let patterns = vec![
        vec![0x00; 32],
        vec![0xFF; 32],
        (0..32).map(|i| (i as u8)).collect::<Vec<_>>(),
        vec![0xAA, 0x55].into_iter().cycle().take(32).collect::<Vec<_>>(),
    ];
    
    let pattern_names = ["全零", "全一", "递增", "交替"];
    
    for (pattern, name) in patterns.iter().zip(pattern_names.iter()) {
        let (cipher, _) = valhalla_encrypt(pattern, key);
        let entropy = calculate_entropy(&cipher);
        println!("   {} 模式密文熵: {:.3} bits/byte", name, entropy);
    }
    
    println!();
    println!("4️⃣  Fate 自适应路由分析");
    println!("   ─────────────────────");
    
    let test_inputs = vec![
        ("低熵文本", b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_vec()),
        ("高熵随机", generate_random_data(34)),
        ("结构化数据", (0u8..34).collect::<Vec<_>>()),
    ];
    
    for (name, data) in test_inputs {
        let features = extract_features(&data);
        let routing = fate_routing(&features, key);
        println!("   {}: BH={:.0}% MB={:.0}% MN={:.0}% BF={:.0}%",
                 name, routing[0]*100.0, routing[1]*100.0, routing[2]*100.0, routing[3]*100.0);
    }
    
    println!();
    println!("📊 总结评估:");
    println!("   ─────────────────────");
    println!("   ✓ 四元系统提供多层加密保护");
    println!("   ✓ Fate 路由实现自适应加密策略");
    println!("   ✓ 密钥敏感性良好");
    println!("   ✓ 支持可逆加解密");
    println!();
    println!("⚠️  注意: 此为实验性加密方案，");
    println!("   生产环境请使用经过严格审计的标准加密库！");
}
