//! Minimal smoke checks only. Statistical / comparative validation:
//! run from repo root: `./external_bench/run_all.sh` (OpenSSL speed, dieharder, …).

use yaksha_cipher::{aead, core, neural, parametric};

fn main() {
    println!("yaksha_test: smoke (roundtrip + AEAD). Full gate: external_bench/run_all.sh\n");

    let test_data = b"Hello, Yaksha Cipher! This is a test message for encryption.";
    let key = b"secret_key_1234!";

    let encrypted = core::encrypt(test_data, key);
    assert_eq!(test_data.to_vec(), core::decrypt(&encrypted, key), "core roundtrip");

    let encrypted = parametric::encrypt(test_data, key);
    assert_eq!(
        test_data.to_vec(),
        parametric::decrypt(&encrypted, key),
        "parametric roundtrip"
    );

    let encrypted = neural::encrypt(test_data, key);
    assert_eq!(
        test_data.to_vec(),
        neural::decrypt(&encrypted, key),
        "neural roundtrip"
    );

    let aead_key = b"aead_smoke_master_key!";
    let nonce = b"smoke_nonce!";
    let aad = b"aad";
    let (ct, tag) = aead::YakshaAead::seal(aead_key, nonce, test_data, aad);
    let opened = aead::YakshaAead::open(aead_key, nonce, &ct, aad, &tag).expect("aead open");
    assert_eq!(opened, test_data, "aead roundtrip");

    println!("yaksha_test: all smoke checks OK.");
}
