//! GPU Fate evolve roundtrip test with ocean disturbance XOR applied on GPU.

use yaksha_cipher_gpu::GpuContext;

use yaksha_cipher_gpu::fate_evolve_gpu::{gpu_fate_neural_decrypt, gpu_fate_neural_encrypt};

fn main() {
    let ctx = GpuContext::new().expect("GPU context");
    println!("GPU adapter: {}", ctx.info());

    let key = b"gpu_fate_roundtrip_master_key!";
    let nonce = b"gpu_fate_nonce";

    let sizes = [4096usize, 16384, 65536];

    for &size in &sizes {
        let plain: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let (ct, control) = gpu_fate_neural_encrypt(&plain, key, nonce, &ctx);
        let dec = gpu_fate_neural_decrypt(&ct, key, nonce, &control, &ctx);
        assert_eq!(dec, plain, "roundtrip failed for size={}", size);
        println!("roundtrip ok: size={} bytes", size);
    }

    println!("gpu fate neural test: all ok");
}

