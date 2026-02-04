//! Native CUDA FFI for Valhalla
//! 
//! 直接调用CUDA运行时，绕过cudarc版本限制
//! 支持CUDA 13.1 (RTX 4090)

use std::ffi::c_void;
use std::ptr;

// CUDA 运行时 FFI 声明
#[allow(non_camel_case_types)]
type cudaError_t = i32;

#[link(name = "cudart")]
extern "C" {
    fn cudaSetDevice(device: i32) -> cudaError_t;
    fn cudaGetDeviceCount(count: *mut i32) -> cudaError_t;
    fn cudaMalloc(devPtr: *mut *mut c_void, size: usize) -> cudaError_t;
    fn cudaFree(devPtr: *mut c_void) -> cudaError_t;
    fn cudaMemcpy(dst: *mut c_void, src: *const c_void, count: usize, kind: i32) -> cudaError_t;
    fn cudaDeviceSynchronize() -> cudaError_t;
    fn cudaGetLastError() -> cudaError_t;
    fn cudaGetErrorString(error: cudaError_t) -> *const i8;
}

// cuBLAS FFI 声明
#[allow(non_camel_case_types)]
type cublasHandle_t = *mut c_void;
type cublasStatus_t = i32;

#[link(name = "cublas")]
extern "C" {
    fn cublasCreate_v2(handle: *mut cublasHandle_t) -> cublasStatus_t;
    fn cublasDestroy_v2(handle: cublasHandle_t) -> cublasStatus_t;
    fn cublasDgemm_v2(
        handle: cublasHandle_t,
        transa: i32, transb: i32,
        m: i32, n: i32, k: i32,
        alpha: *const f64,
        A: *const f64, lda: i32,
        B: *const f64, ldb: i32,
        beta: *const f64,
        C: *mut f64, ldc: i32,
    ) -> cublasStatus_t;
    fn cublasDdot_v2(
        handle: cublasHandle_t,
        n: i32,
        x: *const f64, incx: i32,
        y: *const f64, incy: i32,
        result: *mut f64,
    ) -> cublasStatus_t;
    fn cublasDnrm2_v2(
        handle: cublasHandle_t,
        n: i32,
        x: *const f64, incx: i32,
        result: *mut f64,
    ) -> cublasStatus_t;
}

const CUDA_SUCCESS: i32 = 0;
const CUBLAS_STATUS_SUCCESS: i32 = 0;
const CUDA_MEMCPY_HOST_TO_DEVICE: i32 = 1;
const CUDA_MEMCPY_DEVICE_TO_HOST: i32 = 2;
const CUBLAS_OP_N: i32 = 0;

/// Native CUDA Accelerator
pub struct NativeCudaAccelerator {
    cublas_handle: cublasHandle_t,
    device_id: i32,
    is_initialized: bool,
}

impl NativeCudaAccelerator {
    /// 创建新的CUDA加速器
    pub fn new() -> Result<Self, String> {
        unsafe {
            // 检查CUDA设备
            let mut count = 0i32;
            if cudaGetDeviceCount(&mut count) != CUDA_SUCCESS || count == 0 {
                return Err("No CUDA devices found".into());
            }
            
            // 设置设备
            if cudaSetDevice(0) != CUDA_SUCCESS {
                return Err("Failed to set CUDA device".into());
            }
            
            // 创建cuBLAS句柄
            let mut handle: cublasHandle_t = ptr::null_mut();
            if cublasCreate_v2(&mut handle) != CUBLAS_STATUS_SUCCESS {
                return Err("Failed to create cuBLAS handle".into());
            }
            
            Ok(Self {
                cublas_handle: handle,
                device_id: 0,
                is_initialized: true,
            })
        }
    }
    
    /// 检查是否可用
    pub fn is_available(&self) -> bool {
        self.is_initialized
    }
    
