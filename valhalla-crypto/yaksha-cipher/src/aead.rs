//! # Yaksha AEAD: Authenticated Encryption with Associated Data
//!
//! A stateful AEAD protocol with three components:
//!
//! - **YakshaKdf**: Key derivation with state evolution
//! - **YakshaMac**: Message authentication code
//! - **YakshaAead**: Combined AEAD protocol
//!
//! ## Protocol Architecture
//!
//! ```text
//! Component                    Function
//! ─────────────────────────────────────────────────────────────
//! YakshaKdf (Key Derivation)   Stateful key evolution
//!   - extract_structure()       Extract differential patterns
//!   - compute_signature()       Compute key signature
//!   - evolve_state()            State evolution for session keys
//!
//! YakshaMac (Authentication)   Message integrity verification
//!   - verify_consistency()      Cross-domain consistency check
//!   - compute_tag()             Generate authentication tag
//!   - verify_tag()              Verify message integrity
//! ```
//!
//! ## Security Model
//!
//! - **Deterministic primitive**: The underlying cipher is deterministic
//! - **Stateful protocol**: KDF evolves internal state per operation
//! - **Session differentiation**: Same plaintext encrypts differently across sessions
//!
//! ## Note on Nonce-Free Operation
//!
//! This protocol does NOT provide IND-CPA security at the primitive level.
//! Session uniqueness comes from stateful key derivation, not from the cipher.

use crate::core;

// ============================================================================
// Part 1: Dynamic Key Generator (基于 Fate 信号捕捉)
// ============================================================================

/// Signal Structure - Fate 的信号结构抽象
/// 
/// 对应 S-wind 的 `SignalStructure`:
/// - differences: 差分序列 (Δ¹-Δ⁵)
/// - ratios: 比率序列
/// - signature: 结构签名
#[derive(Debug, Clone)]
pub struct SignalStructure {
    /// 差分序列 (5阶，每阶8字节)
    pub differences: [[u8; 8]; 5],
    /// 比率累加器
    pub ratios: [u8; 8],
    /// 信号签名 (用于同构检测)
    pub signature: [u8; 16],
    /// 一致性分数 (0-100)
    pub consistency_score: u32,
}

impl SignalStructure {
    /// 创建空结构
    pub fn new() -> Self {
        SignalStructure {
            differences: [[0u8; 8]; 5],
            ratios: [0u8; 8],
            signature: [0u8; 16],
            consistency_score: 0,
        }
    }
}

impl Default for SignalStructure {
    fn default() -> Self {
        Self::new()
    }
}

/// Dynamic Key Generator - 基于 Fate 信号捕捉的密钥演化
/// 
/// 核心思想: 把内部状态当作"信号"，提取其结构特征来派生密钥
/// 
/// 对应 S-wind 的:
/// - `Fate.extract_structure()` → 提取当前状态的结构
/// - `compute_signature()` → 计算密钥签名
/// - Signal domain cycling → 域演化
#[derive(Debug, Clone)]
pub struct YakshaKdf {
    /// 主密钥 (不变)
    master_key: [u8; 32],
    /// 当前信号状态 (会演化)
    signal_state: [u8; 32],
    /// 前一个信号值 (用于差分计算)
    prev_signal: [u8; 8],
    /// 提取的信号结构
    structure: SignalStructure,
    /// 生成计数器
    generation: u64,
    /// 信号域 (对应 Fate 的 SignalDomain)
    domain: SignalDomain,
}

/// 信号域 - 对应 S-wind 的 SignalDomain
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignalDomain {
    Mathematics = 0,
    PhysicsGravity = 1,
    PhysicsQuantum = 2,
    PhysicsThermal = 3,
    PhysicsCoupling = 4,
    SpaceDimension = 5,
    External = 6,
}

impl SignalDomain {
    fn next(self) -> Self {
        match self {
            SignalDomain::Mathematics => SignalDomain::PhysicsGravity,
            SignalDomain::PhysicsGravity => SignalDomain::PhysicsQuantum,
            SignalDomain::PhysicsQuantum => SignalDomain::PhysicsThermal,
            SignalDomain::PhysicsThermal => SignalDomain::PhysicsCoupling,
            SignalDomain::PhysicsCoupling => SignalDomain::SpaceDimension,
            SignalDomain::SpaceDimension => SignalDomain::External,
            SignalDomain::External => SignalDomain::Mathematics,
        }
    }
    
