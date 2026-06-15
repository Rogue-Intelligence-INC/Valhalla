//! # Geometric Mode: 信息几何加密
//!
//! 基于 ADGCF (Adaptive Degree of Freedom Geometry Computation Framework) 的加密算法。
//!
//! ## 核心概念
//! - **Fisher Information Matrix**: 定义概率空间的度量
//! - **Cartan Connection**: 流形上的几何连接
//! - **测地线变换**: 沿测地线进行加密变换
//! - **曲率调制**: 使用曲率参数增加复杂度
//!
//! ## 密钥空间
//! 传统加密: key → parameters (参数由密钥派生，密钥空间不变)
//! 几何加密: key → manifold point → geometric transform (真正的几何空间扩展)

use rayon::prelude::*;

// ============================================================================
// 信息几何核心结构
// ============================================================================

/// Fisher 信息矩阵 (简化为对角形式用于加密)
#[derive(Debug, Clone)]
pub struct FisherMetric {
    /// 对角元素 (1/p_i - 1)
    pub diagonal: Vec<f64>,
    /// 维度
    pub dimension: usize,
}

impl FisherMetric {
    /// 从概率分布计算 Fisher 度量
    pub fn from_distribution(distribution: &[f64]) -> Self {
        let diagonal: Vec<f64> = distribution.iter()
            .map(|&p| {
                let p_safe = p.max(1e-10);
                1.0 / p_safe - 1.0
            })
            .collect();
        Self {
            dimension: distribution.len(),
            diagonal,
        }
    }
    
    /// 从密钥派生概率分布，再计算 Fisher 度量
    pub fn from_key(key: &[u8], dimension: usize) -> Self {
        // 将密钥转换为概率分布
        let distribution = key_to_distribution(key, dimension);
        Self::from_distribution(&distribution)
    }
    
    /// 计算测地线距离 (简化版)
    pub fn geodesic_distance(&self, p1: &[f64], p2: &[f64]) -> f64 {
        let mut sum = 0.0;
        for i in 0..self.dimension.min(p1.len()).min(p2.len()) {
            let diff = p2[i] - p1[i];
            sum += self.diagonal[i] * diff * diff;
        }
        sum.sqrt()
    }
}

/// Cartan 连接 (简化版)
#[derive(Debug, Clone)]
pub struct CartanConnection {
    /// α 参数 (-1 到 1)
    pub alpha: f64,
    /// 连接系数 (简化为标量)
    pub connection_strength: f64,
}

impl CartanConnection {
    /// 从密钥派生 Cartan 连接
    pub fn from_key(key: &[u8]) -> Self {
        let key_sum: u64 = key.iter().map(|&b| b as u64).sum();
        
        // α 参数: -1 到 1
        let alpha = ((key_sum % 200) as f64 / 100.0) - 1.0;
        
        // 连接强度: 0.1 到 1.0
        let connection_strength = 0.1 + (key[0] as f64 / 255.0) * 0.9;
        
        Self { alpha, connection_strength }
    }
}

/// 几何加密空间
#[derive(Debug, Clone)]
pub struct GeometricSpace {
    /// Fisher 度量
    pub fisher: FisherMetric,
    /// Cartan 连接
    pub cartan: CartanConnection,
    /// 基点分布 (密钥派生)
    pub base_distribution: Vec<f64>,
    /// 曲率参数
    pub curvature: f64,
}

impl GeometricSpace {
    /// 从密钥构建几何空间
    pub fn from_key(key: &[u8]) -> Self {
        let dimension = 16; // 使用 16 维概率空间
        
        // 1. 密钥 → 概率分布 (基点)
        let base_distribution = key_to_distribution(key, dimension);
        
        // 2. 计算 Fisher 度量
        let fisher = FisherMetric::from_distribution(&base_distribution);
        
        // 3. 派生 Cartan 连接
        let cartan = CartanConnection::from_key(key);
        
        // 4. 计算曲率 (从 Fisher 度量的条件数)
        let max_diag = fisher.diagonal.iter().cloned().fold(f64::MIN, f64::max);
        let min_diag = fisher.diagonal.iter().cloned().fold(f64::MAX, f64::min);
        let curvature = if min_diag > 1e-10 {
            (max_diag / min_diag).ln().tanh()
        } else {
            0.5
        };
        
        Self {
            fisher,
            cartan,
            base_distribution,
            curvature,
        }
    }
    
