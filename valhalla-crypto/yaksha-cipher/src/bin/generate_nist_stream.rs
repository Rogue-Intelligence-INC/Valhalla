//! Multi-bitstream binary file for NIST SP 800-22 STS (proportion / uniformity).
//!
//! Each bitstream = `core::encrypt_with_session` (shell+Ocean) over random plaintext,
//! with a unique session derived from stream index.

use std::io::{self, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let num_streams: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(200);
    let bits_per_stream: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1_000_000);
    let bytes_per_stream = (bits_per_stream / 8) as usize;
    let key = b"yaksha_test_key_for_dieharder_2026!";

    eprintln!(
        "NIST stream file: {num_streams} bitstreams × {bits_per_stream} bits ({bytes_per_stream} B each)"
    );

    let mut lcg: u64 = 0x5DEECE66Du64;
    let mut out = io::stdout().lock();

    for idx in 0..num_streams {
        let session = (idx as u64).to_le_bytes();
        let mut session_ext = [0u8; 16];
        session_ext[..8].copy_from_slice(&session);

        let plaintext: Vec<u8> = (0..bytes_per_stream)
            .map(|_| {
                lcg = lcg.wrapping_mul(0x5DEECE66Du64).wrapping_add(0xB);
                (lcg >> 16) as u8
            })
            .collect();

        let ct = yaksha_cipher::core::encrypt_with_session(&plaintext, key, &session_ext);
        if ct.len() != bytes_per_stream {
            eprintln!(
                "WARN stream {idx}: ciphertext len {} != expected {bytes_per_stream}",
                ct.len()
            );
        }
        out.write_all(&ct).expect("write stream");
        if (idx + 1) % 20 == 0 {
            eprintln!("  ... {}/{} streams", idx + 1, num_streams);
        }
    }
    eprintln!("Done. total bytes = {}", num_streams as u64 * bytes_per_stream as u64);
}