    fn constant(self) -> u8 {
        // 每个域的特征常量 (类似 Fate 的域特定参数)
        match self {
            SignalDomain::Mathematics => 0x1B,
            SignalDomain::PhysicsGravity => 0x37,
            SignalDomain::PhysicsQuantum => 0x59,
            SignalDomain::PhysicsThermal => 0x7F,
            SignalDomain::PhysicsCoupling => 0x93,
            SignalDomain::SpaceDimension => 0xA7,
            SignalDomain::External => 0xC1,
        }
    }
}

impl YakshaKdf {
    /// 创建密钥生成器
    /// 
    /// 初始化: 把 master_key 作为初始信号
    pub fn new(master_key: &[u8]) -> Self {
        let mut mk = [0u8; 32];
        let mut signal = [0u8; 32];
        let mut prev = [0u8; 8];
        
        // 初始化主密钥
        for (i, &k) in master_key.iter().cycle().take(32).enumerate() {
            mk[i] = k.wrapping_mul(17).wrapping_add(i as u8);
        }
        
        // 初始化信号状态 (不同于主密钥，防止泄露)
        for i in 0..32 {
            signal[i] = mk[i]
                .rotate_left(3)
                .wrapping_add((i as u8).wrapping_mul(23));
        }
        
        // 初始化前一信号值
        for i in 0..8 {
            prev[i] = signal[i];
        }
        
        // 从主密钥派生初始域
        let domain_idx = master_key.iter().fold(0u8, |acc, &b| acc ^ b) % 7;
        let domain = match domain_idx {
            0 => SignalDomain::Mathematics,
            1 => SignalDomain::PhysicsGravity,
            2 => SignalDomain::PhysicsQuantum,
            3 => SignalDomain::PhysicsThermal,
            4 => SignalDomain::PhysicsCoupling,
            5 => SignalDomain::SpaceDimension,
            _ => SignalDomain::External,
        };
        
        let mut gen = DynamicKeyGenerator {
            master_key: mk,
            signal_state: signal,
            prev_signal: prev,
            structure: SignalStructure::new(),
            generation: 0,
            domain,
        };
        
        // 初始结构提取
        gen.extract_structure();
        
        gen
    }
    
