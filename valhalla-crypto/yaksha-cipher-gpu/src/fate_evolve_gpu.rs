//! GPU-backed Fate evolve (ocean disturbance XOR applied on GPU).
//!
//! Mirrors `yaksha-cipher/src/evolve.rs`: ocean phases use GPU XOR; bodies and cascade on CPU.

use yaksha_cipher::aead::YakshaKdf;
use yaksha_cipher::frac_container::FracDimContainer;
use yaksha_cipher::ocean_builtin::ocean_phase;
use yaksha_cipher::parametric;
use yaksha_cipher::neural;
use yaksha_cipher::{CascadePattern, FateMode, FateYakshaControl};

use yaksha_cipher::cascade;
use crate::ocean_gpu::apply_ocean_phase_gpu;
use crate::GpuContext;

pub fn gpu_fate_neural_encrypt(
    plaintext: &[u8],
    master_key: &[u8],
    nonce: &[u8],
    ctx: &GpuContext,
) -> (Vec<u8>, FateYakshaControl) {
    if plaintext.is_empty() || master_key.is_empty() {
        return (plaintext.to_vec(), FateYakshaControl::default());
    }

    let container = FracDimContainer::standard();

    let mut kdf = YakshaKdf::new(master_key);
    let _session_key = kdf.derive_key(nonce);
    let control = FateYakshaControl::from_fate(&mut kdf);

    let phase1_input = apply_ocean_phase_gpu(
        ctx,
        plaintext,
        master_key,
        nonce,
        ocean_phase::ONE,
        container,
    );

    let mut buf = match control.mode {
        FateMode::Neural => neural::encrypt_body(&phase1_input, master_key),
        FateMode::Parametric => parametric::encrypt_entropy_body(&phase1_input, master_key),
        FateMode::NeuralThenParametric => {
            let tmp = neural::encrypt_body(&phase1_input, master_key);
            parametric::encrypt_entropy_body(&tmp, master_key)
        }
        FateMode::ParametricThenNeural => {
            let tmp = parametric::encrypt_entropy_body(&phase1_input, master_key);
            neural::encrypt_body(&tmp, master_key)
        }
    };

    buf = apply_ocean_phase_gpu(
        ctx,
        &buf,
        master_key,
        nonce,
        ocean_phase::TWO,
        container,
    );

    apply_fate_cascade(&mut buf, master_key, &control);

    (buf, control)
}

pub fn gpu_fate_neural_decrypt(
    ciphertext: &[u8],
    master_key: &[u8],
    nonce: &[u8],
    control: &FateYakshaControl,
    ctx: &GpuContext,
) -> Vec<u8> {
    if ciphertext.is_empty() || master_key.is_empty() {
        return ciphertext.to_vec();
    }

    let container = FracDimContainer::standard();

    let mut buf = ciphertext.to_vec();
    apply_fate_cascade_inverse(&mut buf, master_key, control);

    buf = apply_ocean_phase_gpu(
        ctx,
        &buf,
        master_key,
        nonce,
        ocean_phase::TWO,
        container,
    );

    let plain = match control.mode {
        FateMode::Neural => neural::decrypt_body(&buf, master_key),
        FateMode::Parametric => parametric::decrypt_entropy_body(&buf, master_key),
        FateMode::NeuralThenParametric => {
            let tmp = parametric::decrypt_entropy_body(&buf, master_key);
            neural::decrypt_body(&tmp, master_key)
        }
        FateMode::ParametricThenNeural => {
            let tmp = neural::decrypt_body(&buf, master_key);
            parametric::decrypt_entropy_body(&tmp, master_key)
        }
    };

    apply_ocean_phase_gpu(
        ctx,
        &plain,
        master_key,
        nonce,
        ocean_phase::ONE,
        container,
    )
}

fn apply_fate_cascade(buf: &mut [u8], key: &[u8], control: &FateYakshaControl) {
    let breaths = control.breaths.max(1);
    match control.cascade_pattern {
        CascadePattern::ForwardFeistel => {
            for _ in 0..breaths {
                cascade::cascade_encrypt(buf, key);
            }
        }
        CascadePattern::Bidirectional => {
            for _ in 0..breaths {
                cascade::cascade_encrypt(buf, key);
                cascade::cascade_decrypt(buf, key);
            }
        }
        CascadePattern::MultiRound(extra) => {
            let total = breaths.saturating_add(extra).max(2);
            for _ in 0..total {
                cascade::cascade_encrypt(buf, key);
            }
        }
    }
}

fn apply_fate_cascade_inverse(buf: &mut [u8], key: &[u8], control: &FateYakshaControl) {
    let breaths = control.breaths.max(1);
    match control.cascade_pattern {
        CascadePattern::ForwardFeistel => {
            for _ in 0..breaths {
                cascade::cascade_decrypt(buf, key);
            }
        }
        CascadePattern::Bidirectional => {
            for _ in 0..breaths {
                cascade::cascade_encrypt(buf, key);
                cascade::cascade_decrypt(buf, key);
            }
        }
        CascadePattern::MultiRound(extra) => {
            let total = breaths.saturating_add(extra).max(2);
            for _ in 0..total {
                cascade::cascade_decrypt(buf, key);
            }
        }
    }
}
