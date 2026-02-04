# S-wind: Formal Algebraic Exploration System over ℚ

**Version**: 1.1.1 (Final)  
**Generated**: 2026-01-24  
**Status**: Production Ready (Self-Loop + Auto-Archive + Meta-System)  
**License**: MIT

---

## Foundational Statement

> **All numeric computation in S-wind is defined over mathematically unbounded rational numbers (ℚ).**  
> Concrete constructors such as `from_i128` are provided solely as *implementation conveniences*, not as theoretical limits.

This positions S-wind not as a BigInt library, but as an **arithmetic model** operating natively over ℚ.

---

## Executive Summary

**S-wind** is a *formal exploration system*—not a computation framework. The distinction is fundamental:

| Computation Framework | Formal Exploration System (S-wind) |
|-----------------------|------------------------------------|
| Optimizes for speed | Optimizes for *verifiability* |
| Produces numeric results | Produces *auditable derivation traces* |
| Finds patterns by fitting | Identifies patterns by *structural recognition* |
| Success = fast convergence | Success = *verified formula or impossibility proof* |

### Epistemological Core

S-wind's architecture reflects an **epistemological** orientation:

| Concept | Role in S-wind |
|---------|----------------|
| **Reverse Verification** | Given result, identify which equation (cause) generated it |
| **Fate-Managed Verification** | Real algebraic pattern consistency, not fake self-consistency |
| **Discovery Criterion** | Formal: `(Result ≠ Known) ∧ (Verified) ∧ (Non-Trivial)` |
| **Negative Knowledge** | "Pattern class exhausted" is a valid, valuable output |

### Why This Matters

| Problem | Traditional Tools | S-wind Solution |
|---------|-------------------|-----------------|
| Floating-point errors | IEEE 754 rounding | ℚ arithmetic: zero error |
| "Discovery" inflation | No formal criterion | Strict Discovery Criterion |
| Black-box results | No derivation trace | Step-by-step auditable proofs |
| Fitting ≠ Discovery confusion | Unmarked | Rigorous labeling system |
| **Fake OOS Verification** | Self-consistency check | **Real algebraic verification** |

### Discovery Criterion

```
Discovery = (Result ≠ Known Authority) ∧ (Verified Correct) ∧ (Non-Trivial)

Where:
  Non-Trivial = (DoF ≥ 2)
  DoF = sample_size - (degree + 1)
  
  For polynomial of degree d, there are d+1 parameters (a₀, a₁, ..., a_d)
```

**Academic Statement**:
> *Fitting (including exact rational interpolation) is not counted as discovery. It is reported as an auditable hypothesis generator, and is only upgraded when it passes pre-registered out-of-sample verification within a constrained pattern class.*

---

## Part I: Architecture Overview

### System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         S-wind v1.1.1                           │
│                Formal Algebraic Exploration System              │
└─────────────────────────────────────────────────────────────────┘
                               │
       ┌───────────────────────┼───────────────────────┐
       │                       │                       │
       ▼                       ▼                       ▼
┌───────────────┐       ┌───────────────┐       ┌───────────────┐
│     Fate      │       │    Voyager    │       │     Quad      │
│ (Exploration) │◄─────►│ (Detection)   │       │  (Physics)    │
└───────┬───────┘       └───────────────┘       └───────────────┘
        │                                               │
        ▼                                               ▼
┌───────────────┐       ┌───────────────┐       ┌───────────────┐
│ KnowledgeBase │       │    Space      │       │     EPR       │
│  (199 seqs)   │       │ (Dimensions)  │       │  (Precision)  │
└───────────────┘       └───────────────┘       └───────────────┘
```

### Module Statistics

| Module | Lines of Code | Responsibility |
|--------|---------------|----------------|
| **Fate** | 3,524 | Core exploration engine, self-loop, verification management |
| **Voyager** | 1,863 | Pattern detection, reverse verification, derivation traces |
| **Knowledge** | 4,466 | 199 OEIS sequences, theorems, physical laws, pattern classes |
| **Symbolic (EPR)** | 2,097 | Unlimited precision rational arithmetic |
| **Quad** | 699 | Four physics simulation systems |
| **Space** | 427 | Fractional dimension management |
| **Total** | ~13,000+ | Complete formal exploration system |

### Module Dependency Graph

```
EPR (Foundation)
 └─► Rational
      └─► Voyager (Pattern Detection)
           └─► Fate (Exploration Engine)
                ├─► KnowledgeBase (Domain Knowledge)
                ├─► Space (Dimension Management)
                └─► Quad (Physics Simulation)