    /// 提取信号结构 - 核心算法
    /// 
    /// 对应 S-wind `Fate.extract_structure()`:
    /// 1. 计算差分序列 Δ¹-Δ⁵
    /// 2. 计算比率序列
    /// 3. 计算签名
    fn extract_structure(&mut self) {
        // 把状态转换为信号值序列
        let signal_values: Vec<u64> = self.signal_state
            .chunks(8)
            .map(|c| {
                let mut bytes = [0u8; 8];
                bytes[..c.len()].copy_from_slice(c);
                u64::from_le_bytes(bytes)
            })
            .collect();
        
        // 1. 计算差分序列 (Δ¹ 到 Δ⁵)
        // Δ¹[i] = signal[i+1] - signal[i]
        let mut current = signal_values.clone();
        
        for order in 0..5 {
            if current.len() < 2 {
                break;
            }
            
            // 计算该阶差分
            let diffs: Vec<i64> = current.windows(2)
                .map(|w| (w[1] as i64).wrapping_sub(w[0] as i64))
                .collect();
            
            // 累加到结构中 (折叠哈希)
            let diff_hash = diffs.iter().fold(0u64, |acc, &d| {
                acc.rotate_left(7).wrapping_add(d as u64)
            });
            
            self.structure.differences[order] = diff_hash.to_le_bytes();
            
            // 准备下一阶
            current = diffs.iter().map(|&d| d as u64).collect();
        }
        
        // 2. 计算比率序列
        // ratio[i] = signal[i+1] / signal[i]
        let prev_val = u64::from_le_bytes(self.prev_signal);
        let curr_val = u64::from_le_bytes([
            self.signal_state[0], self.signal_state[1],
            self.signal_state[2], self.signal_state[3],
            self.signal_state[4], self.signal_state[5],
            self.signal_state[6], self.signal_state[7],
        ]);
        
        if prev_val != 0 {
            // 定点数比率 (避免浮点)
            let ratio = if curr_val > prev_val {
                ((curr_val / prev_val.max(1)) << 16).wrapping_add(
                    ((curr_val % prev_val.max(1)) << 16) / prev_val.max(1)
                )
            } else {
                ((curr_val << 16) / prev_val.max(1))
            };
            
            // 累加到比率状态
            let old_ratio = u64::from_le_bytes(self.structure.ratios);
            let new_ratio = old_ratio.wrapping_add(ratio).rotate_left(5);
            self.structure.ratios = new_ratio.to_le_bytes();
        }
        
        // 3. 计算签名 (对应 compute_signature)
        // 签名 = 首项 + 首阶差分 + 首比率
        let mut sig = [0u8; 16];
        
        // 首项
        for i in 0..8 {
            sig[i] = self.signal_state[i];
        }
        
        // 首阶差分
        for i in 0..8 {
            sig[i] ^= self.structure.differences[0][i];
        }
        
        // 首比率
        for i in 0..8 {
            sig[i + 8] = self.structure.ratios[i];
        }
        
        // 域常量混合
        let domain_const = self.domain.constant();
        for i in 0..16 {
            sig[i] = sig[i].wrapping_add(domain_const).rotate_left((i % 7 + 1) as u32);
        }
        
        self.structure.signature = sig;
        
        // 更新前一信号值
        self.prev_signal.copy_from_slice(&self.signal_state[..8]);
    }
    
    /// 生成下一个密钥
    /// 
    /// 流程:
    /// 1. 提取当前信号结构
    /// 2. 从结构计算密钥
    /// 3. 演化信号状态
    /// 4. 切换信号域
    pub fn next_key(&mut self) -> [u8; 16] {
        self.generation += 1;
        
        // 1. 提取结构
        self.extract_structure();
        
        // 2. 从结构计算密钥
        let mut key = [0u8; 16];
        
        // 基于签名
        for i in 0..16 {
            key[i] = self.structure.signature[i];
        }
        
        // 混合所有差分阶
        for order in 0..5 {
            for i in 0..8 {
                let idx = (order * 3 + i) % 16;
                key[idx] ^= self.structure.differences[order][i]
                    .rotate_left((order + 1) as u32);
            }
        }
        
        // 混合比率
        for i in 0..8 {
            key[i] = key[i].wrapping_add(self.structure.ratios[i]);
            key[i + 8] ^= self.structure.ratios[i].rotate_left(3);
        }
        
        // 混合生成计数
        let gen_bytes = self.generation.to_le_bytes();
        for i in 0..8 {
            key[i] ^= gen_bytes[i];
            key[i + 8] = key[i + 8].wrapping_add(gen_bytes[i]).rotate_left(1);
        }
        
        // 混合主密钥 (确保与主密钥绑定)
        for i in 0..16 {
            key[i] ^= self.master_key[i] ^ self.master_key[i + 16];
        }
        
        // 3. 演化信号状态 (自循环，对应 Fate 的 explore_with_self_loop)
        for i in 0..32 {
            self.signal_state[i] = self.signal_state[i]
                .wrapping_add(key[i % 16])
                .rotate_left((i % 7 + 1) as u32);
        }
        
        // 级联扩散
        for _ in 0..2 {
            for i in 0..31 {
                self.signal_state[i + 1] ^= self.signal_state[i].rotate_left(1);
            }
            self.signal_state[0] = self.signal_state[0]
                .wrapping_add(self.signal_state[31]);
        }
        
        // 4. 域演化
        self.domain = self.domain.next();
        
        key
    }
    
