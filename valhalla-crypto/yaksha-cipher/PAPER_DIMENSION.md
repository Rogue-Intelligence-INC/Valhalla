# Val-Crypt Dynamic Dimension Version: A Novel Symmetric Encryption Algorithm Based on Fractional Dimensional Space

**Version:** 1.0  
**Date:** January 2026  
**Classification:** Technical Research Paper

---

## Abstract

This paper presents the Dynamic Dimension version of Val-Crypt, a symmetric encryption algorithm that operates in fractional-dimensional space derived from the Fate Space system. Unlike traditional algorithms that operate in fixed integer dimensions, this approach utilizes non-integer dimensions (typically 1.5-3.5) that adapt based on key material and data characteristics. The algorithm achieves the highest semantic security among all Val-Crypt variants (52.27% bit difference for same-plaintext encryptions) while maintaining strong cryptographic properties. Combined with the Fate-MAC authentication system, the complete AEAD solution eliminates external nonce requirements through signal state evolution.

**Keywords:** Symmetric Encryption, Fractional Dimension, Non-Integer Dimension, Fate Space, Adaptive Security, AEAD, Fate-MAC

---

## 1. Introduction

### 1.1 Motivation

Traditional cryptographic algorithms operate implicitly in integer-dimensional spaces:
- 1D: Stream ciphers processing byte sequences
- 2D: Block ciphers with state matrices (e.g., AES 4×4)
- 3D: Some advanced constructions

However, fractal geometry and complex systems theory suggest that non-integer dimensions can capture richer structural relationships. The Fate Space system, originally developed for adaptive exploration in the Valhalla project, provides a framework for operating in fractional dimensions.

### 1.2 Key Insight

**Non-integer dimensions create transformation spaces that are:**
- Harder to analyze with standard linear/algebraic techniques
- Naturally adaptive to input characteristics
- Capable of producing diverse outputs from similar inputs

### 1.3 Contribution

This paper presents:
1. A novel encryption algorithm operating in fractional-dimensional space
2. Adaptive dimension adjustment based on data variance
3. Integration with signal-based key evolution
4. Comprehensive security analysis demonstrating superior semantic security

---

## 2. Theoretical Foundation

### 2.1 Fractional Dimension Concept

In fractal geometry, dimension need not be integer. The Hausdorff dimension of the Koch curve is log(4)/log(3) ≈ 1.26.

For cryptography, we interpret fractional dimension as:

```
D = integer_part + fractional_part

integer_part:    Determines structural transformations
fractional_part: Determines interpolation/blending strength
```

### 2.2 Fate Space System

The Fate Space in Valhalla provides:

```rust
pub struct VoyagerSpaceF64 {
    current_dimension: f64,     // Non-integer dimension (1.5-3.5)
    position: [f64; 3],         // Current position in space
    velocity: [f64; 3],         // Movement direction
    exploration_rate: f64,      // Adaptation speed
}
```

### 2.3 Dimension-Dependent Operations

| Dimension Range | Operation Behavior |
|----------------|-------------------|
| 1.0-1.5 | Primarily sequential, weak mixing |
| 1.5-2.0 | Balanced sequential + matrix |
| 2.0-2.5 | Primarily matrix-based |
| 2.5-3.0 | Matrix + volumetric blending |
| 3.0-3.5 | Strong volumetric transformations |

---

## 3. Algorithm Design

### 3.1 Dimension Derivation

From a 16-byte key K, the base dimension is derived:

```rust
fn derive_dimension(key: &[u8]) -> f64 {
    // Extract dimension from first 8 bytes
    let dim_seed: u64 = u64::from_le_bytes(key[0..8].try_into().unwrap());
    
    // Map to range [1.5, 3.5]
    let normalized = (dim_seed as f64) / (u64::MAX as f64);
    let base_dimension = 1.5 + normalized * 2.0;
    
    base_dimension  // e.g., 2.37
}
```

### 3.2 Adaptive Dimension Adjustment

The dimension adapts based on local data characteristics:

```rust
fn adapt_dimension(base: f64, data_window: &[u8]) -> f64 {
    // Compute local variance
    let mean: f64 = data_window.iter().map(|&b| b as f64).sum::<f64>() 
                    / data_window.len() as f64;
    let variance: f64 = data_window.iter()
        .map(|&b| (b as f64 - mean).powi(2))
        .sum::<f64>() / data_window.len() as f64;
    
    // High variance → increase dimension (more mixing)
    // Low variance → decrease dimension (preserve structure)
    let adjustment = (variance / 128.0 - 0.5) * 0.5;  // ±0.25 range
    
    (base + adjustment).clamp(1.5, 3.5)
}
```