    /// GPU批量余弦相似度
    pub fn batch_cosine_similarity(&self, vectors: &[Vec<f64>], query: &[f64]) -> Result<Vec<f64>, String> {
        if vectors.is_empty() {
            return Ok(vec![]);
        }
        
        let n = vectors.len();
        let dim = query.len();
        let mut results = vec![0.0f64; n];
        
        unsafe {
            // 分配GPU内存
            let mut d_query: *mut c_void = ptr::null_mut();
            let mut d_vec: *mut c_void = ptr::null_mut();
            
            if cudaMalloc(&mut d_query, dim * 8) != CUDA_SUCCESS {
                return Err("Failed to allocate query memory".into());
            }
            if cudaMalloc(&mut d_vec, dim * 8) != CUDA_SUCCESS {
                cudaFree(d_query);
                return Err("Failed to allocate vector memory".into());
            }
            
            // 复制query到GPU
            cudaMemcpy(d_query, query.as_ptr() as *const c_void, dim * 8, CUDA_MEMCPY_HOST_TO_DEVICE);
            
            // 计算query的范数
            let mut query_norm = 0.0f64;
            cublasDnrm2_v2(self.cublas_handle, dim as i32, d_query as *const f64, 1, &mut query_norm);
            
            // 对每个向量计算余弦相似度
            for (i, vec) in vectors.iter().enumerate() {
                // 复制向量到GPU
                cudaMemcpy(d_vec, vec.as_ptr() as *const c_void, dim * 8, CUDA_MEMCPY_HOST_TO_DEVICE);
                
                // 计算点积
                let mut dot = 0.0f64;
                cublasDdot_v2(self.cublas_handle, dim as i32, 
                    d_vec as *const f64, 1, 
                    d_query as *const f64, 1, 
                    &mut dot);
                
                // 计算向量范数
                let mut vec_norm = 0.0f64;
                cublasDnrm2_v2(self.cublas_handle, dim as i32, d_vec as *const f64, 1, &mut vec_norm);
                
                // 计算余弦相似度
                if vec_norm > 1e-10 && query_norm > 1e-10 {
                    results[i] = dot / (vec_norm * query_norm);
                }
            }
            
            cudaFree(d_query);
            cudaFree(d_vec);
        }
        
        Ok(results)
    }
    
    /// GPU矩阵乘法 (A[m,k] * B[k,n] = C[m,n])
    pub fn matmul_f64(&self, a: &[f64], b: &[f64], m: usize, n: usize, k: usize) -> Result<Vec<f64>, String> {
        let mut c = vec![0.0f64; m * n];
        
        unsafe {
            let mut d_a: *mut c_void = ptr::null_mut();
            let mut d_b: *mut c_void = ptr::null_mut();
            let mut d_c: *mut c_void = ptr::null_mut();
            
            cudaMalloc(&mut d_a, m * k * 8);
            cudaMalloc(&mut d_b, k * n * 8);
            cudaMalloc(&mut d_c, m * n * 8);
            
            cudaMemcpy(d_a, a.as_ptr() as *const c_void, m * k * 8, CUDA_MEMCPY_HOST_TO_DEVICE);
            cudaMemcpy(d_b, b.as_ptr() as *const c_void, k * n * 8, CUDA_MEMCPY_HOST_TO_DEVICE);
            
            let alpha = 1.0f64;
            let beta = 0.0f64;
            
            // cuBLAS uses column-major, so we compute B^T * A^T = (A*B)^T
            cublasDgemm_v2(
                self.cublas_handle,
                CUBLAS_OP_N, CUBLAS_OP_N,
                n as i32, m as i32, k as i32,
                &alpha,
                d_b as *const f64, n as i32,
                d_a as *const f64, k as i32,
                &beta,
                d_c as *mut f64, n as i32,
            );
            
            cudaMemcpy(c.as_mut_ptr() as *mut c_void, d_c, m * n * 8, CUDA_MEMCPY_DEVICE_TO_HOST);
            
            cudaFree(d_a);
            cudaFree(d_b);
            cudaFree(d_c);
        }
        
        Ok(c)
    }
    
    /// 批量信号处理
    pub fn batch_signal_weights(&self, intensities: &[f64], system_types: &[u8]) -> ([f64; 4], [u32; 4]) {
        // 这个操作CPU更快（简单的累加）
        let mut weights = [0.0f64; 4];
        let mut counts = [0u32; 4];
        
        for (&intensity, &sys_type) in intensities.iter().zip(system_types.iter()) {
            let idx = (sys_type as usize).min(3);
            let quality = 1.0 - (-intensity).exp();
            weights[idx] += intensity * quality;
            counts[idx] += 1;
        }
        
        (weights, counts)
    }
}

impl Drop for NativeCudaAccelerator {
    fn drop(&mut self) {
        if self.is_initialized {
            unsafe {
                cublasDestroy_v2(self.cublas_handle);
            }
        }
    }
}

/// 检查CUDA是否可用
pub fn cuda_available() -> bool {
    unsafe {
        let mut count = 0i32;
        cudaGetDeviceCount(&mut count) == CUDA_SUCCESS && count > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cuda_available() {
        println!("CUDA available: {}", cuda_available());
    }
}