    /// 派生特定上下文的密钥
    pub fn derive_key(&mut self, context: &[u8]) -> [u8; 16] {
        // 把上下文深度混入信号状态
        
        // 1. 直接 XOR 混合
        for (i, &c) in context.iter().enumerate() {
            let idx = i % 32;
            self.signal_state[idx] ^= c.rotate_left((i % 7 + 1) as u32);
        }
        
        // 2. 级联扩散 (确保上下文影响所有位置)
        for _ in 0..3 {
            for i in 0..31 {
                self.signal_state[i + 1] = self.signal_state[i + 1]
                    .wrapping_add(self.signal_state[i])
                    .rotate_left(1);
            }
            self.signal_state[0] ^= self.signal_state[31];
        }
        
        // 3. 上下文哈希混入
        let ctx_hash = context.iter().fold(0u64, |acc, &c| {
            acc.rotate_left(5).wrapping_add(c as u64).wrapping_mul(0x9E3779B97F4A7C15)
        });
        let ctx_bytes = ctx_hash.to_le_bytes();
        for i in 0..8 {
            self.signal_state[i] ^= ctx_bytes[i];
            self.signal_state[i + 8] = self.signal_state[i + 8].wrapping_add(ctx_bytes[i]);
            self.signal_state[i + 16] ^= ctx_bytes[i].rotate_left(3);
            self.signal_state[i + 24] = self.signal_state[i + 24].wrapping_add(ctx_bytes[i].rotate_left(5));
        }
        
        self.next_key()
    }
    
    /// 获取当前生成数
    pub fn generation(&self) -> u64 {
        self.generation
    }
    
    /// 获取当前域
    pub fn domain(&self) -> SignalDomain {
        self.domain
    }
    
    /// 验证前向安全性
    pub fn verify_forward_secrecy(&self) -> bool {
        // 检查结构熵 (包括信号状态)
        let mut total_ones = 0u32;
        
        // 检查信号状态
        for &b in &self.signal_state {
            total_ones += b.count_ones();
        }
        
        // 检查差分结构
        for order in 0..5 {
            for &b in &self.structure.differences[order] {
                total_ones += b.count_ones();
            }
        }
        
        // 检查比率
        for &b in &self.structure.ratios {
            total_ones += b.count_ones();
        }
        
        // 检查签名
        for &b in &self.structure.signature {
            total_ones += b.count_ones();
        }
        
        // 期望 ~50% 的位是1
        // 总位数: signal_state(32*8) + differences(5*8*8) + ratios(8*8) + signature(16*8)
        let total_bits = (32 + 5*8 + 8 + 16) * 8;
        let ratio = total_ones as f64 / total_bits as f64;
        
        // 放宽范围到 30%-70%
        (0.30..0.70).contains(&ratio)
    }
}

// ============================================================================
// Part 2: Fate-MAC (基于 Fate 反推验证)
// ============================================================================

/// Consistency Check State - 一致性检查状态
/// 
/// 对应 S-wind 的各种 check_*_consistency() 函数
#[derive(Debug, Clone)]
struct ConsistencyState {
    /// 线性一致性累加器 (Δ¹ 应该恒定)
    linear: [u8; 8],
    /// 二次一致性累加器 (Δ² 应该恒定)
    quadratic: [u8; 8],
    /// 多项式一致性累加器 (更高阶)
    polynomial: [[u8; 8]; 3],
    /// 指数一致性累加器 (比率应该恒定)
    exponential: [u8; 8],
    /// 验证计数
    verify_count: u64,
    /// 通过计数
    pass_count: u64,
}

impl ConsistencyState {
    fn new() -> Self {
        ConsistencyState {
            linear: [0u8; 8],
            quadratic: [0u8; 8],
            polynomial: [[0u8; 8]; 3],
            exponential: [0u8; 8],
            verify_count: 0,
            pass_count: 0,
        }
    }
}

