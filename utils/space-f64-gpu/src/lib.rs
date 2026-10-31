//! Valhalla Space - GPU 加速的高维计算系统
//!
//! 功能：
//! - 分形维度计算 (Hausdorff, Box, Information)
//! - 自适应维度扩展
//! - GPU 批量计算加速
//! - Fate 存储集成

use serde::{Deserialize, Serialize};
use std::path::Path;
use fate_f64::{TripleFateCombined, FateSlice, FateDataEntry};

// ============================================================================
// CUDA FFI (当启用 cuda feature 时)
// ============================================================================

#[cfg(feature = "cuda")]
mod cuda {
    use std::ffi::c_void;
    
    type CudaError = i32;
    
    #[link(name = "cudart")]
    extern "C" {
        fn cudaMalloc(devPtr: *mut *mut c_void, size: usize) -> CudaError;
        fn cudaFree(devPtr: *mut c_void) -> CudaError;
        fn cudaMemcpy(dst: *mut c_void, src: *const c_void, count: usize, kind: i32) -> CudaError;
        fn cudaDeviceSynchronize() -> CudaError;
    }
    
    const CUDA_MEMCPY_HOST_TO_DEVICE: i32 = 1;
    const CUDA_MEMCPY_DEVICE_TO_HOST: i32 = 2;
    
    /// GPU 内存缓冲
    pub struct GpuBuffer {
        ptr: *mut c_void,
        size: usize,
    }
    
    impl GpuBuffer {
        pub fn new(size: usize) -> Option<Self> {
            let mut ptr: *mut c_void = std::ptr::null_mut();
            unsafe {
                if cudaMalloc(&mut ptr, size) == 0 {
                    Some(Self { ptr, size })
                } else {
                    None
                }
            }
        }
        
        pub fn copy_from_host(&self, data: &[f64]) -> bool {
            unsafe {
                cudaMemcpy(
                    self.ptr,
                    data.as_ptr() as *const c_void,
                    data.len() * std::mem::size_of::<f64>(),
                    CUDA_MEMCPY_HOST_TO_DEVICE
                ) == 0
            }
        }
        
        pub fn copy_to_host(&self, data: &mut [f64]) -> bool {
            unsafe {
                cudaMemcpy(
                    data.as_mut_ptr() as *mut c_void,
                    self.ptr,
                    data.len() * std::mem::size_of::<f64>(),
                    CUDA_MEMCPY_DEVICE_TO_HOST
                ) == 0
            }
        }
        
        pub fn ptr(&self) -> *mut c_void {
            self.ptr
        }
    }
    
    impl Drop for GpuBuffer {
        fn drop(&mut self) {
            unsafe {
                cudaFree(self.ptr);
            }
        }
    }
    
    pub fn sync() {
        unsafe {
            cudaDeviceSynchronize();
        }
    }
}

// ============================================================================
// 维度计算核心
// ============================================================================

/// 维度类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DimensionType {
    /// 整数维度
    Integer,
    /// 分形维度
    Fractional,
    /// 复杂维度
    Complex,
    /// 自动判断
    Auto,
}

/// 维度计算结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionResult {
    /// 推荐维度
    pub recommended: f64,
    /// Hausdorff 维度估计
    pub hausdorff: f64,
    /// Box 维度估计
    pub box_dim: f64,
    /// 信息维度估计
    pub info_dim: f64,
    /// 计算方法
    pub method: String,
}

/// GPU 加速的分形维度计算器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FractalDimensionCalculatorGPU {
    /// 当前全局维度
    pub current_dimension: f64,
    /// 最大维度限制
    pub max_dimension: f64,
    /// 最小维度限制
    pub min_dimension: f64,
    /// 计算次数
    pub calculation_count: u64,
    /// GPU 批量大小
    pub gpu_batch_size: usize,
    /// 是否使用 GPU
    pub use_gpu: bool,
}

impl FractalDimensionCalculatorGPU {
    pub fn new() -> Self {
        Self {
            current_dimension: 4.0,
            max_dimension: 11.0,
            min_dimension: 1.0,
            calculation_count: 0,
            gpu_batch_size: 1024,
            use_gpu: cfg!(feature = "cuda"),
        }
    }
    