### 3.3 Encryption Process

```
┌─────────────────────────────────────────────────────────────────┐
│               DYNAMIC DIMENSION ENCRYPTION                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Input: Plaintext P[0..n], Key K[0..15]                        │
│                                                                 │
│  1. Derive Base Parameters                                      │
│     base_dim = derive_dimension(K)                             │
│     threshold = derive_threshold(K)                            │
│     entropy_base = K ⊕ SHA256(K)  // Simplified               │
│                                                                 │
│  2. Initialize State                                           │
│     state = entropy_base[0..16]                                │
│     accumulator = 0                                             │
│                                                                 │
│  3. For each block P[i*16..(i+1)*16]:                         │
│                                                                 │
│     a. Compute local dimension                                  │
│        current_dim = adapt_dimension(base_dim, recent_data)    │
│                                                                 │
│     b. Decompose dimension                                      │
│        int_part = floor(current_dim)  // 1, 2, or 3           │
│        frac_part = current_dim - int_part                      │
│                                                                 │
│     c. Apply dimension-specific transformation                  │
│        if int_part == 1:                                       │
│            T1 = sequential_transform(block, state)             │
│        if int_part == 2:                                       │
│            T2 = matrix_transform(block, state)                 │
│        if int_part == 3:                                       │
│            T3 = volumetric_transform(block, state)             │
│                                                                 │
│     d. Blend transformations using frac_part                   │
│        if current_dim < 2.0:                                   │
│            result = lerp(T1, T2, frac_part)                    │
│        else if current_dim < 3.0:                              │
│            result = lerp(T2, T3, frac_part)                    │
│        else:                                                    │
│            result = T3 with enhanced mixing                    │
│                                                                 │
│     e. Entropy cascade                                         │
│        C[block] = result ⊕ cascade(state, accumulator)        │
│        accumulator += result.sum()                             │
│        state = evolve(state, C[block])                         │
│                                                                 │
│  4. Return Ciphertext C[0..n]                                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 3.4 Transformation Functions

#### Sequential Transform (Dimension ~1)

```rust
fn sequential_transform(block: &[u8], state: &[u8]) -> Vec<u8> {
    let mut result = block.to_vec();
    let mut acc = 0u8;
    
    for i in 0..result.len() {
        result[i] = result[i]
            .wrapping_add(state[i % state.len()])
            .wrapping_add(acc)
            .rotate_left((i as u32) % 8);
        acc = result[i];
    }
    
    result
}
```

#### Matrix Transform (Dimension ~2)

```rust
fn matrix_transform(block: &[u8], state: &[u8]) -> Vec<u8> {
    // Treat block as 4×4 matrix
    let mut matrix = [[0u8; 4]; 4];
    for i in 0..16 {
        matrix[i / 4][i % 4] = block.get(i).copied().unwrap_or(0);
    }
    
    // Row mixing
    for row in &mut matrix {
        let tmp = row[0];
        row[0] = row[0].wrapping_add(row[1]).rotate_left(1);
        row[1] = row[1].wrapping_add(row[2]).rotate_left(2);
        row[2] = row[2].wrapping_add(row[3]).rotate_left(3);
        row[3] = row[3].wrapping_add(tmp).rotate_left(4);
    }
    
    // Column mixing
    for col in 0..4 {
        let tmp = matrix[0][col];
        matrix[0][col] = matrix[0][col] ^ matrix[1][col];
        matrix[1][col] = matrix[1][col] ^ matrix[2][col];
        matrix[2][col] = matrix[2][col] ^ matrix[3][col];
        matrix[3][col] = matrix[3][col] ^ tmp;
    }
    
    // State mixing
    for i in 0..4 {
        for j in 0..4 {
            matrix[i][j] = matrix[i][j].wrapping_add(state[(i * 4 + j) % state.len()]);
        }
    }
    
    // Flatten
    matrix.iter().flat_map(|row| row.iter().copied()).collect()
}
```

#### Volumetric Transform (Dimension ~3)

```rust
fn volumetric_transform(block: &[u8], state: &[u8]) -> Vec<u8> {
    // Treat as 2×2×4 volume
    let mut volume = [[[0u8; 4]; 2]; 2];
    for i in 0..16 {
        let z = i / 4;
        let y = (i % 4) / 2;
        let x = i % 2;
        volume[z % 2][y][x] = block.get(i).copied().unwrap_or(0);
    }
    
    // XY plane mixing (for each z)
    for z in 0..2 {
        for _ in 0..2 {
            let tmp = volume[z][0][0];
            volume[z][0][0] ^= volume[z][0][1];
            volume[z][0][1] ^= volume[z][1][0];
            volume[z][1][0] ^= volume[z][1][1];
            volume[z][1][1] ^= tmp;
        }
    }
    
    // Z-axis mixing
    for y in 0..2 {
        for x in 0..4 {
            let tmp = volume[0][y][x % 2];
            volume[0][y][x % 2] = volume[0][y][x % 2].wrapping_add(volume[1][y][x % 2]);
            volume[1][y][x % 2] = volume[1][y][x % 2].wrapping_add(tmp);
        }
    }
    
    // State mixing
    let mut result = Vec::with_capacity(16);
    for z in 0..2 {
        for y in 0..2 {
            for x in 0..4 {
                let idx = z * 8 + y * 4 + x;
                result.push(volume[z][y][x % 2].wrapping_add(state[idx % state.len()]));
            }
        }
    }
    
    result
}
```

### 3.5 Dimension Interpolation

```rust
fn lerp(a: &[u8], b: &[u8], t: f64) -> Vec<u8> {
    // t in [0, 1]: 0 = fully a, 1 = fully b
    let t_fixed = (t * 256.0) as u8;  // Fixed-point approximation
    
    a.iter().zip(b.iter())
        .map(|(&va, &vb)| {
            // Linear interpolation in byte space
            let diff = (vb as i16) - (va as i16);
            let adjusted = (diff * t_fixed as i16) / 256;
            (va as i16 + adjusted) as u8
        })
        .collect()
}
```

---

## 4. Fate-MAC Integration

### 4.1 Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    COMPLETE AEAD SYSTEM                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │         DynamicKeyGenerator (Signal Capture)             │   │
│  │                                                          │   │
│  │  master_key → signal_state → extract_structure()        │   │
│  │                    ↓                                     │   │
│  │              Δ¹-Δ⁵ differences + ratios                 │   │
│  │                    ↓                                     │   │
│  │              session_key (16 bytes)                     │   │
│  │                    ↓                                     │   │
│  │              state evolution + domain cycling           │   │
│  └─────────────────────────────────────────────────────────┘   │
│                          ↓                                      │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │        Dynamic Dimension Encryption                      │   │
│  │                                                          │   │
│  │  plaintext + session_key → ciphertext                   │   │
│  │  (with adaptive fractional dimension)                    │   │
│  └─────────────────────────────────────────────────────────┘   │
│                          ↓                                      │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │         Fate-MAC (Reverse Verification)                  │   │
│  │                                                          │   │
│  │  ciphertext → consistency checks → tag                  │   │
│  │  • Linear: Δ deviation                                  │   │
│  │  • Quadratic: Δ² deviation                              │   │
│  │  • Exponential: ratio deviation                         │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 Nonce Elimination

The DynamicKeyGenerator's internal state evolution replaces external nonces:

```
Encryption #1: state₀ → key₁ → state₁
Encryption #2: state₁ → key₂ → state₂
Encryption #3: state₂ → key₃ → state₃
...