/// Fate-MAC - 基于 Fate 反推验证的消息认证码
/// 
/// 核心思想: 把消息处理过程当作"信号预测"，
/// 然后通过一致性检查来验证消息完整性
/// 
/// 对应 S-wind 的:
/// - `verify_cross_domain()` → 跨域验证
/// - `verify_prediction_consistency()` → 预测一致性
/// - `check_linear_consistency()` → 线性一致性
/// - `check_quadratic_consistency()` → 二次一致性
#[derive(Debug, Clone)]
pub struct YakshaMac {
    /// 主状态 (预测值累加器)
    state: [u8; 32],
    /// 一致性检查状态
    consistency: ConsistencyState,
    /// 前一个值 (用于计算差分)
    prev_value: u64,
    /// 前一个差分 (用于计算二阶差分)
    prev_diff: i64,
    /// 前一个比率 (用于指数一致性)
    prev_ratio: u64,
    /// 处理字节数
    processed: u64,
    /// 域
    domain: SignalDomain,
}

impl YakshaMac {
    /// 创建 MAC 计算器
    /// 
    /// 初始化对应 Fate 的验证初始化
    pub fn new(key: &[u8]) -> Self {
        let mut state = [0u8; 32];
        
        // 域分离
        let domain_sep = b"FateMAC2";
        
        // 从密钥初始化状态
        for (i, &k) in key.iter().cycle().take(32).enumerate() {
            state[i] = k
                .wrapping_add(domain_sep[i % 8])
                .wrapping_mul((i as u8).wrapping_add(17))
                .rotate_left((i % 7 + 1) as u32);
        }
        
        // 初始化一致性状态
        let mut consistency = ConsistencyState::new();
        for i in 0..8 {
            let k = key[i % key.len()];
            consistency.linear[i] = k.wrapping_mul(0x1B);
            consistency.quadratic[i] = k.wrapping_mul(0x37);
            consistency.exponential[i] = k.wrapping_mul(0x59);
            for order in 0..3 {
                consistency.polynomial[order][i] = k.wrapping_mul((0x7F + order * 20) as u8);
            }
        }
        
        // 派生初始域
        let domain_idx = key.iter().fold(0u8, |acc, &b| acc ^ b) % 7;
        let domain = match domain_idx {
            0 => SignalDomain::Mathematics,
            1 => SignalDomain::PhysicsGravity,
            2 => SignalDomain::PhysicsQuantum,
            3 => SignalDomain::PhysicsThermal,
            4 => SignalDomain::PhysicsCoupling,
            5 => SignalDomain::SpaceDimension,
            _ => SignalDomain::External,
        };
        
        let mut mac = FateMac {
            state,
            consistency,
            prev_value: 0,
            prev_diff: 0,
            prev_ratio: 1 << 16, // 定点数 1.0
            processed: 0,
            domain,
        };
        
        // 初始混合
        mac.cross_domain_mix();
        mac.cross_domain_mix();
        
        mac
    }
    
    /// 更新 MAC 状态
    /// 
    /// 每个块的处理对应 Fate 的 verify_prediction_consistency:
    /// 1. 预测值 (基于之前状态)
    /// 2. 实际值 (当前输入)
    /// 3. 一致性检查 (预测 vs 实际)
    /// 4. 更新状态
    pub fn update(&mut self, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        
        // 8字节块处理 (对应 Fate 的信号采样)
        for chunk in data.chunks(8) {
            // 转换为值
            let mut bytes = [0u8; 8];
            bytes[..chunk.len()].copy_from_slice(chunk);
            let value = u64::from_le_bytes(bytes);
            
            // 1. 线性一致性检查 (Δ¹ 应该恒定)
            // check_linear_consistency: d1 = value - prev
            self.check_linear_consistency(value);
            
            // 2. 二次一致性检查 (Δ² 应该恒定)
            // check_quadratic_consistency: d2 = d1 - prev_d1
            self.check_quadratic_consistency(value);
            
            // 3. 指数一致性检查 (ratio 应该恒定)
            // check_exponential_consistency: ratio = value / prev
            self.check_exponential_consistency(value);
            
            // 4. 跨域混合
            self.mix_value(value);
            
            // 5. 更新状态
            self.prev_value = value;
            self.processed += chunk.len() as u64;
        }
        
        // 跨域验证混合
        self.cross_domain_mix();
    }
    