```

---

## Part II: Core Modules (Detailed)

### 2.1 EPR - Extended Precision Rational

**Source**: `src/symbolic/epr.rs` (737 lines)

#### Design Philosophy

```rust
// EPR Design Philosophy (from source):
// - Exact computation, never approximate
// - No floating point
// - Infinite precision
// - No external dependencies
```

#### Implementation

EPR uses `Vec<u64>` to store arbitrary large integers (base 2^64):

```rust
/// Extended precision integer using base 2^64
pub struct ExtendedInt {
    /// Number's limbs (base 2^64)
    /// limbs[0] = least significant, limbs[n-1] = most significant
    /// Example: 2^64 + 5 = [5, 1]
    limbs: Vec<u64>,
    /// Sign: true = negative
    negative: bool,
}

/// EPR = ExtendedInt / ExtendedInt
pub struct EPR {
    pub numerator: ExtendedInt,
    pub denominator: ExtendedInt,
}
```

#### Key Operations

| Operation | Implementation | Precision |
|-----------|----------------|-----------|
| Addition | Cross-multiply denominators | Exact |
| Multiplication | Limb-by-limb with carry | Exact |
| Division | Numerator × 1/Denominator | Exact |
| GCD Reduction | Euclidean algorithm | Automatic |
| Comparison | Cross-multiply | Exact |

#### Why EPR, Not BigInt Libraries?

| Aspect | External Libraries (num-bigint) | EPR |
|--------|--------------------------------|-----|
| Dependencies | External crate | None |
| Design Philosophy | General-purpose BigInt | ℚ-native arithmetic |
| Integration | Requires conversion | Direct S-wind integration |
| Control | Black-box | Full source control |

---

### 2.2 Fate - Unified Exploration Engine

**Source**: `src/fate.rs` (3,524 lines)

#### Design Philosophy

```rust
// Fate Design Philosophy (from source):
// - Fate is FREE - no hardcoded mathematical knowledge
// - All patterns and known sequences loaded from KnowledgeBase
// - Supports self-evolution (self-loop cycling)
// - Uses only EPR (infinite precision), never approximate
// - System explores ALL methods - no premature termination
```

#### Architecture

```
                    ┌─────────────────┐
                    │      Fate       │
                    │ (Unified Explorer)│
                    └────────┬────────┘
                             │
          ┌──────────────────┼──────────────────┐
          │                  │                  │
    ┌─────▼─────┐     ┌─────▼─────┐     ┌─────▼─────┐
    │  Explore  │     │ Verify    │     │   Self    │
    │ (Pattern) │     │ (Algebra) │     │ (Cycling) │
    └───────────┘     └───────────┘     └───────────┘
```

#### Fate-Managed Verification (v1.1.1 Critical Update)

**Problem (v1.1.0 Bug)**: OOS verification was checking polynomial self-consistency (the polynomial predicts its own extrapolations correctly), not actual pattern validity.

**Solution (v1.1.1)**: Fate now manages verification using **real algebraic pattern consistency**:

```rust
fn verify_prediction_consistency(&self, signal: &[EPR], predicted: &EPR, pattern_class: &str) -> bool {
    // Build extended signal
    let mut extended: Vec<EPR> = signal.to_vec();
    extended.push(predicted.clone());
    
    // Use pattern-specific algebraic verification
    if pattern_class.contains("linear") {
        self.check_linear_consistency(&extended)      // Δ¹ constant?
    } else if pattern_class.contains("quadratic") {
        self.check_quadratic_consistency(&extended)   // Δ² constant?
    } else if pattern_class.contains("polynomial") {
        self.check_polynomial_consistency(&extended, pattern_class)  // Δᵈ constant?
    } else if pattern_class.contains("reciprocal") {
        self.check_reciprocal_consistency(&extended)  // c/(n+k) form?
    } else if pattern_class.contains("exponential") {
        self.check_exponential_consistency(&extended) // ratio constant?
    } else {
        false  // Unknown pattern cannot be verified
    }
}
```

**Critical Protection**: Pure interpolation (degree ≥ n-1) is **automatically rejected**:

```rust
fn check_polynomial_consistency(&self, signal: &[EPR], pattern_class: &str) -> bool {
    let degree = extract_degree(pattern_class);
    let n = signal.len();
    
    // CRITICAL CHECK: Pure interpolation cannot be verified!
    // For n points with degree (n-1) polynomial:
    // - Δ^(n-1) has only 1 value
    // - Cannot compare to verify consistency
    if degree >= n - 1 {
        return false;  // REJECT
    }
    
    // ... actual verification ...
}
```

#### Self-Loop Exploration

**Purpose**: Convert short trivial fits into verified discoveries by extending signals using detected formulas.

**Process** (v1.1.1 Verified):
```
Input: [4, 7, 10]  (3 points, linear 3n+1, DoF=1, trivial)
       │
       ▼