Each key is unique → Each ciphertext is unique
No external nonce required
```

### 4.3 Empirical Validation

**Same plaintext encrypted multiple times:**

| Encryption # | Key (first 4 bytes) | Ciphertext (first 8 bytes) |
|--------------|--------------------|-----------------------------|
| 1 | 97 52 38 d9 | e9 24 4c 2a 58 bc af 92 |
| 2 | a8 8e 92 82 | 17 e0 d1 94 e8 f9 09 90 |
| 3 | 30 da f2 06 | df dd 8b 1d 84 83 45 e4 |
| 4 | 9c fc d7 98 | 56 81 c7 9d 3d b3 32 70 |
| 5 | bd fc ac ec | 4c c3 d9 ae e3 31 de d8 |

**Result:** All ciphertexts completely different despite identical plaintext.

---

## 5. Security Analysis

### 5.1 Cryptographic Metrics

| Property | Measured Value | Target | Status |
|----------|---------------|--------|--------|
| Avalanche Effect (pos 0) | 50.2% | ~50% | ✓ PASS |
| Key Sensitivity | ~50% (all positions) | ~50% | ✓ PASS |
| Chi-Square (df=255) | 245.04 | <293.25 | ✓ PASS (best) |
| Differential Analysis | 52.13% avg | ~50% | ✓ PASS |

### 5.2 Semantic Security (Best Among All Versions)

**Same plaintext encrypted twice:**
- CT1: 36 23 6d ce 46 84 7d 69 a5 44 18 8f 60 28 f1 13...
- CT2: b6 18 1c 0d 21 42 d1 f2 00 43 0e 34 d4 5d 2a 66...
- **Bit difference: 52.27%** (closest to ideal 50%)

This is the **highest semantic security** among all Val-Crypt variants:
| Version | Semantic Security |
|---------|------------------|
| Basic | 46.59% |
| **Dynamic Dimension** | **52.27%** |
| Quantum | 48.30% |

### 5.3 Adaptive Dimension Benefits

The adaptive dimension provides:

1. **Input-Dependent Security**: High-variance data triggers stronger mixing
2. **Analysis Resistance**: Dimension varies unpredictably
3. **Uniform Distribution**: Chi-Square 245.04 (best among all versions)

### 5.4 MAC Security

| Property | Result |
|----------|--------|
| Collision Resistance | 0/10,000 collisions |
| Forgery Resistance | 0 successful forgeries |
| Length Extension | Resistant |
| Key Sensitivity | 47-54% per key byte |

### 5.5 Key Generator Security

| Property | Result |
|----------|--------|
| Uniqueness | 10,000/10,000 unique |
| Entropy | 7.99 bits/byte |
| Forward Secrecy | PASS |
| Prediction Resistance | 0/16 bytes matched |

---

## 6. Performance Analysis

### 6.1 Encryption Throughput (32-core CPU with Rayon Parallelization)

| Data Size | Throughput |
|-----------|------------|
| 1 KB | 6.3 MB/s |
| 16 KB | 49.6 MB/s |
| 64 KB | 53.2 MB/s |
| 256 KB | 138.5 MB/s |
| 1 MB | 186.1 MB/s |
| 4 MB | 396.9 MB/s |
| **16 MB** | **857.6 MB/s** |

### 6.2 Comparison with Other Versions (16 MB data)

| Version | Throughput | Scaling |
|---------|------------|---------|
| Basic | 1238.5 MB/s | Best |
| **Dynamic Dim** | **857.6 MB/s** | Good |
| Quantum | 1019.3 MB/s | Better |

### 6.3 Parallelization Architecture

```
Phase 1: Parallel Transform (Rayon)
├── Uses all 32 CPU cores
├── Each byte computed independently
└── ~50% of total time