    /// 沿测地线变换 (加密核心)
    #[inline]
    pub fn geodesic_transform(&self, value: u8, position: usize) -> u8 {
        // 位置决定在流形上的方向
        let direction_idx = position % self.fisher.dimension;
        let metric_value = self.fisher.diagonal[direction_idx];
        
        // α-连接调制
        let alpha_factor = ((1.0 + self.cartan.alpha) / 2.0 * 255.0) as u8;
        
        // 曲率调制
        let curvature_factor = ((self.curvature * 127.0) as u8).wrapping_add(1);
        
        // 连接强度
        let conn_factor = ((self.cartan.connection_strength * 255.0) as u8) | 1;
        
        // 度量因子
        let metric_factor = ((metric_value.tanh() * 127.0 + 128.0) as u8) | 1;
        
        // 组合变换 (确保可逆)
        let step1 = value.wrapping_add(alpha_factor);
        let step2 = step1.wrapping_mul(metric_factor); // 乘以奇数，可逆
        let step3 = step2.rotate_left((curvature_factor % 7 + 1) as u32);
        let step4 = step3.wrapping_add(conn_factor);
        
        step4
    }
    
    /// 逆测地线变换 (解密核心)
    #[inline]
    pub fn inverse_geodesic_transform(&self, value: u8, position: usize) -> u8 {
        let direction_idx = position % self.fisher.dimension;
        let metric_value = self.fisher.diagonal[direction_idx];
        
        let alpha_factor = ((1.0 + self.cartan.alpha) / 2.0 * 255.0) as u8;
        let curvature_factor = ((self.curvature * 127.0) as u8).wrapping_add(1);
        let conn_factor = ((self.cartan.connection_strength * 255.0) as u8) | 1;
        let metric_factor = ((metric_value.tanh() * 127.0 + 128.0) as u8) | 1;
        
        // 逆变换
        let step1 = value.wrapping_sub(conn_factor);
        let step2 = step1.rotate_right((curvature_factor % 7 + 1) as u32);
        let step3 = step2.wrapping_mul(mod_inverse(metric_factor));
        let step4 = step3.wrapping_sub(alpha_factor);
        
        step4
    }
}

// ============================================================================
// 加密/解密函数
// ============================================================================

/// 使用几何空间加密
pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    if plaintext.is_empty() || key.is_empty() {
        return plaintext.to_vec();
    }
    
    let len = plaintext.len();
    let space = GeometricSpace::from_key(key);
    
    // Phase 1: 几何变换 (并行)
    let mut output: Vec<u8> = (0..len)
        .into_par_iter()
        .map(|i| {
            let byte = plaintext[i];
            let k = key[i % key.len()];
            
            // 几何变换
            let geo_transformed = space.geodesic_transform(byte, i);
            
            // 位置密钥混合
            let pos_factor = ((i.wrapping_mul(17)) & 0xFF) as u8;
            let mixed = geo_transformed
                .wrapping_add(k)
                .wrapping_add(pos_factor);
            
            mixed ^ k
        })
        .collect();
    
    // Phase 2: 几何级联 (顺序)
    // 使用曲率和 α 参数初始化级联
    let cascade_init = key[0]
        .wrapping_add(((space.curvature * 255.0) as u8))
        .wrapping_add(((space.cartan.alpha + 1.0) * 127.0) as u8);
    
    let mut cascade = cascade_init;
    for i in 0..len {
        let t = output[i];
        output[i] = t ^ cascade;
        cascade = cascade.wrapping_add(t).rotate_left(1);
    }
    
    output
}