┌──────────────────────────────────────────────────────────────┐
│ Iteration 1-10: Detect f(n) = 3n+1                           │
│                 Extend: [4,7,10] → 53 points                 │
├──────────────────────────────────────────────────────────────┤
│ Verification:  Δ¹ constant = 3 for all 53 points             │
│                Cross-domain: 51 verified points              │
├──────────────────────────────────────────────────────────────┤
│ Final DoF = 46 (= 48 - 2)                                    │
│ is_discovery = true ✓                                        │
│ 📦 AUTO-ARCHIVED: discovery_10                               │
│ 🌀 NEW META-SYSTEM: Space container created                  │
└──────────────────────────────────────────────────────────────┘
       │
       ▼
Output: Short trivial fit → Extended verified discovery (DoF +45!)
```

**Verified Results** (2026-01-24):

| Pattern | Input | DoF₀ | Self-Loop | DoF_final | Result |
|---------|-------|------|-----------|-----------|--------|
| 3n+1 | 3 pts | 1 | 3→53 | 46 | EmergentLaw |
| 2n²+5 | 4 pts | 1 | 4→54 | 46 | EmergentLaw |
| 7n+3 | 3 pts | 1 | 3→53 | 46 | EmergentLaw |

#### Exploration Flow

```
explore_with_verification(signal, kb)
    │
    ├─► 1. Check known sequences (from KB)
    │       ├─► Match by first_terms (integer sequences)
    │       └─► Match by closed_form (fractional sequences)
    │
    ├─► 2. Try pattern detection
    │       ├─► Reciprocal (constant product)
    │       ├─► Exponential (constant ratio)
    │       ├─► Periodic
    │       ├─► Factorial
    │       ├─► Polynomial (finite differences)
    │       └─► Linear Recurrence
    │
    ├─► 3. If pattern found:
    │       ├─► FitAnalysis (DoF calculation)
    │       ├─► Cross-domain verification (Fate-managed)
    │       │       ├─► Predict next values using formula
    │       │       ├─► Verify algebraic consistency (NOT self-consistency!)
    │       │       └─► Reject pure interpolations
    │       └─► Classify fit level
    │
    └─► 4. Generate Discovery Assessment
            ├─► OEIS check (post-verification, not blocking)
            ├─► DoF check (≥2 required for discovery)
            └─► Final status assignment