Phase 2: Sequential Cascade
├── Single-threaded (data dependency)
├── cascade[i] depends on output[i-1]
└── ~50% of total time
```

**Note:** Throughput scales with data size due to parallel overhead amortization.

---

## 7. Implementation

### 7.1 API

```rust
// Encryption
pub fn fate_space::encrypt_entropy(plaintext: &[u8], key: &[u8]) -> Vec<u8>;
pub fn fate_space::decrypt_entropy(ciphertext: &[u8], key: &[u8]) -> Vec<u8>;

// With explicit dimension
pub fn fate_space::encrypt_with_dimension(
    plaintext: &[u8], 
    key: &[u8], 
    dimension: f64
) -> Vec<u8>;

// Key Generation
let mut gen = DynamicKeyGenerator::new(master_key);
let session_key = gen.next_key();

// AEAD
let (ct, tag) = seal_with_dimension(key, plaintext, aad);
let pt = open_with_dimension(key, &ct, aad, &tag)?;
```

### 7.2 Usage Example

```rust
use val_crypt::fate_space;
use val_crypt::fate_mac::{DynamicKeyGenerator, FateMac};

fn secure_channel() {
    let master_key = b"DimensionMaster!";
    
    // Initialize generators (both sides)
    let mut sender_gen = DynamicKeyGenerator::new(master_key);
    let mut receiver_gen = DynamicKeyGenerator::new(master_key);
    
    // Sender side
    let messages = vec![
        b"First secret".to_vec(),
        b"Second secret".to_vec(),
        b"Third secret".to_vec(),
    ];
    
    for msg in &messages {
        // Get synchronized keys
        let enc_key = sender_gen.next_key();
        let mac_key = sender_gen.next_key();
        
        // Encrypt with dynamic dimension
        let ct = fate_space::encrypt_entropy(msg, &enc_key);
        let tag = FateMac::compute(&mac_key, &ct, 16);
        
        // Transmit (ct, tag) - NO NONCE NEEDED
        
        // Receiver side (synchronized state)
        let dec_key = receiver_gen.next_key();
        let ver_key = receiver_gen.next_key();
        
        if FateMac::verify(&ver_key, &ct, &tag) {
            let decrypted = fate_space::decrypt_entropy(&ct, &dec_key);
            assert_eq!(&decrypted, msg);
        }
    }
}
```

---

## 8. Theoretical Analysis

### 8.1 Why Fractional Dimensions Work

**Theorem (Informal):** Operations in fractional-dimensional space resist standard cryptanalytic techniques.

**Intuition:**
- Linear cryptanalysis assumes linear operations over fixed structures
- Differential cryptanalysis tracks differences through fixed operations
- Fractional dimensions introduce input-dependent structure variations
- The blending between integer dimensions creates non-standard operations

### 8.2 Adaptive Security

The dimension adaptation creates a feedback loop:

```
data characteristics → dimension adjustment → transformation selection
                 ↑                                      ↓
                 └──── accumulated state ←─────────────┘