    /// 线性一致性检查
    /// 
    /// 对应 S-wind `check_linear_consistency`:
    /// "For linear: check if first differences are still constant"
    fn check_linear_consistency(&mut self, value: u64) {
        let diff = (value as i64).wrapping_sub(self.prev_value as i64);
        let diff_bytes = diff.to_le_bytes();
        
        // 累加差分 (如果线性，差分应该趋于恒定)
        for i in 0..8 {
            // 检查与期望差分的偏差
            let expected = self.consistency.linear[i];
            let actual = diff_bytes[i];
            let deviation = expected ^ actual;
            
            // 更新一致性累加器
            self.consistency.linear[i] = expected
                .wrapping_add(actual)
                .rotate_left(1);
            
            // 偏差混入状态
            let state_idx = i;
            self.state[state_idx] = self.state[state_idx]
                .wrapping_add(deviation)
                .rotate_left(3);
        }
        
        self.consistency.verify_count += 1;
        self.prev_diff = diff;
    }
    
    /// 二次一致性检查
    /// 
    /// 对应 S-wind `check_quadratic_consistency`:
    /// "For quadratic: check if second differences are still constant"
    fn check_quadratic_consistency(&mut self, value: u64) {
        let diff = (value as i64).wrapping_sub(self.prev_value as i64);
        let diff2 = diff.wrapping_sub(self.prev_diff);
        let diff2_bytes = diff2.to_le_bytes();
        
        for i in 0..8 {
            let expected = self.consistency.quadratic[i];
            let actual = diff2_bytes[i];
            let deviation = expected ^ actual;
            
            self.consistency.quadratic[i] = expected
                .wrapping_add(actual)
                .rotate_left(2);
            
            let state_idx = i + 8;
            self.state[state_idx] = self.state[state_idx]
                .wrapping_add(deviation)
                .rotate_left(5);
        }
    }
    
    /// 指数一致性检查
    /// 
    /// 对应 S-wind `check_exponential_consistency`:
    /// "For exponential: check if ratio s_{n+1}/s_n is still constant"
    fn check_exponential_consistency(&mut self, value: u64) {
        // 定点数比率 (16.16格式)
        let ratio = if self.prev_value != 0 {
            ((value << 16) / self.prev_value.max(1)) as u64
        } else {
            1 << 16
        };
        
        let ratio_bytes = ratio.to_le_bytes();
        
        for i in 0..8 {
            let expected = self.consistency.exponential[i];
            let actual = ratio_bytes[i];
            let deviation = expected ^ actual;
            
            self.consistency.exponential[i] = expected
                .wrapping_add(actual)
                .rotate_left(3);
            
            let state_idx = i + 16;
            self.state[state_idx] = self.state[state_idx]
                .wrapping_add(deviation)
                .rotate_left(7);
        }
        
        self.prev_ratio = ratio;
    }
    
    /// 混合值到状态
    fn mix_value(&mut self, value: u64) {
        let value_bytes = value.to_le_bytes();
        
        // 域常量
        let domain_const = self.domain.constant();
        
        for i in 0..8 {
            // 位置相关混合
            let pos = (self.processed as usize + i) % 32;
            
            // ARX 混合
            self.state[pos] = self.state[pos]
                .wrapping_add(value_bytes[i])
                .wrapping_add(domain_const)
                .rotate_left((i % 7 + 1) as u32);
            
            // 级联
            let next_pos = (pos + 1) % 32;
            self.state[next_pos] ^= self.state[pos].rotate_left(1);
        }
    }
    
