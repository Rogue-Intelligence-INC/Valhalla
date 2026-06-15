//! # Yaksha Cipher
//!
//! A symmetric encryption framework with parameterized transformations and stateful AEAD.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Yaksha Cipher Protocol                   │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Stateful Key Derivation (KDF)     ← Session differentiation│
//! │           ↓                                                 │
//! │  Yaksha Encryption (deterministic) ← Core cipher primitive  │
//! │           ↓                                                 │
//! │  Yaksha MAC (authentication)       ← Integrity tag          │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Modules
//!
//! | Module | Description | Use Case |
//! |--------|-------------|----------|
//! | `core` | Basic encryption primitive | Simple encrypt/decrypt |
//! | `parametric` | Parameterized transformation | Key-derived parameters |
//! | `neural` | Neural-inspired transformation | MeshBrain-based |
//! | `aead` | Authenticated encryption | Full AEAD protocol |
//!
//! ## Security Properties
//!
//! - **Validation**: No in-crate statistical unit tests — run the **industry harness**
//!   from repo root: `./external_bench/run_all.sh` (OpenSSL `speed`, **dieharder** on
//!   generated ciphertext streams, AEAD bench, timing heuristic).
//! - **Deterministic primitive**: Same key + plaintext = same ciphertext
//! - **Protocol-level differentiation**: Stateful KDF provides session uniqueness
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use yaksha_cipher::{parametric, neural, aead};
//!
//! let key = b"secret_key_1234!";
//! let plaintext = b"Hello, World!";
//!
//! // Parametric mode (key-derived transformation parameters)
//! let ciphertext = parametric::encrypt(plaintext, key);
//! let decrypted = parametric::decrypt(&ciphertext, key);
//!
//! // Neural mode (MeshBrain-inspired transformation)
//! let ciphertext = neural::encrypt(plaintext, key);
//! let decrypted = neural::decrypt(&ciphertext, key);
//!
//! // AEAD mode (authenticated encryption)
//! let mut cipher = aead::YakshaAead::new(key);
//! let (ciphertext, tag) = cipher.encrypt(plaintext, b"associated_data");
//! let decrypted = cipher.decrypt(&ciphertext, &tag, b"associated_data")?;
//! ```

// Core modules
pub mod core;
pub mod parametric;
pub mod neural;
pub mod aead;
pub mod cascade;
pub mod frac_container;
pub mod ocean_pipeline;
pub mod shell;

pub use cascade::{
    WAVE_BLOCK, FEISTEL_ROUNDS, PHASE1_FEISTEL_ROUNDS, FEISTEL_ROUNDS_PHASE2,
    wave_block_phase1_encrypt, wave_block_phase1_decrypt,
};
pub use frac_container::{FracDimContainer, OCEAN_LATTICE_DIM, YAKSHA_DIMENSION};
pub use ocean_pipeline::{decrypt_with_ocean, encrypt_with_ocean, primitive_session_anchor};
pub use shell::{core_body_decrypt, core_body_encrypt, decrypt as shell_decrypt, decrypt_primitive as shell_decrypt_primitive, encrypt as shell_encrypt, encrypt_primitive as shell_encrypt_primitive};
pub mod evolve;
pub mod ocean_builtin;
pub use ocean_builtin::{
    fnv1a64, ocean_apply_inplace, ocean_phase, ocean_seed, ocean_stream_byte,
    ocean_stream_byte_2d, OceanMaskContext, OCEAN_MIX_ROUNDS, OCEAN_MIX_ROUNDS_MAX,
    OCEAN_MIX_ROUNDS_MIN,
};

// Legacy/experimental
pub mod vector;
pub mod geometric;

// Re-exports for convenience
pub use core::{encrypt as core_encrypt, decrypt as core_decrypt};
pub use parametric::{
    encrypt as parametric_encrypt,
    decrypt as parametric_decrypt,
    ParametricSpace,
};
pub use neural::{
    encrypt as neural_encrypt,
    decrypt as neural_decrypt,
    NeuralState,
    SignalType,
};
pub use aead::{
    YakshaAead,
    YakshaKdf,
    YakshaMac,
    SignalStructure,
    SignalDomain,
};
pub use geometric::{
    encrypt as geometric_encrypt,
    decrypt as geometric_decrypt,
    GeometricSpace,
};

// Experimental / evolve API re-exports
pub use evolve::{
    fate_neural_encrypt,
    fate_neural_encrypt_with_control,
    fate_neural_decrypt,
    fate_decide_for,
    FateYakshaControl,
    FateMode,
    CascadePattern,
    Scenario,
    DataTypeHint,
    FateDecision,
    FateRuleTemplate,
};

// Backward compatibility aliases
pub mod entropy {
    //! Backward compatibility alias for `core` module
    pub use crate::core::*;
}

pub mod fate_space {
    //! Backward compatibility alias for `parametric` module
    pub use crate::parametric::*;
    pub use crate::parametric::encrypt as encrypt_entropy;
    pub use crate::parametric::decrypt as decrypt_entropy;
}

pub mod quantum {
    //! Backward compatibility alias for `neural` module
    pub use crate::neural::*;
}

pub mod fate_mac {
    //! Backward compatibility alias for `aead` module
    pub use crate::aead::*;
    pub use crate::aead::YakshaAead as FateAead;
    pub use crate::aead::YakshaKdf as DynamicKeyGenerator;
    pub use crate::aead::YakshaMac as FateMac;
}
