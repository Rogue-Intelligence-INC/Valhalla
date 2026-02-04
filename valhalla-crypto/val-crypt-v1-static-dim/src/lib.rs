//! # Val-Crypt: Dual-Mode Symmetric Encryptions
//!
//! Two reversible encryption algorithms: ARX block cipher and trajectory stream cipher.
//!
//! ## Algorithms
//!
//! | Name | Type | Speed | Avalanche |
//! |------|------|-------|-----------|
//! | **Vector** | ARX Block Cipher | ~885 MB/s | 50.7% |
//! | **Entropy** | Trajectory Stream Cipher | ~888 MB/s | 53.6% |
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use val_crypt::{vector, entropy};
//!
//! let key = b"MySecretKey12345";
//! let data = b"Hello, Valhalla!";
//!
//! // Vector mode (ARX-based)
//! let encrypted = vector::encrypt(data, key);
//! let decrypted = vector::decrypt(&encrypted, key);
//!
//! // Entropy mode (trajectory-based)
//! let encrypted = entropy::encrypt(data, key);
//! let decrypted = entropy::decrypt(&encrypted, key);
//! ```

pub mod vector;
pub mod entropy;

// Re-export for convenience
pub use vector::{encrypt as vector_encrypt, decrypt as vector_decrypt};
pub use entropy::{encrypt as entropy_encrypt, decrypt as entropy_decrypt};
