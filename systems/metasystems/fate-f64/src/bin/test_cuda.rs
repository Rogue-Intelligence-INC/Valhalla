use fate_f64::native_cuda::{NativeCudaAccelerator, cuda_available};

fn main() {
    println!("🔬 Testing Native CUDA for RTX 4090...");
    println!("CUDA available: {}", cuda_available());
    
    match NativeCudaAccelerator::new() {
        Ok(acc) => {
            println!("✅ CUDA Accelerator initialized!");
            
            let vectors = vec![
                vec![1.0, 0.0, 0.0],
                vec![0.0, 1.0, 0.0],
                vec![1.0, 1.0, 0.0],
            ];
            let query = vec![1.0, 0.0, 0.0];
            
            match acc.batch_cosine_similarity(&vectors, &query) {
                Ok(results) => {
                    println!("✅ GPU Cosine similarities: {:?}", results);
                    println!("   Expected: [1.0, 0.0, ~0.707]");
                }
                Err(e) => println!("❌ Error: {}", e),
            }
            
            // 测试矩阵乘法
            let a = vec![1.0, 2.0, 3.0, 4.0];  // 2x2
            let b = vec![5.0, 6.0, 7.0, 8.0];  // 2x2
            match acc.matmul_f64(&a, &b, 2, 2, 2) {
                Ok(c) => {
                    println!("✅ GPU MatMul result: {:?}", c);
                }
                Err(e) => println!("❌ MatMul error: {}", e),
            }
        }
        Err(e) => println!("❌ Failed to initialize CUDA: {}", e),
    }
}