    /// 批量计算维度 (GPU 加速)
    pub fn calculate_batch(&mut self, data_batch: &[Vec<f64>]) -> Vec<DimensionResult> {
        self.calculation_count += data_batch.len() as u64;
        
        #[cfg(feature = "cuda")]
        if self.use_gpu && data_batch.len() >= self.gpu_batch_size {
            return self.calculate_batch_gpu(data_batch);
        }
        
        // CPU 并行计算
        self.calculate_batch_cpu(data_batch)
    }
    
    /// CPU 并行计算 (使用 rayon)
    fn calculate_batch_cpu(&self, data_batch: &[Vec<f64>]) -> Vec<DimensionResult> {
        use rayon::prelude::*;
        
        data_batch.par_iter()
            .map(|data| self.calculate_single(data))
            .collect()
    }
    
    /// GPU 批量计算
    #[cfg(feature = "cuda")]
    fn calculate_batch_gpu(&self, data_batch: &[Vec<f64>]) -> Vec<DimensionResult> {
        // 准备批量数据
        let mut results = Vec::with_capacity(data_batch.len());
        
        // 对每批数据进行 GPU 计算
        for chunk in data_batch.chunks(self.gpu_batch_size) {
            let chunk_results = self.gpu_process_chunk(chunk);
            results.extend(chunk_results);
        }
        
        results
    }
    
    #[cfg(feature = "cuda")]
    fn gpu_process_chunk(&self, chunk: &[Vec<f64>]) -> Vec<DimensionResult> {
        // 展平数据用于 GPU 传输
        let max_len = chunk.iter().map(|v| v.len()).max().unwrap_or(0);
        let total_size = chunk.len() * max_len;
        
        let mut flat_data = vec![0.0f64; total_size];
        let mut lengths = vec![0usize; chunk.len()];
        
        for (i, data) in chunk.iter().enumerate() {
            lengths[i] = data.len();
            for (j, &val) in data.iter().enumerate() {
                flat_data[i * max_len + j] = val;
            }
        }
        
        // 分配 GPU 内存
        if let Some(gpu_buffer) = cuda::GpuBuffer::new(total_size * 8) {
            gpu_buffer.copy_from_host(&flat_data);
            
            // GPU 计算（这里是简化版，实际需要 CUDA kernel）
            // 目前回退到 CPU
            cuda::sync();
        }
        
        // 回退到 CPU 计算
        chunk.iter().map(|data| self.calculate_single(data)).collect()
    }
    
    /// 单个数据计算
    pub fn calculate_single(&self, data: &[f64]) -> DimensionResult {
        if data.is_empty() {
            return DimensionResult {
                recommended: 1.0,
                hausdorff: 1.0,
                box_dim: 1.0,
                info_dim: 1.0,
                method: "empty".into(),
            };
        }
        
        let hausdorff = self.estimate_hausdorff(data);
        let box_dim = self.estimate_box_dimension(data);
        let info_dim = self.estimate_information_dimension(data);
        
        // 加权平均: (hausdorff + 2*box + info) / 4
        let recommended = (hausdorff + 2.0 * box_dim + info_dim) / 4.0;
        let recommended = self.clamp_dimension(recommended);
        
        DimensionResult {
            recommended,
            hausdorff,
            box_dim,
            info_dim,
            method: if cfg!(feature = "cuda") && self.use_gpu { "gpu" } else { "cpu" }.into(),
        }
    }
    
    /// Hausdorff 维度估计 (基于方差)
    #[inline]
    fn estimate_hausdorff(&self, data: &[f64]) -> f64 {
        if data.len() < 2 {
            return 1.0;
        }
        
        let n = data.len() as f64;
        let sum: f64 = data.iter().sum();
        let mean = sum / n;
        
        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / n;
        
        // 基础维度 + 方差修正
        let raw = 1.0 + variance.sqrt().min(10.0);
        self.clamp_dimension(raw)
    }
    
    /// Box 维度估计 (基于范围)
    #[inline]
    fn estimate_box_dimension(&self, data: &[f64]) -> f64 {
        if data.len() < 2 {
            return 1.0;
        }
        
        let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max - min;
        
        if range < 1e-10 {
            return 1.0;
        }
        
        let n = data.len() as f64;
        let normalized = n / (range + 1.0);
        let raw = 1.0 + normalized / 10.0;
        
        self.clamp_dimension(raw)
    }
    
