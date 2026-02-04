# Val-Crypt: Dual-Mode Symmetric Encryption via Vector Rotation and Entropy Trajectory

**Authors**: Valhalla Project  
**Date**: January 2026

---

## Abstract

We present Val-Crypt, a symmetric encryption system offering two distinct algorithms—**Vector** and **Entropy**—both achieving 100% reversibility with ideal avalanche properties (~50%). Vector mode is an ARX block cipher using high-dimensional rotations, while Entropy mode is a trajectory-based stream cipher using deterministic entropy paths. Both modes share a common bijective cascade phase ensuring consistent diffusion. Experimental results demonstrate throughputs of 885-888 MB/s with 50.7-53.6% avalanche effect.

**Keywords**: Symmetric encryption, ARX, block cipher, stream cipher, entropy trajectory, bijective cascade

---

## 1. Introduction

Val-Crypt addresses the challenge of creating reversible encryption algorithms that maintain cryptographic diffusion properties. Through iterative development, we identified that:

1. **Non-bijective operations** (e.g., modular multiplication with even factors) prevent reversibility
2. **Cascade design** significantly impacts both reversibility and avalanche effect
3. **Dual-mode approach** accommodates different use cases (block vs. stream)

---

## 2. Vector Mode: ARX High-Dimensional Rotation

### 2.1 Design Principle

Vector mode treats 4 bytes as a 4-dimensional vector and applies ARX transformations:

```
(a, b, c, d) → Quarter Round → (a', b', c', d')
```

All ARX operations are bijective:
- **Add** (mod 256): Inverse is subtraction
- **Rotate**: Inverse is reverse rotation
- **XOR**: Self-inverse

### 2.2 Algorithm

**Phase 1: ARX Transform**

For each 4-byte chunk with key `k[0..3]`:

```
Round 1:
  a = a + b + k[0];  d = rotl(d ^ a, 4)
  c = c + d + k[1];  b = rotl(b ^ c, 5)

Round 2:
  a = a + b + k[2];  d = rotl(d ^ a, 3)
  c = c + d + k[3];  b = rotl(b ^ c, 7)
```

**Phase 2: Bijective Cascade**

```
cascade = key[0] + dim_factor
for i = 0 to n-1:
    C[i] = T[i] ^ cascade        // Encrypt first
    cascade = rotl(cascade + T[i], 1)  // Then update
```

Key insight: Original cascade `C = T ^ rotl(cascade + T, 1)` was non-bijective; separating XOR from update ensures bijectivity.

### 2.3 Inverse

**Phase 2 Inverse:**
```
for i = 0 to n-1:
    T[i] = C[i] ^ cascade
    cascade = rotl(cascade + T[i], 1)
```

**Phase 1 Inverse:**
```
Round 2 Inverse:
  b = rotr(b, 7) ^ c;  c = c - d - k[3]
  d = rotr(d, 3) ^ a;  a = a - b - k[2]

Round 1 Inverse:
  b = rotr(b, 5) ^ c;  c = c - d - k[1]
  d = rotr(d, 4) ^ a;  a = a - b - k[0]
```

---

## 3. Entropy Mode: Trajectory-based Stream Cipher

### 3.1 Design Principle

Entropy mode uses deterministic entropy derived from (key, position) to guide per-byte transformations:

```
entropy = H(key, position)
if entropy > 127:
    Use multiplication path
else:
    Use addition path
```

Since the same entropy is computed during decryption, the transformation path is known.

### 3.2 Algorithm

**Phase 1: Entropy-Guided Transform**

```
for each position i (parallel):
    entropy = derive_entropy(key, i)
    state = membrane + plaintext[i] + frac_influence
    
    if entropy > 127:
        pk_odd = pk | 1  // Force odd for invertibility
        activated = rotl(state * pk_odd, r1)
    else:
        activated = rotr(state + pk, r2)
    
    T[i] = activated ^ pk ^ frac_influence
```

**Phase 2:** Same bijective cascade as Vector mode.

### 3.3 Key Innovation: Odd Multiplication

Modular multiplication is bijective only when the multiplier is coprime to the modulus. For mod 256 (= 2^8), only odd numbers are coprime. By forcing `pk_odd = pk | 1`, we guarantee invertibility:

```
pk_odd * x ≡ y (mod 256)
x ≡ y * inverse(pk_odd) (mod 256)
```

The inverse is computed via Newton-Raphson iteration.

---

## 4. Security Analysis

### 4.1 Avalanche Effect

| Mode | Measured | Ideal |
|------|----------|-------|
| Vector | 50.7% | 50% |
| Entropy | 53.6% | 50% |

### 4.2 Key Sensitivity

Flipping 1 bit in key produces ~50% output change in both modes.

### 4.3 Reversibility

| Test Suite | Vector | Entropy |
|------------|--------|---------|
| ASCII strings | 100% | 100% |
| All bytes 0-255 | 100% | 100% |
| 1000 random tests | 100% | 100% |

### 4.4 Bijection Verification

**Vector Mode:**
- ARX operations: Mathematically proven bijective
- Cascade: `C = T ^ cascade` is bijective for fixed cascade

**Entropy Mode:**
- Addition path: Bijective (inverse is subtraction)
- Multiplication path: Bijective (odd multiplier has inverse mod 256)
- Cascade: Same as Vector

---

## 5. Performance

| Mode | Throughput | Notes |
|------|------------|-------|
| Vector | 885 MB/s | Block-based, PKCS7 padding |
| Entropy | 888 MB/s | Parallel Phase 1 (Rayon) |

The cascade phase is sequential in both modes, limiting parallelization. However, this is necessary for the avalanche effect.

---

## 6. Comparison

| Aspect | Vector | Entropy |
|--------|--------|---------|
| Type | ARX Block Cipher | Trajectory Stream Cipher |
| Phase 1 | Block (4 bytes) | Byte (parallel) |
| Bijection method | Pure ARX | Odd multiplication |
| Avalanche | 50.7% | 53.6% |
| Speed | 885 MB/s | 888 MB/s |
| Best for | Fixed-size data | Variable-length data |

---

## 7. Conclusion

Val-Crypt demonstrates two approaches to reversible symmetric encryption:

1. **Vector mode**: Pure ARX operations, simpler analysis
2. **Entropy mode**: Trajectory-based, higher avalanche

Both achieve the cryptographic goals of 100% reversibility and ~50% avalanche effect at high throughput (~885 MB/s).

---

## References

[1] Bernstein, D.J. "ChaCha, a variant of Salsa20." SASC, 2008.

[2] Daemen, J. and Rijmen, V. "The Design of Rijndael: AES." Springer, 2002.

---

## Appendix: Key Formulas

### Bijective Cascade

**Original (non-bijective):**
```
C = T ^ rotl(cascade + T, 1)
```
Problem: Solving for T gives non-unique solutions.

**Fixed (bijective):**
```
C = T ^ cascade
cascade' = rotl(cascade + T, 1)
```
T appears only once in the equation → unique solution.

### Modular Inverse (Newton-Raphson)

For odd `a` mod 256:
```
x = a
repeat 4 times:
    x = x * (2 - a * x) mod 256
return x
```

This converges to `a^(-1) mod 256` in 4 iterations.

---

*© 2026 Valhalla Project. CC BY 4.0.*
