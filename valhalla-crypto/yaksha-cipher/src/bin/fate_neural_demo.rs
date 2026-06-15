use yaksha_cipher::{fate_neural_decrypt, fate_neural_encrypt};

fn main() {
    let key = b"FateYakshaMasterKey!";
    let nonce = b"yaksha_fate_demo_nonce";
    let plaintext = b"Fate-controlled Yaksha demo payload (roundtrip test)";

    let (ciphertext, control) = fate_neural_encrypt(plaintext, key, nonce);
    let recovered = fate_neural_decrypt(&ciphertext, key, nonce, &control);

    assert_eq!(recovered, plaintext);

    println!("FateYakshaControl: mode={:?}, breaths={}, label={}",
        control.mode, control.breaths, control.label);
    println!("Ciphertext length: {}", ciphertext.len());
}