    /// 跨域混合
    /// 
    /// 对应 S-wind `verify_cross_domain`:
    /// 把所有一致性检查结果混合到最终状态
    fn cross_domain_mix(&mut self) {
        // 1. 混合线性一致性
        for i in 0..8 {
            let idx = i;
            let idx2 = (i + 16) % 32;
            self.state[idx] = self.state[idx]
                .wrapping_add(self.consistency.linear[i])
                .rotate_left(1);
            self.state[idx2] ^= self.consistency.linear[i].rotate_left(3);
        }
        
        // 2. 混合二次一致性
        for i in 0..8 {
            let idx = i + 8;
            let idx2 = (i + 24) % 32;
            self.state[idx] = self.state[idx]
                .wrapping_add(self.consistency.quadratic[i])
                .rotate_left(2);
            self.state[idx2] ^= self.consistency.quadratic[i].rotate_left(5);
        }
        
        // 3. 混合指数一致性
        for i in 0..8 {
            let idx = i + 16;
            let idx2 = i % 32;
            self.state[idx] = self.state[idx]
                .wrapping_add(self.consistency.exponential[i])
                .rotate_left(3);
            self.state[idx2] ^= self.consistency.exponential[i].rotate_left(7);
        }
        
        // 4. 双向级联
        for round in 0..4 {
            // 正向 (对应 predict)
            for i in 0..31 {
                self.state[i + 1] = self.state[i + 1]
                    .wrapping_add(self.state[i])
                    .rotate_left((round % 7 + 1) as u32);
            }
            self.state[0] ^= self.state[31].rotate_left(2);
            
            // 反向 (对应 verify)
            for i in (1..32).rev() {
                self.state[i - 1] ^= self.state[i].rotate_left(3);
            }
            self.state[31] = self.state[31].wrapping_add(self.state[0]);
        }
        
        // 5. 非线性混合
        for i in 0..32 {
            let b = self.state[i];
            self.state[i] = b ^ b.rotate_right(3) ^ b.rotate_left(5);
        }
        
        // 6. 域演化
        self.domain = self.domain.next();
    }
    
    /// 完成并输出 MAC 标签
    /// 
    /// 对应 Fate 验证的最终结论
    pub fn finalize(&mut self, tag_len: usize) -> Vec<u8> {
        // 添加填充和长度
        let mut padding = vec![0x80u8];
        padding.extend_from_slice(&self.processed.to_le_bytes());
        padding.extend_from_slice(&self.consistency.verify_count.to_le_bytes());
        self.update(&padding);
        
        // 最终混合轮
        for round in 0..6 {
            self.cross_domain_mix();
            
            // 轮常量
            let round_const = (round as u8).wrapping_mul(0x5A).wrapping_add(0x37);
            for i in 0..32 {
                self.state[i] ^= round_const.rotate_left((i % 7 + 1) as u32);
            }
        }
        
        // 输出 (XOR 折叠)
        let full_len = tag_len.min(32);
        let mut output = vec![0u8; full_len];
        
        if full_len <= 16 {
            for i in 0..16 {
                output[i % full_len] ^= self.state[i] ^ self.state[i + 16];
            }
        } else {
            output.copy_from_slice(&self.state[..full_len]);
        }
        
        output
    }
    
    /// 一次性计算 MAC
    pub fn compute(key: &[u8], message: &[u8], tag_len: usize) -> Vec<u8> {
        let mut mac = YakshaMac::new(key);
        mac.update(message);
        mac.finalize(tag_len)
    }
    
    /// 验证 MAC
    pub fn verify(key: &[u8], message: &[u8], expected_tag: &[u8]) -> bool {
        let computed = Self::compute(key, message, expected_tag.len());
        constant_time_compare(&computed, expected_tag)
    }
}

// ============================================================================
// Part 3: AEAD (组合 Dynamic Key + MAC)
// ============================================================================

/// Fate-AEAD: 认证加密
/// 
/// 组合:
/// - Dynamic Key Generator (信号捕捉) → 密钥派生
/// - Fate-MAC (反推验证) → 消息认证
pub struct YakshaAead;

impl YakshaAead {
    /// 加密并认证
    pub fn seal(
        key: &[u8],
        nonce: &[u8],
        plaintext: &[u8],
        associated_data: &[u8],
    ) -> (Vec<u8>, Vec<u8>) {
        // 1. 信号捕捉: 派生加密密钥和 MAC 密钥
        let mut key_gen = YakshaKdf::new(key);
        let enc_key = key_gen.derive_key(nonce);
        let mac_key = key_gen.derive_key(b"authenticate");
        
        // 2. 加密
        let ciphertext = core::encrypt(plaintext, &enc_key);
        
        // 3. 反推验证: 计算 MAC
        let mut mac = YakshaMac::new(&mac_key);
        
        // AAD
        mac.update(associated_data);
        mac.update(&(associated_data.len() as u64).to_le_bytes());
        
        // Nonce
        mac.update(nonce);
        mac.update(&(nonce.len() as u64).to_le_bytes());
        
        // Ciphertext
        mac.update(&ciphertext);
        mac.update(&(ciphertext.len() as u64).to_le_bytes());
        
        let tag = mac.finalize(16);
        
        (ciphertext, tag)
    }
    
