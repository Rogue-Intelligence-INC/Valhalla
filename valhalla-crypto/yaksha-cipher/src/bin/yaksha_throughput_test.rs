//! CPU throughput test (fast profile).

use std::hint::black_box;
use std::time::Instant;

use yaksha_cipher::aead::YakshaAead;
use yaksha_cipher::config;
use yaksha_cipher::core;

fn main() {
    println!("=== Yaksha CPU Throughput Test ===");
    println!(
        "profile={:?} mac={:?} ocean_rounds={} shell={}",
        config::profile(),
        config::mac_mode(),
        config::ocean_mix_rounds(),
        config::shell_enabled()
    );

    let size = 16 * 1024 * 1024;
    let key = b"yaksha_aead_master_key_32bytes!!";
    let nonce = b"yaksha_nonce_12b";
    let aad = b"aad";
    let plaintext: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();

    let (ct, tag) = YakshaAead::seal(key, nonce, &plaintext, aad);
    let pt = YakshaAead::open(key, nonce, &ct, aad, &tag).expect("AEAD roundtrip");
    assert_eq!(pt, plaintext);
    println!("AEAD roundtrip: OK");

    let core_key = b"benchmark_key_1234567890!";
    let c = core::encrypt(&plaintext, core_key);
    assert_eq!(core::decrypt(&c, core_key), plaintext);
    println!("Core roundtrip: OK");

    bench("AEAD seal", 3, size, || {
        black_box(YakshaAead::seal(key, nonce, &plaintext, aad));
    });
    bench("AEAD open", 3, size, || {
        black_box(YakshaAead::open(key, nonce, &ct, aad, &tag).unwrap());
    });
    bench("core encrypt", 3, size, || {
        black_box(core::encrypt(&plaintext, core_key));
    });
}

fn bench(label: &str, iters: u32, size: usize, mut f: impl FnMut()) {
    f();
    let start = Instant::now();
    for _ in 0..iters {
        f();
    }
    let secs = start.elapsed().as_secs_f64();
    let mbps = size as f64 * iters as f64 / secs / (1024.0 * 1024.0);
    println!("  {:<16} {:>8.0} MB/s", label, mbps);
}
