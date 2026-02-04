//! # Quantum Mode: 量子模拟加密
//!
//! 基于原始 MeshBrain 物理模型的加密算法。
//!
//! ## 原始 MeshBrain 参数 (来自 systems/metasystems/meshbrain)
//! - base_membrane_thickness: 10.0
//! - network_density: 0.7
//! - fluctuation_intensity: 0.3
//! - membrane_thickness = base × (1 + intensity × 0.1)
//! - intelligence_level = intensity × fluctuation_intensity
//! 
//! ## 信号类型
//! - ContinuousFluctuationMode: intensity <= 0.5
//! - StructuralReorganizationMode: intensity > 0.5

use rayon::prelude::*;

// ============================================================================
// 原始 MeshBrain 配置 (完全复制自原项目)
// ============================================================================

/// 膜脑配置 - 来自 systems/metasystems/meshbrain
pub const BASE_MEMBRANE_THICKNESS: f64 = 10.0;
pub const NETWORK_DENSITY: f64 = 0.7;
pub const FLUCTUATION_INTENSITY: f64 = 0.3;
pub const CORE_COUNT: usize = 1000;

/// 信号类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignalType {
    /// 连续涨落模式 (intensity <= 0.5)
    ContinuousFluctuationMode,
    /// 结构重组模式 (intensity > 0.5)
    StructuralReorganizationMode,
}

impl SignalType {
    #[inline]
    pub fn from_intensity(intensity: f64) -> Self {
        if intensity > 0.5 {
            Self::StructuralReorganizationMode
        } else {
            Self::ContinuousFluctuationMode
        }
    }
}

// ============================================================================
// 量子加密核心
// ============================================================================

/// 使用 MeshBrain 物理模型加密
pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    if plaintext.is_empty() || key.is_empty() {
        return plaintext.to_vec();
    }

    let len = plaintext.len();
    let key_len = key.len();
    
    // 从密钥派生 MeshBrain 参数
    let key_sum: u64 = key.iter().map(|&b| b as u64).sum();
    let membrane_base = (key[0] as f64 / 255.0) * BASE_MEMBRANE_THICKNESS + 1.0;
    let density = NETWORK_DENSITY + (key_sum % 100) as f64 / 1000.0; // 0.7 - 0.8
    let fluctuation = FLUCTUATION_INTENSITY + (key_sum % 50) as f64 / 500.0; // 0.3 - 0.4

    // Phase 1: MeshBrain 变换 (并行)
    let mut output: Vec<u8> = (0..len)
        .into_par_iter()
        .map(|i| {
            let byte = plaintext[i];
            let k = key[i % key_len];
            
            // 计算位置 (3D 核心位置模拟)
            let pos_x = (i % 16) as f64 / 16.0;
            let pos_y = ((i / 16) % 16) as f64 / 16.0;
            let pos_z = (i / 256) as f64 / 256.0;
            let position_factor = ((pos_x + pos_y + pos_z) * 255.0) as u8;
            
            // 使用确定性熵决定信号类型 (保证可逆)
            let entropy = derive_entropy(key, i);
            let intensity = entropy as f64 / 255.0; // 用熵代替字节值
            
            // 原始公式: membrane_thickness = base × (1 + intensity × 0.1)
            let membrane_thickness = membrane_base * (1.0 + intensity * 0.1);
            let membrane_factor = ((membrane_thickness * 10.0) as u8).wrapping_mul(17);
            
            // 原始公式: intelligence_level = intensity × fluctuation_intensity
            let intelligence = intensity * fluctuation;
            let intelligence_factor = ((intelligence * 255.0) as u8).wrapping_mul(23);
            
            // 网络连接因子
            let connectivity_factor = ((density * 255.0) as u8).wrapping_add(k);
            
            // 确定信号类型 (基于熵，与解密一致)
            let signal_type = SignalType::from_intensity(intensity);
            
            // 根据信号类型选择变换
            let state = byte
                .wrapping_add(membrane_factor)
                .wrapping_add(position_factor);
            
            let activated = match signal_type {
                SignalType::StructuralReorganizationMode => {
                    // 结构重组: 强非线性变换
                    let pk_odd = (k.wrapping_add(intelligence_factor)) | 1;
                    state.wrapping_mul(pk_odd)
                        .rotate_left(((membrane_thickness as u32) % 5) + 1)
                        .wrapping_add(connectivity_factor)
                }
                SignalType::ContinuousFluctuationMode => {
                    // 连续涨落: 加性变换
                    state.wrapping_add(k)
                        .wrapping_add(intelligence_factor)
                        .rotate_right(((density * 7.0) as u32) + 1)
                }
            };
            
            // 最终混合
            activated ^ k ^ membrane_factor ^ intelligence_factor
        })
        .collect();

    // Phase 2: 级联传播 (顺序，模拟神经网络状态传递)
    let cascade_init = key[0]
        .wrapping_add(((membrane_base * 10.0) as u8).wrapping_mul(17))
        .wrapping_add(((density * 255.0) as u8));
    
    let mut cascade = cascade_init;
    for i in 0..len {
        let t = output[i];
        output[i] = t ^ cascade;
        cascade = cascade.wrapping_add(t).rotate_left(1);
    }

    output
}