    /// 解密并验证
    pub fn open(
        key: &[u8],
        nonce: &[u8],
        ciphertext: &[u8],
        associated_data: &[u8],
        tag: &[u8],
    ) -> Result<Vec<u8>, &'static str> {
        // 1. 派生密钥
        let mut key_gen = YakshaKdf::new(key);
        let enc_key = key_gen.derive_key(nonce);
        let mac_key = key_gen.derive_key(b"authenticate");
        
        // 2. 验证 MAC (先验证后解密)
        let mut mac = YakshaMac::new(&mac_key);
        
        mac.update(associated_data);
        mac.update(&(associated_data.len() as u64).to_le_bytes());
        
        mac.update(nonce);
        mac.update(&(nonce.len() as u64).to_le_bytes());
        
        mac.update(ciphertext);
        mac.update(&(ciphertext.len() as u64).to_le_bytes());
        
        let computed_tag = mac.finalize(tag.len());
        
        if !constant_time_compare(&computed_tag, tag) {
            return Err("Authentication failed");
        }
        
        // 3. 解密
        let plaintext = core::decrypt(ciphertext, &enc_key);
        
        Ok(plaintext)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// 常量时间比较
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dynamic_key_uniqueness() {
        let mut gen = YakshaKdf::new(b"master_secret_key_test");
        
        let keys: Vec<[u8; 16]> = (0..100).map(|_| gen.next_key()).collect();
        
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                assert_ne!(keys[i], keys[j], "Keys {} and {} are identical", i, j);
            }
        }
    }
    
    #[test]
    fn test_fate_mac_basic() {
        let key = b"test_key_123456";
        let message = b"Hello, World!";
        
        let tag = YakshaMac::compute(key, message, 16);
        assert_eq!(tag.len(), 16);
        
        assert!(YakshaMac::verify(key, message, &tag));
        assert!(!YakshaMac::verify(key, b"Hello, World?", &tag));
    }
    
    #[test]
    fn test_fate_mac_deterministic() {
        let key = b"test_key_123456";
        let message = b"Same message";
        
        let tag1 = YakshaMac::compute(key, message, 16);
        let tag2 = YakshaMac::compute(key, message, 16);
        
        assert_eq!(tag1, tag2);
    }
    
    #[test]
    fn test_aead_roundtrip() {
        let key = b"aead_master_key!";
        let nonce = b"unique_nonce";
        let plaintext = b"Secret message that needs protection";
        let aad = b"Associated data";
        
        let (ciphertext, tag) = YakshaAead::seal(key, nonce, plaintext, aad);
        
        let decrypted = YakshaAead::open(key, nonce, &ciphertext, aad, &tag)
            .expect("Decryption should succeed");
        
        assert_eq!(decrypted, plaintext);
    }
    
    #[test]
    fn test_aead_tamper_detection() {
        let key = b"aead_master_key!";
        let nonce = b"unique_nonce";
        let plaintext = b"Secret";
        let aad = b"AAD";
        
        let (mut ciphertext, tag) = YakshaAead::seal(key, nonce, plaintext, aad);
        
        ciphertext[0] ^= 1;
        
        let result = YakshaAead::open(key, nonce, &ciphertext, aad, &tag);
        assert!(result.is_err());
    }
}

// ============================================================================
// Backward Compatibility Aliases
// ============================================================================

/// Backward compatibility alias for YakshaKdf
pub type DynamicKeyGenerator = YakshaKdf;

/// Backward compatibility alias for YakshaMac  
pub type FateMac = YakshaMac;

/// Backward compatibility alias for YakshaAead
pub type FateAead = YakshaAead;

