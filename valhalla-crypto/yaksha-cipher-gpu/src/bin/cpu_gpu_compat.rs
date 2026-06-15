//! CPU ↔ GPU ciphertext compatibility gate (industry KAT).

use yaksha_cipher::{fate_neural_decrypt, fate_neural_encrypt, neural, parametric};
use yaksha_cipher_gpu::{
    fate_evolve_gpu::{gpu_fate_neural_decrypt, gpu_fate_neural_encrypt},
    shell_gpu::{encrypt_neural_cpu_ocean, encrypt_parametric_cpu_ocean},
    encrypt_neural, encrypt_parametric, decrypt_neural, decrypt_parametric, GpuContext,
};

fn main() {
    let key = b"yaksha_cpu_gpu_compat_key_2026!!";
    let sizes = [63usize, 4095, 4096, 65536];
    let mut failed = 0u32;

    println!("=== CPU ↔ GPU compatibility KAT ===\n");

    let ctx = match GpuContext::new() {
        Some(c) => c,
        None => {
            eprintln!("FATAL: no GPU — cannot run CPU/GPU compat test");
            std::process::exit(1);
        }
    };

    for &len in &sizes {
        let plain: Vec<u8> = (0..len).map(|i| (i.wrapping_mul(31) ^ 0xA5) as u8).collect();

        // neural
        let cpu_n = neural::encrypt(&plain, key);
        let gpu_n = encrypt_neural(&ctx, &plain, key);
        if cpu_n != gpu_n {
            eprintln!("FAIL neural encrypt len={len}: CPU != GPU");
            failed += 1;
        } else if decrypt_neural(&ctx, &gpu_n, key) != plain {
            eprintln!("FAIL neural gpu decrypt len={len}");
            failed += 1;
        } else {
            println!("[OK] neural len={len}: CPU ct == GPU ct, roundtrip OK");
        }

        // parametric
        let cpu_p = parametric::encrypt(&plain, key);
        let gpu_p = encrypt_parametric(&ctx, &plain, key);
        if cpu_p != gpu_p {
            eprintln!("FAIL parametric encrypt len={len}: CPU != GPU");
            failed += 1;
        } else if decrypt_parametric(&ctx, &gpu_p, key) != plain {
            eprintln!("FAIL parametric gpu decrypt len={len}");
            failed += 1;
        } else {
            println!("[OK] parametric len={len}: CPU ct == GPU ct, roundtrip OK");
        }

        // GPU ocean path vs CPU ocean path (shell_gpu cross-check)
        let cpu_ocean = encrypt_neural_cpu_ocean(&plain, key);
        if cpu_ocean != gpu_n {
            eprintln!("FAIL neural CPU-ocean vs GPU-ocean len={len}");
            failed += 1;
        }
        let cpu_p_ocean = encrypt_parametric_cpu_ocean(&plain, key);
        if cpu_p_ocean != gpu_p {
            eprintln!("FAIL parametric CPU-ocean vs GPU-ocean len={len}");
            failed += 1;
        }
    }

    // fate evolve @ 64KB chunk
    let plain64: Vec<u8> = (0..65536).map(|i| (i as u8).wrapping_mul(17)).collect();
    let nonce = 42u64.to_le_bytes();
    let (cpu_ct, ctrl) = fate_neural_encrypt(&plain64, key, &nonce);
    let (gpu_ct, _) = gpu_fate_neural_encrypt(&plain64, key, &nonce, &ctx);
    if cpu_ct != gpu_ct {
        eprintln!("FAIL fate evolve 64KB: CPU ct != GPU ct");
        failed += 1;
    } else if gpu_fate_neural_decrypt(&gpu_ct, key, &nonce, &ctrl, &ctx) != plain64 {
        eprintln!("FAIL fate evolve gpu decrypt 64KB");
        failed += 1;
    } else if fate_neural_decrypt(&cpu_ct, key, &nonce, &ctrl) != plain64 {
        eprintln!("FAIL fate evolve cpu decrypt 64KB");
        failed += 1;
    } else {
        println!("[OK] fate evolve 64KB: CPU ct == GPU ct, cross-decrypt OK");
    }

    println!();
    if failed > 0 {
        eprintln!("cpu_gpu_compat: {failed} failure(s)");
        std::process::exit(1);
    }
    println!("cpu_gpu_compat: ALL PASSED");
}