```

#### Fit Level Classification

| Level | Criteria | Academic Meaning |
|-------|----------|------------------|
| **InterpolantFit** | DoF = 0 (pure interpolation) | Mathematically guaranteed, zero information |
| **DescriptiveModel** | DoF > 0, OOS failed | Empirical description only |
| **PredictiveFit** | OOS passed, effective DoF < 2 | Predictive but insufficient constraint |
| **EmergentLaw** | OOS passed, effective DoF ≥ 2 | Candidate for discovery |

---

### 2.3 Voyager - Pattern Detection System

**Source**: `utils/voyager/src/lib.rs` (1,863 lines)

#### Core Capabilities

```rust
// Voyager Core Capabilities:
// 1. detect_signal_precisely() - Detect if signal matches known patterns
// 2. create_mathematical_equation() - Create equation for unknown signals
// 3. expand_dimensions_mathematically() - Expand dimensions and archive
// 4. verify_signal_against_equation() - Reverse verification (result→cause)
//
// All computation uses exact algebra (Rational i128/i128), zero floating-point.
```

#### Detection Methods

| Method | Description | Pattern |
|--------|-------------|---------|
| **ConstantDifference** | nth order difference is constant | Polynomial degree n-1 |
| **ConstantRatio** | Adjacent ratio is constant | Exponential/Geometric |
| **ConstantProduct** | a[n] × (n+k) = constant | Reciprocal (c/(n+k)) |
| **Factorial** | a[n] / a[n-1] = n | n! |
| **Recurrence** | Linear combination of previous terms | Fibonacci-type |

#### Equation Types

```rust
pub enum EquationType {
    Constant(Rational),                    // f(n) = c
    Linear { slope, intercept },           // f(n) = an + b
    Quadratic { a, b, c },                 // f(n) = an² + bn + c
    Polynomial { coefficients },           // f(n) = Σ aᵢnⁱ
    Periodic { period, values },           // f(n) repeats
    Exponential { base, coefficient },     // f(n) = c × bⁿ
    Reciprocal { numerator, offset },      // f(n) = a/(n+b)
    Unknown,
}
```

#### Derivation Trace Generation

Every detection generates a step-by-step trace:

```
(1) Input: [1, 4, 9, 16, 25]
(2) Δ¹ = [3, 5, 7, 9]
(3) Δ² = [2, 2, 2]
(4) ✓ Δ² constant = 2
(5) Polynomial coefficients:
    a₂ = Δ²/2! = 2/2 = 1
    a₁ = Δ¹[0] - a₂ = 3 - 1 = 2
    a₀ = f(1) - a₁ - a₂ = 1 - 2 - 1 = -2 (adjusted for index)
(6) Formula: f(n) = n²
```

---

### 2.4 KnowledgeBase - Domain Knowledge Storage

**Source**: `src/knowledge/` (4,466 lines total)

#### Architecture

```rust
// KnowledgeBase Architecture:
// - Fate loads pattern classes from here - it has NO hardcoded patterns
// - All mathematical knowledge lives here
// Storage format: JSON (efficient with serde)
// Import/Export: TeX (human-readable)
```

#### Data Structures

```rust
pub struct KnowledgeBase {
    pub physical_constants: Vec<PhysicalConstant>,   // NIST CODATA
    pub math_constants: Vec<MathConstant>,           // π, e, γ, etc.
    pub math_theorems: Vec<MathTheorem>,             // 78 theorems
    pub physical_laws: Vec<PhysicalLaw>,             // 60 laws
    pub discovered: Vec<DiscoveredEquation>,         // S-wind discoveries
    pub pattern_classes: Vec<PatternClass>,          // For Fate
    pub known_sequences: Vec<KnownSequence>,         // 199 sequences (OEIS)
}
```

#### Pre-loaded Knowledge

| Category | Count | Source |
|----------|-------|--------|
| Physical Constants | 20 | NIST CODATA 2022 |
| Mathematical Constants | 15 | Standard references |
| Mathematical Theorems | 78 | Comprehensive coverage |
| Physical Laws | 60 | Classical to Modern physics |
| Known Sequences | 199 | OEIS built-in |
| Pattern Classes | 8 | S-wind detection methods |

#### Pattern Classes

```rust
pub struct PatternClass {
    pub id: String,
    pub name: String,
    pub detection: DetectionMethod,
    pub complexity: Complexity,
    pub priority: u8,
    pub description: String,
}

pub enum DetectionMethod {
    ConstantDifference { order: usize },
    ConstantRatio,
    ConstantProduct { factor_form: String },
    Recurrence { max_order: usize },
    Periodic { max_period: usize },
    Factorial,
    Logarithmic,
    Custom { detector_id: String },
}
```

---

### 2.5 Space - Fractional Dimension System

**Source**: `src/space.rs` (427 lines)

#### Features

- **FractionalDimensionCalculator**: Hausdorff, Box, Information dimensions
- **adaptive_dimension_expansion()**: Automatic dimension adjustment
- **Discovery-triggered container creation**: New containers for new patterns

#### Usage

```rust
let mut space = SWindSpace::new();

