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
//! - **Avalanche effect**: ~50% (near-optimal)
//! - **Key sensitivity**: ~50%
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
pub mod config;
pub mod core;
pub mod parametric;
pub mod neural;
pub mod aead;
pub mod cascade;
pub mod frac_container;
pub mod ocean_builtin;
pub mod ocean_pipeline;
pub mod shell;
pub mod mac_fast;

pub use ocean_pipeline::{encrypt_with_ocean, decrypt_with_ocean, primitive_session_anchor};

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
pub use config::{MacMode, ThroughputProfile};
pub use mac_fast::YakshaMacFast;
pub use geometric::{
    encrypt as geometric_encrypt,
    decrypt as geometric_decrypt,
    GeometricSpace,
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
