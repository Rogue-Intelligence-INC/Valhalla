//! GPU version of `yaksha-cipher` yaksha_test.
//!
//! GPU backend in this crate supports:
//! - `parametric` (dimension)
//! - `neural` (quantum)
//!
//! This binary is a **smoke gate**: roundtrip correctness only.
//! Formal security is not defined by ad-hoc avalanche percentages; ciphertext statistics
//! for the project use **`external_bench`** (e.g. dieharder STS on generated streams).
//!
//! Run:
//!   cargo run --release --bin yaksha_test_gpu

use yaksha_cipher_gpu::{
    decrypt_dimension, decrypt_quantum, encrypt_dimension, encrypt_quantum, GpuContext,
};

fn main() {
    let ctx = match GpuContext::new() {
        Some(c) => c,
        None => {
            eprintln!("No GPU available for yaksha_test_gpu.");
            std::process::exit(1);
        }
    };

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              Yaksha Cipher GPU Test Suite                    ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    test_correctness(&ctx);

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    All Tests Passed (GPU)                    ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}

fn test_correctness(ctx: &GpuContext) {
    println!("【Correctness Tests】\n");

    let test_data = b"Hello, Yaksha Cipher! This is a test message for encryption.";
    let key = b"secret_key_1234!";

    // Parametric (dimension) mode
    let encrypted = encrypt_dimension(ctx, test_data, key);
    let decrypted = decrypt_dimension(ctx, &encrypted, key);
    let param_ok = test_data.to_vec() == decrypted;
    println!(
        "  Parametric mode: {}",
        if param_ok { "✓ PASS" } else { "✗ FAIL" }
    );

    // Neural (quantum) mode
    let encrypted = encrypt_quantum(ctx, test_data, key);
    let decrypted = decrypt_quantum(ctx, &encrypted, key);
    let neural_ok = test_data.to_vec() == decrypted;
    println!(
        "  Neural mode:     {}",
        if neural_ok { "✓ PASS" } else { "✗ FAIL" }
    );

    assert!(param_ok && neural_ok, "Correctness test failed");
}