/// 使用 MeshBrain 物理模型解密
pub fn decrypt(ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    if ciphertext.is_empty() || key.is_empty() {
        return ciphertext.to_vec();
    }

    let len = ciphertext.len();
    let key_len = key.len();
    
    // 从密钥派生 MeshBrain 参数 (必须与加密完全一致)
    let key_sum: u64 = key.iter().map(|&b| b as u64).sum();
    let membrane_base = (key[0] as f64 / 255.0) * BASE_MEMBRANE_THICKNESS + 1.0;
    let density = NETWORK_DENSITY + (key_sum % 100) as f64 / 1000.0;
    let fluctuation = FLUCTUATION_INTENSITY + (key_sum % 50) as f64 / 500.0;

    // Phase 2 逆: 级联解除
    let cascade_init = key[0]
        .wrapping_add(((membrane_base * 10.0) as u8).wrapping_mul(17))
        .wrapping_add(((density * 255.0) as u8));
    
    let mut phase1_output = ciphertext.to_vec();
    let mut cascade = cascade_init;
    
    for i in 0..len {
        let c = phase1_output[i];
        let t = c ^ cascade;
        phase1_output[i] = t;
        cascade = cascade.wrapping_add(t).rotate_left(1);
    }

    // Phase 1 逆: MeshBrain 逆变换 (并行)
    let plaintext: Vec<u8> = (0..len)
        .into_par_iter()
        .map(|i| {
            let t = phase1_output[i];
            let k = key[i % key_len];
            
            // 重新计算所有因子 (必须与加密一致)
            let pos_x = (i % 16) as f64 / 16.0;
            let pos_y = ((i / 16) % 16) as f64 / 16.0;
            let pos_z = (i / 256) as f64 / 256.0;
            let position_factor = ((pos_x + pos_y + pos_z) * 255.0) as u8;
            
            // 使用与加密相同的熵决定信号类型
            let entropy = derive_entropy(key, i);
            let intensity = entropy as f64 / 255.0;
            
            // 重新计算所有因子 (必须与加密完全一致)
            let membrane_thickness = membrane_base * (1.0 + intensity * 0.1);
            let membrane_factor = ((membrane_thickness * 10.0) as u8).wrapping_mul(17);
            let intelligence = intensity * fluctuation;
            let intelligence_factor = ((intelligence * 255.0) as u8).wrapping_mul(23);
            let connectivity_factor = ((density * 255.0) as u8).wrapping_add(k);
            let signal_type = SignalType::from_intensity(intensity);
            
            // 逆变换
            let activated = t ^ k ^ membrane_factor ^ intelligence_factor;
            
            let state = match signal_type {
                SignalType::StructuralReorganizationMode => {
                    let pk_odd = (k.wrapping_add(intelligence_factor)) | 1;
                    let pk_inv = mod_inverse(pk_odd);
                    
                    activated
                        .wrapping_sub(connectivity_factor)
                        .rotate_right(((membrane_thickness as u32) % 5) + 1)
                        .wrapping_mul(pk_inv)
                }
                SignalType::ContinuousFluctuationMode => {
                    activated
                        .rotate_left(((density * 7.0) as u32) + 1)
                        .wrapping_sub(intelligence_factor)
                        .wrapping_sub(k)
                }
            };
            
            state
                .wrapping_sub(position_factor)
                .wrapping_sub(membrane_factor)
        })
        .collect();

    plaintext
}

