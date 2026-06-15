# Yaksha Cipher GPU Native Adaptability (Ocean XOR + Fate Evolve)

## Goal
Implement the “ocean interference source” (Ocean) as GPU-accelerated *XOR mask application* while keeping PRG/key-stream generation on CPU (so CPU-side additions remain CPU-only). The GPU kernel performs only bitwise XOR.

## What is accelerated on GPU
1. **Ocean XOR application**: `dst[i] = src[i] ^ mask[i]` via `wgpu` compute shader.
2. **Dimension / Quantum base transforms**: GPU computes container parameters (membrane/density/connectivity) and then runs the existing GPU-backed encryption/decryption logic.
3. **Fate evolve cascade breaths** remain CPU-based for correctness (they call `yaksha_cipher::cascade`).

## What stays on CPU (by design)
1. **PRG generation (key-stream)** for Ocean disturbance:
   - Uses `SimplePrg` LCG-like update with additions.
   - The generated mask bytes are produced on CPU.
2. **Fate-controlled cascade breaths**:
   - `cascade_encrypt` / `cascade_decrypt` are CPU reversible operations.

## Encryption pipeline (GPU Fate Neural with Ocean XOR)
Entry point: `gpu_fate_neural_encrypt(...)` in `src/fate_evolve_gpu.rs`.

Given `(plaintext, master_key, nonce)` and a GPU context `ctx`:
1. **KDF + control derivation (CPU)**:
   - `YakshaKdf::derive_key(nonce)` (session key)
   - `FateYakshaControl::from_fate(&mut kdf)`
2. **Phase 1 Ocean disturbance (CPU PRG + GPU XOR apply)**:
   - Generate `mask1[i]` with PRG phase `1`
   - Apply XOR on GPU using `apply_xor_mask(ctx, plaintext, mask1)`
3. **Base transform chosen by Fate mode** (GPU base):
   - `Neural`: `encrypt_quantum(ctx, phase1_input, master_key)`
   - `Parametric`: `encrypt_dimension(ctx, phase1_input, master_key)`
   - Mixed modes run the two transforms in the same order as `yaksha-cipher` Fate logic.
4. **Phase 2 Ocean disturbance (CPU PRG + GPU XOR apply)**:
   - Generate `mask2[i]` with PRG phase `2`
   - Apply XOR on GPU.
5. **Fate-controlled cascade breaths (CPU reversible)**:
   - Apply `cascade_encrypt/decrypt` as specified by `FateYakshaControl`.
6. **Return** `(ciphertext, control)`.

## Decryption pipeline (inverse of encryption)
Entry point: `gpu_fate_neural_decrypt(...)`:
1. **Undo cascade breaths** (CPU inverse order).
2. **Remove Phase 2 Ocean disturbance** with the same PRG phase `2` mask, applied via GPU XOR.
3. **Invert base transform** via GPU:
   - `decrypt_quantum` / `decrypt_dimension` (and mixed inverses in reverse order).
4. **Remove Phase 1 Ocean disturbance** via PRG phase `1` and GPU XOR.
5. **Return plaintext**.

## GPU XOR kernel details
Kernel: `src/xor_mask.rs` (`xor_mask` WGSL compute shader)
```text
dst[idx] = src[idx] ^ mask[idx]
```
Implementation notes:
- Packs input bytes into `u32` little-endian words for GPU buffers.
- Pads to 4-byte alignment; truncates back to original length after read-back.
- This kernel performs XOR-only operations (no addition on GPU).

## Testing that was run
1. `cargo run --release --bin gpu_test`
   - Verifies dimension/quantum roundtrip and baseline correctness.
2. `cargo run --release --bin gpu_fate_neural_test`
   - Verifies end-to-end GPU Fate Neural encrypt->decrypt roundtrip while Ocean XOR is applied on GPU.

## “GPU native adaptability” conclusion
Ocean XOR is the cleanest GPU-native slot because it is:
- embarrassingly parallel (byte-wise XOR),
- reversible by construction,
- and expressible as XOR-only GPU compute (no PRG additions on GPU).