// Create containers
space.create_container(ContainerType::Basic);
space.create_fractional_container(&data);

// Fate-driven adjustment (can increase or decrease)
let adjustment = space.fate_driven_adjustment(&data);

// Create container on discovery
let discovery = space.on_discovery(&data, "pattern_id");
```

---

### 2.6 Quad - Physics Simulation Systems

**Source**: `src/quad.rs` (699 lines)

Four rigorous physics simulation systems using exact rational arithmetic:

| System | Physics | Formula |
|--------|---------|---------|
| **GravitationalField** | Newtonian gravity | F = GM/r², r_s = 2GM/c² |
| **QuantumFieldSystem** | Quantum mechanics | E_n = ℏω(n + ½) |
| **ThermalRadiationProcessor** | Stefan-Boltzmann | P = σAT⁴ |
| **FundamentalCouplingCoordinator** | Coupling constants | α_eff coordination |

---

## Part III: Verification System (v1.1.1)

### The Verification Bug (v1.1.0)

**Problem**: The old verification functions were "empty shells":

```rust
// OLD (BUG): Always returned true!
fn verify_polynomial_continuation(&self, signal: &[EPR], _i: usize, _predicted: &EPR) -> bool {
    signal.len() >= 2  // This is ALWAYS true for any input
}
```

This caused:
- Möbius M(n)/√n to be incorrectly classified as "EmergentLaw"
- Pure interpolations (DoF=0) to pass OOS verification
- False "discoveries" in chaotic sequences

### The Fix (v1.1.1)

**Real algebraic verification** managed by Fate:

```rust
fn check_linear_consistency(&self, signal: &[EPR]) -> bool {
    let d1 = signal[1].clone() - signal[0].clone();
    for i in 1..signal.len() - 1 {
        let diff = signal[i + 1].clone() - signal[i].clone();
        if diff != d1 {
            return false;  // Not constant difference → Not linear
        }
    }
    true
}

fn check_polynomial_consistency(&self, signal: &[EPR], pattern_class: &str) -> bool {
    let degree = extract_degree(pattern_class);
    let n = signal.len();
    
    // CRITICAL: Reject pure interpolation
    if degree >= n - 1 {
        return false;  // Cannot verify with only 1 Δᵈ value
    }
    
    // Compute differences and verify constancy at degree d
    // ...
}
```

### Verification Test Results

| Test | Before (Bug) | After (Fix) | Correct? |
|------|--------------|-------------|----------|
| Linear 3n+1 | ❌ DescriptiveModel | ✅ EmergentLaw | ✅ |
| Möbius N=10 | ❌ EmergentLaw | ✅ InterpolantFit | ✅ |
| Fibonacci | ✅ [Known] | ✅ [Known] | ✅ |
| n² | ✅ [Known] | ✅ [Known] | ✅ |

---

## Part IV: Label Classification System

### Fit Levels

| Label | Symbol | Meaning | DoF Requirement |
|-------|--------|---------|-----------------|
| `[Recognized]` | 📚 | Known sequence from KB | N/A |
| `[InterpolantFit]` | ⚪ | Pure interpolation, zero information | DoF = 0 |
| `[DescriptiveModel]` | 🔵 | Empirical description, OOS failed | DoF > 0 |
| `[PredictiveFit]` | 🟡 | OOS passed, insufficient DoF | DoF < 2 |
| `[EmergentLaw]` | 🟢 | Candidate discovery | DoF ≥ 2, OOS passed |

### Discovery Assessment (Three-Layer)

```
Layer 1: OEIS Status
  - ExactMatch: Found in OEIS
  - NoMatch: Not in OEIS (potential discovery)
  - NotApplicable: Not an integer sequence

Layer 2: Discovery Status  
  - [Known]: In KnowledgeBase
  - [TrivialInterpolation]: DoF = 0
  - [DescriptiveOnly]: OOS failed
  - [CandidateLaw]: Passes basic criteria
  - [StrongCandidate]: Passes all criteria, not in OEIS
  - [GenuineDiscovery]: Verified novel + correct + non-trivial