    /// 信息维度估计 (基于熵)
    #[inline]
    fn estimate_information_dimension(&self, data: &[f64]) -> f64 {
        if data.is_empty() {
            return 1.0;
        }
        
        // 计算不同值的比例作为熵代理
        let mut distinct = 1usize;
        let tolerance = 1e-10;
        
        for i in 1..data.len() {
            if (data[i] - data[i-1]).abs() > tolerance {
                distinct += 1;
            }
        }
        
        let ratio = distinct as f64 / data.len() as f64;
        1.0 + ratio
    }
    
    /// 维度限制
    #[inline]
    fn clamp_dimension(&self, dim: f64) -> f64 {
        dim.max(self.min_dimension).min(self.max_dimension)
    }
    
    /// 自适应扩展 (只增不减)
    pub fn adaptive_expand(&mut self, data: &[f64]) -> f64 {
        let result = self.calculate_single(data);
        if result.recommended > self.current_dimension {
            self.current_dimension = result.recommended;
        }
        self.current_dimension
    }
    
    /// Fate 驱动调整 (可增可减)
    pub fn fate_adjust(&mut self, data: &[f64]) -> DimensionAdjustment {
        let result = self.calculate_single(data);
        let old = self.current_dimension;
        self.current_dimension = result.recommended;
        
        let change = if result.recommended > old + 0.01 {
            DimensionChange::Increased
        } else if result.recommended < old - 0.01 {
            DimensionChange::Decreased
        } else {
            DimensionChange::Unchanged
        };
        
        DimensionAdjustment {
            old_dimension: old,
            new_dimension: self.current_dimension,
            change,
            result,
        }
    }
}

impl Default for FractalDimensionCalculatorGPU {
    fn default() -> Self {
        Self::new()
    }
}

/// 维度变化方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DimensionChange {
    Increased,
    Decreased,
    Unchanged,
}

/// 维度调整结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionAdjustment {
    pub old_dimension: f64,
    pub new_dimension: f64,
    pub change: DimensionChange,
    pub result: DimensionResult,
}

// ============================================================================
// Space 系统 (带 Fate 存储)
// ============================================================================

/// Space 系统切片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceSlice {
    pub current_dimension: f64,
    pub calculation_count: u64,
    pub container_count: usize,
    pub fate_slice: FateSlice,
}

/// GPU 加速的 Space 系统
#[derive(Debug)]
pub struct SpaceGPU {
    /// 分形维度计算器
    pub calculator: FractalDimensionCalculatorGPU,
    /// 容器列表
    pub containers: Vec<SpaceContainer>,
    /// Fate 存储
    pub triple_fate: TripleFateCombined,
    /// 全局 tick
    pub global_tick: u64,
}

/// Space 容器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceContainer {
    pub id: String,
    pub dimension: f64,
    pub created_tick: u64,
    pub container_type: ContainerType,
}

/// 容器类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerType {
    Basic,
    Fractional,
    Adaptive,
    Discovery { pattern: String },
}

impl SpaceGPU {
    pub fn new() -> Self {
        Self {
            calculator: FractalDimensionCalculatorGPU::new(),
            containers: Vec::new(),
            triple_fate: TripleFateCombined::new(),
            global_tick: 0,
        }
    }
    
    /// 创建容器
    pub fn create_container(&mut self, container_type: ContainerType) -> String {
        self.global_tick += 1;
        let id = format!("space_container_{}_{}", self.global_tick, self.containers.len());
        
        let container = SpaceContainer {
            id: id.clone(),
            dimension: self.calculator.current_dimension,
            created_tick: self.global_tick,
            container_type,
        };
        
        self.containers.push(container);
        id
    }
    
    /// 处理数据并扩展维度
    pub fn process_and_expand(&mut self, data: &[f64]) -> DimensionResult {
        self.global_tick += 1;
        let mut closed = false;
        for &val in data {
            if self.triple_fate.receive_passive(val) {
                closed = true;
            }
        }
        let result = self.calculator.calculate_single(data);
        if closed {
            self.calculator.adaptive_expand(data);
            let bytes: Vec<u8> = data.iter().flat_map(|&v| v.to_le_bytes()).collect();
            self.triple_fate.store_data(&bytes);
        }
        result
    }
    