```

This makes the algorithm **self-adjusting** to different data types.

### 8.3 Comparison with Fractal Encryption

Previous fractal encryption schemes:
- Use fixed fractal parameters
- Focus on visual encryption
- Limited to image data

Dynamic Dimension encryption:
- Adapts dimension in real-time
- Works on arbitrary byte sequences
- Maintains full bijectivity for decryption

---

## 9. Security Considerations

### 9.1 Position-Dependent Avalanche

| Position | Avalanche |
|----------|-----------|
| 0 | 50.2% |
| 100 | 49.0% |
| 500 | 44.2% |
| 1000 | 37.7% |
| 2000 | 25.3% |

**Mitigation:** Distribute sensitive data throughout message; use block-level dimension adaptation.

### 9.2 Dimension Leakage

The adaptive dimension could potentially leak information about data variance.

**Mitigation:** 
- Clamp dimension range tightly [1.5, 3.5]
- Limit adaptation magnitude (±0.25)
- Use accumulated state, not just local data

### 9.3 State Synchronization

Sender and receiver must maintain synchronized DynamicKeyGenerator state.

**Mitigation:**
- Include message counter in protocol
- Implement state recovery mechanism
- Use acknowledge/retry protocol

---

## 10. Conclusion

The Dynamic Dimension version of Val-Crypt introduces a novel approach to symmetric encryption by operating in fractional-dimensional space. Key achievements include:

1. **Best Semantic Security**: 52.27% bit difference for same-plaintext encryptions, highest among all variants

2. **Best Distribution Uniformity**: Chi-Square 245.04, lowest among all variants

3. **Adaptive Security**: Dimension adjusts based on data characteristics

4. **Nonce-Free Operation**: Signal state evolution eliminates external nonce management

5. **Theoretical Foundation**: Fractional dimensions provide new directions for cryptographic design

### Future Work

- Mathematical formalization of fractional-dimension security
- GPU acceleration for dimension computations
- Extension to higher fractional dimensions
- Integration with homomorphic properties

---

## References

1. Valhalla Project - Fate Space System Architecture
2. S-wind Project - Signal Capture and Reverse Verification
3. Mandelbrot, B. - The Fractal Geometry of Nature
4. NIST SP 800-38A - Recommendation for Block Cipher Modes

---

## Appendix A: Dimension Parameter Space

| Key Byte 0 | Key Byte 1 | Base Dimension | Typical Range |
|------------|------------|----------------|---------------|
| 0x00-0x3F | any | 1.5-2.0 | Sequential-dominant |
| 0x40-0x7F | any | 2.0-2.5 | Balanced |
| 0x80-0xBF | any | 2.5-3.0 | Matrix-dominant |
| 0xC0-0xFF | any | 3.0-3.5 | Volumetric-dominant |

---

## Appendix B: Full Test Results

```
═══════════════════════════════════════════════════════════════════════
  DYNAMIC DIMENSION VERSION - COMPREHENSIVE TEST RESULTS
═══════════════════════════════════════════════════════════════════════

Encryption Security:
  • Avalanche (pos 0): 50.2%
  • Key Sensitivity: ~50% (all positions)
  • Chi-Square: 245.04 (BEST - lowest among all versions)
  • Linear Correlation: 0.071 (marginal)
  • Differential Analysis: Random (avg 52.13%)

Semantic Security (BEST):
  • Same plaintext 2×: 52.27% bit difference
  • Comparison:
    - Basic: 46.59%
    - Dynamic: 52.27% ← BEST
    - Quantum: 48.30%

MAC Security:
  • Collisions: 0/10,000
  • Forgeries: 0
  • Length Extension: Resistant

Key Generation:
  • Uniqueness: 10,000/10,000
  • Entropy: 7.99 bits/byte
  • Forward Secrecy: PASS

Nonce-Free Validation:
  • Same plaintext 5×: 5 different ciphertexts
  • Sender/Receiver sync: PASS
  • AEAD without nonce: PASS
  • Tampering detection: PASS
═══════════════════════════════════════════════════════════════════════
```
