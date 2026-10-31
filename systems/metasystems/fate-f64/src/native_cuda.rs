//! Native CUDA FFI for Valhalla
//! 
//! 直接调用CUDA运行时，绕过cudarc版本限制
//! 支持CUDA 13.1 (RTX 4090)

use std::ffi::c_void;
use std::ptr;

// CUDA 运行时 FFI 声明
#[allow(non_camel_case_types)]
type cudaError_t = i32;

#[link(name = "cudart", kind = "static")]
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
const CUBLAS_OP_T: i32 = 1;

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

    /// 行主序 P 是 n×d。返回 P Pᵀ，n×n，对称。
    pub fn row_gram(&self, row_major: &[f64], n: usize, d: usize) -> Result<Vec<f64>, String> {
        if n == 0 || d == 0 {
            return Ok(Vec::new());
        }
        if row_major.len() < n * d {
            return Err("position buffer short".into());
        }
        let mut gram = vec![0.0f64; n * n];
        unsafe {
            let mut d_p: *mut c_void = ptr::null_mut();
            let mut d_g: *mut c_void = ptr::null_mut();
            if cudaMalloc(&mut d_p, n * d * 8) != CUDA_SUCCESS {
                return Err("cudaMalloc positions".into());
            }
            if cudaMalloc(&mut d_g, n * n * 8) != CUDA_SUCCESS {
                cudaFree(d_p);
                return Err("cudaMalloc gram".into());
            }
            if cudaMemcpy(d_p, row_major.as_ptr() as *const c_void, n * d * 8, CUDA_MEMCPY_HOST_TO_DEVICE) != CUDA_SUCCESS {
                cudaFree(d_p);
                cudaFree(d_g);
                return Err("copy positions".into());
            }
            let alpha = 1.0f64;
            let beta = 0.0f64;
            let status = cublasDgemm_v2(
                self.cublas_handle,
                CUBLAS_OP_T,
                CUBLAS_OP_N,
                n as i32,
                n as i32,
                d as i32,
                &alpha,
                d_p as *const f64,
                d as i32,
                d_p as *const f64,
                d as i32,
                &beta,
                d_g as *mut f64,
                n as i32,
            );
            if status != CUBLAS_STATUS_SUCCESS {
                cudaFree(d_p);
                cudaFree(d_g);
                return Err(format!("cublas gram {status}"));
            }
            cudaDeviceSynchronize();
            if cudaMemcpy(gram.as_mut_ptr() as *mut c_void, d_g, n * n * 8, CUDA_MEMCPY_DEVICE_TO_HOST) != CUDA_SUCCESS {
                cudaFree(d_p);
                cudaFree(d_g);
                return Err("copy gram".into());
            }
            cudaFree(d_p);
            cudaFree(d_g);
        }
        Ok(gram)
    }

    /// 行主序 F = W P。W 是 n×n，P 是 n×d，F 是 n×d。
    pub fn row_mul(&self, weights: &[f64], positions: &[f64], n: usize, d: usize) -> Result<Vec<f64>, String> {
        if n == 0 || d == 0 {
            return Ok(Vec::new());
        }
        if weights.len() < n * n || positions.len() < n * d {
            return Err("field buffer short".into());
        }
        let mut forced = vec![0.0f64; n * d];
        unsafe {
            let mut d_w: *mut c_void = ptr::null_mut();
            let mut d_p: *mut c_void = ptr::null_mut();
            let mut d_f: *mut c_void = ptr::null_mut();
            if cudaMalloc(&mut d_w, n * n * 8) != CUDA_SUCCESS {
                return Err("cudaMalloc weights".into());
            }
            if cudaMalloc(&mut d_p, n * d * 8) != CUDA_SUCCESS {
                cudaFree(d_w);
                return Err("cudaMalloc positions".into());
            }
            if cudaMalloc(&mut d_f, n * d * 8) != CUDA_SUCCESS {
                cudaFree(d_w);
                cudaFree(d_p);
                return Err("cudaMalloc force".into());
            }
            let copy_w = cudaMemcpy(d_w, weights.as_ptr() as *const c_void, n * n * 8, CUDA_MEMCPY_HOST_TO_DEVICE);
            let copy_p = cudaMemcpy(d_p, positions.as_ptr() as *const c_void, n * d * 8, CUDA_MEMCPY_HOST_TO_DEVICE);
            if copy_w != CUDA_SUCCESS || copy_p != CUDA_SUCCESS {
                cudaFree(d_w);
                cudaFree(d_p);
                cudaFree(d_f);
                return Err("copy field".into());
            }
            let alpha = 1.0f64;
            let beta = 0.0f64;
            let status = cublasDgemm_v2(
                self.cublas_handle,
                CUBLAS_OP_N,
                CUBLAS_OP_N,
                d as i32,
                n as i32,
                n as i32,
                &alpha,
                d_p as *const f64,
                d as i32,
                d_w as *const f64,
                n as i32,
                &beta,
                d_f as *mut f64,
                d as i32,
            );
            if status != CUBLAS_STATUS_SUCCESS {
                cudaFree(d_w);
                cudaFree(d_p);
                cudaFree(d_f);
                return Err(format!("cublas force {status}"));
            }
            cudaDeviceSynchronize();
            if cudaMemcpy(forced.as_mut_ptr() as *mut c_void, d_f, n * d * 8, CUDA_MEMCPY_DEVICE_TO_HOST) != CUDA_SUCCESS {
                cudaFree(d_w);
                cudaFree(d_p);
                cudaFree(d_f);
                return Err("copy force".into());
            }
            cudaFree(d_w);
            cudaFree(d_p);
            cudaFree(d_f);
        }
        Ok(forced)
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

    #[test]
    fn row_gram_matches_dots() {
        let accel = NativeCudaAccelerator::new().expect("cuda");
        let positions = [1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 2.0];
        let gram = accel.row_gram(&positions, 4, 2).expect("gram");
        let mut expect = [0.0; 16];
        for i in 0..4 {
            for j in 0..4 {
                let mut dot = 0.0;
                for k in 0..2 {
                    dot += positions[i * 2 + k] * positions[j * 2 + k];
                }
                expect[i * 4 + j] = dot;
            }
        }
        for (got, want) in gram.iter().zip(expect.iter()) {
            assert!((got - want).abs() < 1e-9, "{got} vs {want}");
        }
        let weights = [
            0.0, 0.5, 0.0, 0.0,
            0.0, 0.0, 0.25, 0.0,
            0.0, 0.0, 0.0, 1.0,
            0.5, 0.0, 0.0, 0.0,
        ];
        let forced = accel.row_mul(&weights, &positions, 4, 2).expect("mul");
        for i in 0..4 {
            for k in 0..2 {
                let mut acc = 0.0;
                for j in 0..4 {
                    acc += weights[i * 4 + j] * positions[j * 2 + k];
                }
                let got = forced[i * 2 + k];
                assert!((got - acc).abs() < 1e-9, "F[{i},{k}] {got} vs {acc}");
            }
        }
    }
}
