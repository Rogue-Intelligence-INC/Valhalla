//! # Yaksha Cipher GPU
//!
//! GPU offload is limited to **2D ocean XOR** (mask built on CPU, XOR on GPU via wgpu).
//! All cipher bodies, cascade, and Fate breaths remain on CPU — single source of truth
//! in `yaksha-cipher`.
//!
//! ```text
//! encrypt (GPU path):  Ocean₁ [GPU XOR] → CPU body → Ocean₂ [GPU XOR]
//! fate evolve:         same + CPU cascade breaths
//! ```
//!
//! Enable `experimental` feature for legacy research shaders (not used in production).

pub mod gpu_context;
pub mod dimension_gpu;
pub mod quantum_gpu;
pub mod xor_mask;
pub mod ocean_gpu;
pub mod shell_gpu;
pub mod fate_evolve_gpu;

#[cfg(feature = "experimental")]
pub mod vector_link;
#[cfg(feature = "experimental")]
pub mod random_cascade;
#[cfg(feature = "experimental")]
pub mod superposition_cascade;
#[cfg(feature = "experimental")]
pub mod interference_cascade;
#[cfg(feature = "experimental")]
pub mod wave_particle;
#[cfg(feature = "experimental")]
pub mod superposition;

pub use gpu_context::GpuContext;

pub use shell_gpu::{
    decrypt_neural, decrypt_parametric, encrypt_neural, encrypt_parametric,
};

pub use dimension_gpu::{decrypt as decrypt_parametric_body, encrypt as encrypt_parametric_body};
pub use quantum_gpu::{decrypt as decrypt_neural_body, encrypt as encrypt_neural_body};

pub use shell_gpu::{
    decrypt_neural as decrypt_quantum,
    decrypt_parametric as decrypt_dimension,
    encrypt_neural as encrypt_quantum,
    encrypt_parametric as encrypt_dimension,
};

pub use xor_mask::apply_xor_mask;
pub use ocean_gpu::{apply_ocean_phase_gpu, build_ocean_mask};