/// 确定性熵派生
fn derive_entropy(key: &[u8], position: usize) -> u8 {
    let key_hash: u64 = key.iter().enumerate()
        .fold(0u64, |acc, (i, &b)| acc.wrapping_add((b as u64).wrapping_mul((i + 1) as u64)));
    let pos_hash = (position as u64).wrapping_mul(0x9E3779B9);
    ((key_hash ^ pos_hash) % 256) as u8
}

/// 模 256 乘法逆元 (Newton-Raphson)
fn mod_inverse(a: u8) -> u8 {
    let mut x = a;
    x = x.wrapping_mul(2u8.wrapping_sub(a.wrapping_mul(x)));
    x = x.wrapping_mul(2u8.wrapping_sub(a.wrapping_mul(x)));
    x = x.wrapping_mul(2u8.wrapping_sub(a.wrapping_mul(x)));
    x = x.wrapping_mul(2u8.wrapping_sub(a.wrapping_mul(x)));
    x
}

// ============================================================================
// 辅助函数：获取 MeshBrain 状态
// ============================================================================

/// MeshBrain 加密状态
#[derive(Debug, Clone)]
pub struct MeshBrainState {
    pub membrane_base: f64,
    pub network_density: f64,
    pub fluctuation_intensity: f64,
}

impl MeshBrainState {
    pub fn from_key(key: &[u8]) -> Self {
        let key_sum: u64 = key.iter().map(|&b| b as u64).sum();
        Self {
            membrane_base: (key[0] as f64 / 255.0) * BASE_MEMBRANE_THICKNESS + 1.0,
            network_density: NETWORK_DENSITY + (key_sum % 100) as f64 / 1000.0,
            fluctuation_intensity: FLUCTUATION_INTENSITY + (key_sum % 50) as f64 / 500.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_type() {
        assert_eq!(SignalType::from_intensity(0.3), SignalType::ContinuousFluctuationMode);
        assert_eq!(SignalType::from_intensity(0.7), SignalType::StructuralReorganizationMode);
        assert_eq!(SignalType::from_intensity(0.5), SignalType::ContinuousFluctuationMode);
        assert_eq!(SignalType::from_intensity(0.51), SignalType::StructuralReorganizationMode);
    }

    #[test]
    fn test_meshbrain_state() {
        let key = b"TestKey12345678!";
        let state = MeshBrainState::from_key(key);
        
        assert!(state.membrane_base >= 1.0 && state.membrane_base <= 11.0);
        assert!(state.network_density >= 0.7 && state.network_density <= 0.8);
        assert!(state.fluctuation_intensity >= 0.3 && state.fluctuation_intensity <= 0.4);
    }

    #[test]
    fn test_quantum_avalanche() {
        let data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
        let mut data_mod = data.clone();
        data_mod[0] ^= 0x01;
        
        let key = b"TestKey12345678!";
        let c1 = encrypt(&data, key);
        let c2 = encrypt(&data_mod, key);
        
        let diff: u32 = c1.iter().zip(c2.iter())
            .map(|(&a, &b)| (a ^ b).count_ones())
            .sum();
        let avalanche = diff as f64 / (c1.len() * 8) as f64;
        
        // 雪崩效应应该接近 50%
        println!("Quantum avalanche: {:.2}%", avalanche * 100.0);
        assert!(avalanche > 0.40 && avalanche < 0.60);
    }
}