Layer 3: Negative Knowledge
  - "Pattern class X exhausted" is a valid output
  - Provides information about what the sequence is NOT
```

---

## Part V: CLI Usage

### Basic Commands

```bash
# Pattern detection
swind detect 1,4,9,16,25,36

# Run experiments
swind run

# Query OEIS
swind oeis A000045

# System info
swind info
```

### Example Output

```
╔══════════════════════════════════════════════════════════════════╗
║                    S-wind Pattern Detection                      ║
╚══════════════════════════════════════════════════════════════════╝

Input: [1, 4, 9, 16, 25, 36]
Length: 6

RESULT:
  Known sequence: Square numbers (A000290)
  Formula: n^2
  Matched: A000290

Time: 28.4µs

DISCOVERY STATUS: ❌ NO

Derivation (4 steps):
  (1) Match known sequence (offset 1): Square numbers
  (2) OEIS ID: A000290
  (3) Input starts at index 1 of OEIS sequence
  (4) Closed form: n^2
```

---

## Part VI: Academic Position

### What S-wind Is

1. **A formal exploration system** operating over ℚ
2. **An auditable derivation generator** for mathematical patterns
3. **A discovery criterion enforcer** with strict DoF requirements
4. **A pattern recognition system** with real algebraic verification

### What S-wind Is NOT

1. ❌ A numerical approximation tool
2. ❌ A curve-fitting library
3. ❌ An AI that "discovers" by overfitting
4. ❌ A system that inflates interpolation into "discovery"

### Academic Statement

> *S-wind treats fitting as an information-bearing intermediate object, never as a discovery.*  
> *Only patterns that pass pre-registered out-of-sample verification within a constrained pattern class, with DoF ≥ 2, are candidates for discovery.*

---

## Part VII: Future Directions

### Implemented (v1.1.1)

- ✅ EPR unlimited precision arithmetic
- ✅ Fate-managed real algebraic verification
- ✅ Self-loop exploration
- ✅ Three-layer discovery assessment
- ✅ Pure interpolation rejection
- ✅ 199 OEIS sequences
- ✅ 78 mathematical theorems
- ✅ 60 physical laws

### Reserved (Architecture Ready)

- ⏸ Meta-observation layer (config: `enable_meta`)
- ⏸ Cross-domain isomorphism detection
- ⏸ Novel pattern class generation
- ⏸ Active learning suggestions

---

## Part VIII: File Structure

```
s-wind-sim/
├── src/
│   ├── fate.rs           # Core exploration engine (3,524 lines)
│   ├── space.rs          # Dimension management (427 lines)
│   ├── quad.rs           # Physics systems (699 lines)
│   ├── interface.rs      # API interfaces (642 lines)
│   ├── lib.rs            # Re-exports (179 lines)
│   ├── symbolic/
│   │   ├── epr.rs        # Extended precision (737 lines)
│   │   ├── expr.rs       # Symbolic expressions (1,025 lines)
│   │   └── rational.rs   # Rational wrapper (318 lines)
│   ├── knowledge/
│   │   ├── mod.rs        # KnowledgeBase (365 lines)
│   │   ├── constants.rs  # Physics/math constants (410 lines)
│   │   ├── theorems.rs   # Mathematical theorems (1,627 lines)
│   │   ├── patterns.rs   # Pattern classes (790 lines)
│   │   ├── oeis.rs       # OEIS sequences (1,060 lines)
│   │   └── storage.rs    # Persistence (214 lines)
│   └── bin/
│       ├── swind.rs      # CLI entry point
│       └── ...           # Experiment runners
└── utils/
    └── voyager/
        └── src/lib.rs    # Pattern detection (1,863 lines)
```

---

## Citation

```bibtex
@software{swind2026,
  title = {S-wind: Formal Algebraic Exploration System over ℚ},
  author = {Jouly / Rogue Intelligence},
  version = {1.1.1},
  year = {2026},
  note = {Fate-managed verification, self-loop exploration, three-layer discovery assessment}
}
```

---

*S-wind: Where exactness meets discovery.*  
*Author: Jouly / Rogue Intelligence*
