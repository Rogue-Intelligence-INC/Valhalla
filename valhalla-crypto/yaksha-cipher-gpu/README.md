# Val-Crypt GPU

GPU-accelerated version of Val-Crypt encryption using wgpu/Vulkan.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                 Val-Crypt GPU Hybrid                    │
├─────────────────────────────────────────────────────────┤
│                                                         │
│   GPU (wgpu/Vulkan)         CPU (rayon parallel)        │
│  ┌─────────────────┐       ┌─────────────────────────┐  │
│  │  Container      │       │  Phase 1: Transform     │  │
│  │  Parameters     │──────▶│  (parallel per-byte)    │  │
│  │  Computation    │       └─────────────────────────┘  │
│  └─────────────────┘                 │                  │
│                                      ▼                  │
│                            ┌─────────────────────────┐  │
│                            │  Phase 2: Cascade       │  │
│                            │  (sequential, avalanche)│  │
│                            └─────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Features

- **GPU-accelerated container computation**: Dimension/Quantum parameters computed on GPU
- **CPU parallel transformation**: Phase 1 uses rayon for multi-core parallelism
- **Sequential cascade**: Phase 2 ensures 50% avalanche effect
- **Automatic fallback**: Small data (<4KB) uses pure CPU implementation

## Performance

Tested on NVIDIA RTX 4090:

### Dynamic Dimension Mode

| Size | CPU | GPU | Speedup |
|------|-----|-----|---------|
| 256 KB | 237 MB/s | 319 MB/s | 1.35x |
| 1 MB | 381 MB/s | 960 MB/s | **2.52x** |
| 4 MB | 518 MB/s | 1097 MB/s | **2.12x** |
| 16 MB | 625 MB/s | 1404 MB/s | **2.25x** |

### Quantum Simulation Mode

| Size | CPU | GPU | Speedup |
|------|-----|-----|---------|
| 256 KB | 593 MB/s | 440 MB/s | 0.74x |
| 1 MB | 659 MB/s | 913 MB/s | **1.38x** |
| 4 MB | 724 MB/s | 1060 MB/s | **1.46x** |
| 16 MB | 678 MB/s | 1288 MB/s | **1.90x** |

**Peak throughput: 1.4 GB/s**

## Usage

```rust
use val_crypt_gpu::{GpuContext, encrypt_dimension, decrypt_dimension};

// Initialize GPU context
let ctx = GpuContext::new().expect("Failed to initialize GPU");

// Encrypt
let plaintext = b"Hello, World!";
let key = b"secret_key_1234";
let ciphertext = encrypt_dimension(&ctx, plaintext, key);

// Decrypt
let decrypted = decrypt_dimension(&ctx, &ciphertext, key);
assert_eq!(plaintext.to_vec(), decrypted);
```

## Requirements

- Rust 1.70+
- Vulkan-capable GPU (or Metal on macOS, DX12 on Windows)
- wgpu 0.19

## Security

Both GPU-accelerated modes maintain full security properties:

- **Avalanche Effect**: ~50% bit change on single input bit flip
- **Key Sensitivity**: ~50% bit difference for adjacent keys
- **Correctness**: Verified encrypt-decrypt round-trip for all data sizes

## License

MIT