/// 使用几何空间解密
pub fn decrypt(ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    if ciphertext.is_empty() || key.is_empty() {
        return ciphertext.to_vec();
    }
    
    let len = ciphertext.len();
    let space = GeometricSpace::from_key(key);
    
    // Phase 2 逆: 级联解除
    let cascade_init = key[0]
        .wrapping_add(((space.curvature * 255.0) as u8))
        .wrapping_add(((space.cartan.alpha + 1.0) * 127.0) as u8);
    
    let mut phase1_output = ciphertext.to_vec();
    let mut cascade = cascade_init;
    
    for i in 0..len {
        let c = phase1_output[i];
        let t = c ^ cascade;
        phase1_output[i] = t;
        cascade = cascade.wrapping_add(t).rotate_left(1);
    }
    
    // Phase 1 逆: 几何逆变换 (并行)
    let plaintext: Vec<u8> = (0..len)
        .into_par_iter()
        .map(|i| {
            let t = phase1_output[i];
            let k = key[i % key.len()];
            
            // 逆混合
            let pos_factor = ((i.wrapping_mul(17)) & 0xFF) as u8;
            let unmixed = (t ^ k)
                .wrapping_sub(pos_factor)
                .wrapping_sub(k);
            
            // 逆几何变换
            space.inverse_geodesic_transform(unmixed, i)
        })
        .collect();
    
    plaintext
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 将密钥转换为概率分布
fn key_to_distribution(key: &[u8], dimension: usize) -> Vec<f64> {
    let mut distribution = vec![0.0; dimension];
    
    // 使用密钥填充分布
    for (i, &byte) in key.iter().enumerate() {
        let idx = i % dimension;
        distribution[idx] += byte as f64 + 1.0; // +1 避免零概率
    }
    
    // 如果密钥短于维度，补充
    for i in key.len()..dimension {
        distribution[i] = (key[i % key.len()] as f64 + 1.0) * ((i + 1) as f64);
    }
    
    // 归一化
    let sum: f64 = distribution.iter().sum();
    for p in &mut distribution {
        *p /= sum;
    }
    
    distribution
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
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_to_distribution() {
        let key = b"TestKey12345678!";
        let dist = key_to_distribution(key, 16);
        
        // 检查归一化
        let sum: f64 = dist.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
        
        // 检查所有概率为正
        assert!(dist.iter().all(|&p| p > 0.0));
    }

    #[test]
    fn test_geometric_space_creation() {
        let key = b"TestKey12345678!";
        let space = GeometricSpace::from_key(key);
        
        assert_eq!(space.fisher.dimension, 16);
        assert!(space.cartan.alpha >= -1.0 && space.cartan.alpha <= 1.0);
        assert!(space.cartan.connection_strength >= 0.1 && space.cartan.connection_strength <= 1.0);
        assert!(space.curvature >= 0.0 && space.curvature <= 1.0);
    }

    #[test]
    fn test_geometric_reversible() {
        let data = b"Hello, Geometric Encryption!";
        let key = b"TestKey12345678!";
        
        let encrypted = encrypt(data, key);
        let decrypted = decrypt(&encrypted, key);
        
        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_geometric_avalanche() {
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
        
        println!("Geometric avalanche: {:.2}%", avalanche * 100.0);
        assert!(avalanche > 0.40 && avalanche < 0.60);
    }

    #[test]
    fn test_different_keys_different_spaces() {
        let key1 = b"Key1Key1Key1Key1";
        let key2 = b"Key2Key2Key2Key2";
        
        let space1 = GeometricSpace::from_key(key1);
        let space2 = GeometricSpace::from_key(key2);
        
        // 不同密钥产生不同的几何空间
        assert!((space1.curvature - space2.curvature).abs() > 0.001 ||
                (space1.cartan.alpha - space2.cartan.alpha).abs() > 0.001);
    }
}