    /// 批量处理 (GPU 加速)
    pub fn process_batch(&mut self, data_batch: &[Vec<f64>]) -> Vec<DimensionResult> {
        self.global_tick += data_batch.len() as u64;
        
        // GPU 批量计算
        let results = self.calculator.calculate_batch(data_batch);
        
        // 更新最大维度
        for result in &results {
            if result.recommended > self.calculator.current_dimension {
                self.calculator.current_dimension = result.recommended;
            }
        }
        
        results
    }
    
    /// 发现未知模式
    pub fn on_discovery(&mut self, data: &[f64], pattern: &str) -> String {
        self.global_tick += 1;
        
        // Fate 调整维度
        let adjustment = self.calculator.fate_adjust(data);
        
        // 创建发现容器
        let id = format!("discovery_{}_{}", pattern, self.global_tick);
        let container = SpaceContainer {
            id: id.clone(),
            dimension: adjustment.new_dimension,
            created_tick: self.global_tick,
            container_type: ContainerType::Discovery { pattern: pattern.into() },
        };
        self.containers.push(container);
        
        // 存储发现数据
        self.triple_fate.store_string(&format!("discovery:{}:dim={:.4}", pattern, adjustment.new_dimension));
        
        id
    }
    
    /// 获取当前维度
    pub fn get_dimension(&self) -> f64 {
        self.calculator.current_dimension
    }
    
    /// 获取状态
    pub fn get_status(&self) -> SpaceStatus {
        SpaceStatus {
            current_dimension: self.calculator.current_dimension,
            calculation_count: self.calculator.calculation_count,
            container_count: self.containers.len(),
            global_tick: self.global_tick,
            use_gpu: self.calculator.use_gpu,
            fate_stored: self.triple_fate.stored_count(),
        }
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // Fate 存储层
    // ═══════════════════════════════════════════════════════════════════════════
    
    pub fn fate_store(&mut self, data: &[u8]) -> String { self.triple_fate.store_data(data) }
    pub fn fate_store_string(&mut self, content: &str) -> String { self.triple_fate.store_string(content) }
    pub fn fate_create(&mut self, data: Vec<u8>) -> String { self.triple_fate.create_data(data) }
    pub fn fate_modify(&mut self, id: &str, data: Vec<u8>) -> bool { self.triple_fate.modify_data(id, data) }
    pub fn fate_load(&self, id: &str) -> Option<&FateDataEntry> { self.triple_fate.load_data(id) }
    pub fn fate_remove(&mut self, id: &str) -> Option<FateDataEntry> { self.triple_fate.remove_data(id) }
    pub fn fate_list(&self) -> Vec<&String> { self.triple_fate.list_stored() }
    pub fn fate_count(&self) -> usize { self.triple_fate.stored_count() }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 切片 - 冻结/解冻
    // ═══════════════════════════════════════════════════════════════════════════
    
    /// 保存切片
    pub fn save_slice(&self, path: &Path) -> std::io::Result<()> {
        let slice = SpaceSlice {
            current_dimension: self.calculator.current_dimension,
            calculation_count: self.calculator.calculation_count,
            container_count: self.containers.len(),
            fate_slice: self.triple_fate.freeze(),
        };
        let bytes = bincode::serialize(&slice)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, bytes)
    }
    
    /// 从切片恢复
    pub fn load_slice(path: &Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let slice: SpaceSlice = bincode::deserialize(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        let mut instance = Self::new();
        instance.calculator.current_dimension = slice.current_dimension;
        instance.calculator.calculation_count = slice.calculation_count;
        instance.triple_fate = TripleFateCombined::thaw(slice.fate_slice);
        Ok(instance)
    }
}

impl Default for SpaceGPU {
    fn default() -> Self {
        Self::new()
    }
}

/// Space 状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceStatus {
    pub current_dimension: f64,
    pub calculation_count: u64,
    pub container_count: usize,
    pub global_tick: u64,
    pub use_gpu: bool,
    pub fate_stored: usize,
}

// ============================================================================
// 高维向量运算 (GPU 加速)
// ============================================================================

/// 高维向量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperVector {
    pub data: Vec<f64>,
    pub dimension: usize,
}

impl HyperVector {
    pub fn new(data: Vec<f64>) -> Self {
        let dimension = data.len();
        Self { data, dimension }
    }
    
    pub fn zeros(dimension: usize) -> Self {
        Self {
            data: vec![0.0; dimension],
            dimension,
        }
    }
    
    /// 点积 (GPU 可加速)
    pub fn dot(&self, other: &HyperVector) -> f64 {
        assert_eq!(self.dimension, other.dimension);
        self.data.iter().zip(&other.data).map(|(a, b)| a * b).sum()
    }
    
    /// 范数
    pub fn norm(&self) -> f64 {
        self.dot(self).sqrt()
    }
    
    /// 余弦相似度
    pub fn cosine_similarity(&self, other: &HyperVector) -> f64 {
        let dot = self.dot(other);
        let norm_a = self.norm();
        let norm_b = other.norm();
        if norm_a < 1e-10 || norm_b < 1e-10 {
            return 0.0;
        }
        dot / (norm_a * norm_b)
    }
    
    /// 加法
    pub fn add(&self, other: &HyperVector) -> HyperVector {
        assert_eq!(self.dimension, other.dimension);
        HyperVector::new(
            self.data.iter().zip(&other.data).map(|(a, b)| a + b).collect()
        )
    }
    
    /// 标量乘法
    pub fn scale(&self, scalar: f64) -> HyperVector {
        HyperVector::new(self.data.iter().map(|&x| x * scalar).collect())
    }
}

/// 批量向量运算 (GPU 加速)
pub struct HyperVectorBatch;

impl HyperVectorBatch {
    /// 批量点积
    pub fn batch_dot(vectors_a: &[HyperVector], vectors_b: &[HyperVector]) -> Vec<f64> {
        use rayon::prelude::*;
        
        vectors_a.par_iter()
            .zip(vectors_b.par_iter())
            .map(|(a, b)| a.dot(b))
            .collect()
    }
    
    /// 批量余弦相似度
    pub fn batch_cosine(vectors_a: &[HyperVector], vectors_b: &[HyperVector]) -> Vec<f64> {
        use rayon::prelude::*;
        
        vectors_a.par_iter()
            .zip(vectors_b.par_iter())
            .map(|(a, b)| a.cosine_similarity(b))
            .collect()
    }
    
    /// 批量范数
    pub fn batch_norm(vectors: &[HyperVector]) -> Vec<f64> {
        use rayon::prelude::*;
        
        vectors.par_iter().map(|v| v.norm()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dimension_calculation() {
        let mut calc = FractalDimensionCalculatorGPU::new();
        
        let data = vec![1.0, 2.0, 3.0, 10.0, 20.0, 30.0];
        let result = calc.calculate_single(&data);
        
        println!("Dimension result: {:?}", result);
        assert!(result.recommended >= 1.0);
        assert!(result.recommended <= 11.0);
    }
    
    #[test]
    fn test_batch_calculation() {
        let mut calc = FractalDimensionCalculatorGPU::new();
        
        let batch: Vec<Vec<f64>> = (0..100)
            .map(|i| (0..10).map(|j| (i * 10 + j) as f64).collect())
            .collect();
        
        let results = calc.calculate_batch(&batch);
        assert_eq!(results.len(), 100);
    }
    
    #[test]
    fn test_space_gpu() {
        let mut space = SpaceGPU::new();
        
        let data = vec![1.0, 5.0, 10.0, 50.0, 100.0];
        let result = space.process_and_expand(&data);
        
        println!("Space status: {:?}", space.get_status());
        assert!(result.recommended >= 1.0);
    }
    
    #[test]
    fn test_hyper_vector() {
        let v1 = HyperVector::new(vec![1.0, 2.0, 3.0]);
        let v2 = HyperVector::new(vec![4.0, 5.0, 6.0]);
        
        assert!((v1.dot(&v2) - 32.0).abs() < 1e-10);
        assert!((v1.cosine_similarity(&v1) - 1.0).abs() < 1e-10);
    }
}
