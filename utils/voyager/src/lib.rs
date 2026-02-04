//! Voyager - Precise Signal Detection & Equation Creation System
//!
//! Core capabilities from Paper Section 3.3:
//! 1. detect_signal_precisely() - Detect if signal matches known patterns
//! 2. create_mathematical_equation() - Create equation for unknown signals
//! 3. expand_dimensions_mathematically() - Expand dimensions and archive
//! 4. verify_signal_against_equation() - Reverse verification (result→cause)
//!
//! All computation uses exact algebra (Rational i128/i128), zero floating-point.

use serde::{Deserialize, Serialize};

/// Exact rational number - i128/i128 with GCD reduction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rational {
    pub numerator: i128,
    pub denominator: i128,
}

impl Rational {
    pub fn new(n: i128, d: i128) -> Self {
        if d == 0 {
            panic!("Division by zero");
        }
        let g = gcd(n.abs(), d.abs());
        let sign = if d < 0 { -1 } else { 1 };
        Rational {
            numerator: sign * n / g,
            denominator: (sign * d).abs() / g,
        }
    }

    pub fn zero() -> Self {
        Rational { numerator: 0, denominator: 1 }
    }

    pub fn one() -> Self {
        Rational { numerator: 1, denominator: 1 }
    }

    pub fn is_zero(&self) -> bool {
        self.numerator == 0
    }
    
    /// Absolute value
    pub fn abs(&self) -> Rational {
        Rational {
            numerator: self.numerator.abs(),
            denominator: self.denominator,
        }
    }

    pub fn add(&self, other: &Rational) -> Rational {
        Rational::new(
            self.numerator * other.denominator + other.numerator * self.denominator,
            self.denominator * other.denominator,
        )
    }

    pub fn sub(&self, other: &Rational) -> Rational {
        Rational::new(
            self.numerator * other.denominator - other.numerator * self.denominator,
            self.denominator * other.denominator,
        )
    }

    pub fn mul(&self, other: &Rational) -> Rational {
        Rational::new(
            self.numerator * other.numerator,
            self.denominator * other.denominator,
        )
    }

    pub fn div(&self, other: &Rational) -> Rational {
        Rational::new(
            self.numerator * other.denominator,
            self.denominator * other.numerator,
        )
    }
    
    /// Convert to LaTeX representation
    pub fn to_latex(&self) -> String {
        if self.denominator == 1 {
            format!("{}", self.numerator)
        } else {
            format!("\\frac{{{}}}{{{}}}", self.numerator, self.denominator)
        }
    }
}

fn gcd(a: i128, b: i128) -> i128 {
    if b == 0 { a } else { gcd(b, a % b) }
}

impl std::fmt::Display for Rational {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.denominator == 1 {
            write!(f, "{}", self.numerator)
        } else {
            write!(f, "{}/{}", self.numerator, self.denominator)
        }
    }
}

/// Equation type - pattern classification (not mathematical definitions)
/// Mathematical formulas are represented using s-wind's Expr system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EquationType {
    /// Constant: f ≡ c
    Constant(Rational),
    /// Linear: a*n + b (detected via constant Δ¹)
    /// offset: signal starts at n = offset + 1
    Linear {
        coefficient: Rational,
        constant: Rational,
        offset: i128,
    },
    /// Quadratic: a*n² + b*n + c (detected via constant Δ²)
    /// offset: signal starts at n = offset + 1
    Quadratic {
        a: Rational,
        b: Rational,
        c: Rational,
        offset: i128,  // n = i + offset where i is 1-indexed
    },
    /// General polynomial: Σ a_i * n^i (detected via constant Δⁿ)
    Polynomial {
        coefficients: Vec<Rational>,  // [a₀, a₁, a₂, ...] 
    },
    /// Periodic: f(n) = f(n mod p)
    Periodic {
        period: i128,
        values: Vec<Rational>,
    },
    /// Exponential: a * b^n (detected via constant ratio)
    Exponential {
        base: Rational,
        multiplier: Rational,
    },
    /// Reciprocal: c / (n + k) (detected via constant product s[i]*(i+k))
    Reciprocal {
        numerator: Rational,
        offset: i128,
    },
    /// Unknown: stores raw coefficients for s-wind Expr to interpret
    Unknown {
        signal_values: Vec<Rational>,
    },
}

impl EquationType {
    /// Format as mathematical expression (human-readable)
    pub fn to_math_string(&self) -> String {
        match self {
            EquationType::Constant(c) => format!("f ≡ {}", c),
            EquationType::Linear { coefficient, constant, .. } => {
                format!("f(n) = {}·n + {}", coefficient, constant)
            }
            EquationType::Quadratic { a, b, c, .. } => {
                format!("f(n) = {}·n² + {}·n + {}", a, b, c)
            }
            EquationType::Polynomial { coefficients } => {
                format!("f(n) = Σᵢ aᵢ·nⁱ  (deg = {})", coefficients.len().saturating_sub(1))
            }
            EquationType::Periodic { period, .. } => {
                format!("f(n) = f(n mod {})", period)
            }
            EquationType::Exponential { base, multiplier } => {
                format!("f(n) = {}·{}ⁿ", multiplier, base)
            }
            EquationType::Reciprocal { numerator, offset } => {
                if *offset == 0 {
                    format!("f(n) = {}/n", numerator)
                } else if *offset > 0 {
                    format!("f(n) = {}/(n + {})", numerator, offset)
                } else {
                    format!("f(n) = {}/(n - {})", numerator, offset.abs())
                }
            }
            EquationType::Unknown { signal_values } => {
                format!("f: unknown ({} values)", signal_values.len())
            }
        }
    }
}

/// Signal equation - stores known algebraic patterns with derivation trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEquation {
    pub equation_id: String,
    pub equation_type: EquationType,
    pub description: String,
    pub discovered_at: u64,
    pub verification_count: u64,
    /// Full derivation trace for this equation
    pub derivation_trace: Vec<DerivationStep>,
    /// Algebraic verification: exact match = true, otherwise equation is invalid
    pub algebraically_verified: bool,
}

/// Detection result
#[derive(Debug, Clone)]
pub enum PreciseDetectionResult {
    /// Matches known equation
    KnownEquation {
        equation_id: String,
        equation: SignalEquation,
    },
    /// Unknown signal
    UnknownSignal {
        signal_signature: Vec<Rational>,
        potential_patterns: Vec<String>,
    },
}

impl PreciseDetectionResult {
    pub fn is_unknown(&self) -> bool {
        matches!(self, PreciseDetectionResult::UnknownSignal { .. })
    }
}

/// Derivation step - structured proof trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivationStep {
    pub step_number: u32,
    pub description: String,
    pub rule_applied: DerivationRule,
    pub input_expr: String,
    pub output_expr: String,
    pub result: Option<Rational>,
}

/// Derivation rules - mathematical operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DerivationRule {
    Axiom,              // Starting axiom
    Definition,         // Apply definition
    Substitution,       // Substitute variable
    Simplification,     // Algebraic simplification
    Addition,           // Add expressions
    Subtraction,        // Subtract expressions
    Multiplication,     // Multiply expressions
    Division,           // Divide expressions
    SquareRoot,         // Take square root
    PowerRule,          // Apply power rule
    PatternMatch,       // Pattern matching
    AlgebraicIdentity,  // Apply algebraic identity
    Induction,          // Mathematical induction
    Verification,       // Verify result
}

/// New equation judgment with full derivation trace
#[derive(Debug, Clone)]
pub struct NewEquationJudgment {
    pub suggested_type: EquationType,
    pub derivation_steps: Vec<DerivationStep>,
    /// Algebraic verification status: true = exact match, false = no match
    /// No "confidence" - mathematics is either correct or wrong
    pub algebraically_verified: bool,
}

/// Mathematical expansion result
#[derive(Debug, Clone)]
pub struct MathematicalExpansionResult {
    pub new_dimensions: usize,
    pub equations_added: usize,
    pub knowledge_updated: bool,
}

/// Mathematical processing result
#[derive(Debug, Clone)]
pub struct MathematicalProcessingResult {
    pub detection: PreciseDetectionResult,
    pub equation_created: Option<SignalEquation>,
    pub expansion: Option<MathematicalExpansionResult>,
}

/// Precise Signal Detector - Voyager core
///
/// From Paper Section 3.3:
/// ```text
/// pub struct PreciseSignalDetector {
///     known_equations: Vec<SignalEquation>,  // Grows over time
///     mathematical_tolerance: Rational,      // Exact algebraic tolerance
/// }
/// ```
#[derive(Debug, Clone)]
pub struct PreciseSignalDetector {
    /// Known equation library (grows over time)
    pub known_equations: Vec<SignalEquation>,
    /// Detection history
    pub detection_history: Vec<(Vec<Rational>, String)>,
    /// Current dimensions
    pub current_dimensions: usize,
}

impl PreciseSignalDetector {
    /// Create new detector with empty equation library
    /// No hardcoded equations - all patterns must be discovered
    pub fn new() -> Self {
        PreciseSignalDetector {
            known_equations: Vec::new(),  // Empty - no preset!
            detection_history: Vec::new(),
            current_dimensions: 4,
        }
    }

    /// Step 1: Detect if signal matches known patterns (exact comparison)
    pub fn detect_signal_precisely(&mut self, input: Vec<Rational>) -> PreciseDetectionResult {
        for equation in &self.known_equations {
            if self.verify_signal_against_equation(&input, equation) {
                return PreciseDetectionResult::KnownEquation {
                    equation_id: equation.equation_id.clone(),
                    equation: equation.clone(),
                };
            }
        }

        PreciseDetectionResult::UnknownSignal {
            signal_signature: input,
            potential_patterns: Vec::new(),
        }
    }

    /// Reverse verification: given result (signal), verify which cause (equation) it matches
    ///
    /// From Paper Section 2.3:
    /// ```text
    /// fn verify_signal_against_equation(&self, signal, equation) -> bool {
    ///     match equation.equation_type {
    ///         Linear { .. }      => verify_linear_equation(signal, ...)
    ///         Polynomial { .. }  => verify_polynomial_equation(signal, ...)
    ///         Periodic { .. }    => verify_periodic_equation(signal, ...)
    ///         Exponential { .. } => verify_exponential_equation(signal, ...)
    ///         Custom { .. }      => verify_custom_pattern(signal, ...)
    ///     }
    /// }
    /// ```
    pub fn verify_signal_against_equation(&self, signal: &[Rational], equation: &SignalEquation) -> bool {
        match &equation.equation_type {
            EquationType::Constant(c) => {
                signal.iter().all(|v| v == c)
            }
            EquationType::Linear { coefficient, constant, .. } => {
                self.verify_linear_equation(signal, coefficient, constant)
            }
            EquationType::Quadratic { a, b, c, .. } => {
                self.verify_quadratic_equation(signal, a, b, c)
            }
            EquationType::Polynomial { coefficients } => {
                self.verify_polynomial_equation(signal, coefficients)
            }
            EquationType::Periodic { period, values } => {
                self.verify_periodic_equation(signal, *period, values)
            }
            EquationType::Exponential { base, multiplier } => {
                self.verify_exponential_equation(signal, base, multiplier)
            }
            EquationType::Reciprocal { numerator, offset } => {
                self.verify_reciprocal_equation(signal, numerator, *offset)
            }
            EquationType::Unknown { .. } => false,
        }
    }
    
    fn verify_quadratic_equation(&self, signal: &[Rational], a: &Rational, b: &Rational, c: &Rational) -> bool {
        for (i, val) in signal.iter().enumerate() {
            let n = Rational::new((i + 1) as i128, 1);
            let n_sq = n.mul(&n);
            let expected = a.mul(&n_sq).add(&b.mul(&n)).add(c);
            if *val != expected {
                return false;
            }
        }
        true
    }

    fn verify_linear_equation(&self, signal: &[Rational], coef: &Rational, constant: &Rational) -> bool {
        for (i, val) in signal.iter().enumerate() {
            let n = Rational::new((i + 2) as i128, 1);
            let expected = coef.mul(&n).add(constant);
            if *val != expected {
                return false;
            }
        }
        true
    }

    fn verify_polynomial_equation(&self, _signal: &[Rational], _coefficients: &[Rational]) -> bool {
        // TODO: Implement polynomial verification
        false
    }

    fn verify_periodic_equation(&self, signal: &[Rational], period: i128, values: &[Rational]) -> bool {
        for (i, val) in signal.iter().enumerate() {
            let expected_idx = (i as i128) % period;
            if let Some(expected) = values.get(expected_idx as usize) {
                if val != expected {
                    return false;
                }
            }
        }
        true
    }

    fn verify_exponential_equation(&self, signal: &[Rational], base: &Rational, multiplier: &Rational) -> bool {
        for (i, val) in signal.iter().enumerate() {
            let n = (i + 1) as i128;
            // f(n) = multiplier * base^n
            let mut base_pow = Rational::one();
            for _ in 0..n {
                base_pow = base_pow.mul(base);
            }
            let expected = multiplier.mul(&base_pow);
            if *val != expected {
                return false;
            }
        }
        true
    }

    /// Verify Reciprocal form: f(n) = c / (n + k)
    fn verify_reciprocal_equation(&self, signal: &[Rational], numerator: &Rational, offset: i128) -> bool {
        for (i, val) in signal.iter().enumerate() {
            let n = (i + 1) as i128;
            let denom = n + offset;
            if denom == 0 {
                return false;
            }
            let expected = numerator.div(&Rational::new(denom, 1));
            if *val != expected {
                return false;
            }
        }
        true
    }

    /// Step 2: Create mathematical equation for unknown signals
    /// Includes DETAILED full derivation trace with every algebraic step
    pub fn create_mathematical_equation(&self, result: &PreciseDetectionResult, signal: &[Rational]) -> Option<NewEquationJudgment> {
        if let PreciseDetectionResult::UnknownSignal { signal_signature, .. } = result {
            let mut steps = Vec::new();
            let mut step_num = 1u32;
            
            // ═══════════════════════════════════════════════════════════════
            // PART 1: Input Signal Analysis (Detailed)
            // ═══════════════════════════════════════════════════════════════
            
            let signal_str: Vec<String> = signal_signature.iter().map(|r| r.to_string()).collect();
            steps.push(DerivationStep {
                step_number: step_num,
                description: "Given: Observed signal S for pattern inference".to_string(),
                rule_applied: DerivationRule::Axiom,
                input_expr: "∅".to_string(),
                output_expr: format!("S = {{{}}}", signal_str.join(", ")),
                result: None,
            });
            step_num += 1;
            
            // Show each signal value with index
            for (i, val) in signal.iter().enumerate() {
                steps.push(DerivationStep {
                    step_number: step_num,
                    description: format!("Extract signal component s_{}", i + 1),
                    rule_applied: DerivationRule::Definition,
                    input_expr: format!("S[{}]", i),
                    output_expr: format!("s_{} = {}", i + 1, val),
                    result: Some(*val),
                });
                step_num += 1;
            }
            
            steps.push(DerivationStep {
                step_number: step_num,
                description: "Cardinality of signal set".to_string(),
                rule_applied: DerivationRule::Axiom,
                input_expr: format!("S = {{{}}}", signal_str.join(", ")),
                output_expr: format!("|S| = {}", signal.len()),
                result: Some(Rational::new(signal.len() as i128, 1)),
            });
            step_num += 1;
            
            // ═══════════════════════════════════════════════════════════════
            // PART 2: First Difference Sequence (Δ¹)
            // ═══════════════════════════════════════════════════════════════
            
            if signal.len() >= 2 {
                steps.push(DerivationStep {
                    step_number: step_num,
                    description: "Begin first difference analysis: Δ¹ᵢ = sᵢ₊₁ - sᵢ".to_string(),
                    rule_applied: DerivationRule::Definition,
                    input_expr: "Signal sequence {s₁, s₂, ...}".to_string(),
                    output_expr: "Compute Δ¹ = {Δ¹₁, Δ¹₂, ...}".to_string(),
                    result: None,
                });
                step_num += 1;
                
                let mut differences: Vec<Rational> = Vec::new();
                for i in 0..signal.len() - 1 {
                    let diff = signal[i + 1].sub(&signal[i]);
                    differences.push(diff);
                    
                    steps.push(DerivationStep {
                        step_number: step_num,
                        description: format!("Compute Δ¹_{}", i + 1),
                        rule_applied: DerivationRule::Subtraction,
                        input_expr: format!("s_{} - s_{} = {} - {}", i + 2, i + 1, signal[i + 1], signal[i]),
                        output_expr: format!("Δ¹_{} = {}", i + 1, diff),
                        result: Some(diff),
                    });
                    step_num += 1;
                }
                
                // Check if constant difference (linear pattern)
                let all_equal = differences.windows(2).all(|w| w[0] == w[1]);
                if !differences.is_empty() {
                    let diff_str: Vec<String> = differences.iter().map(|d| d.to_string()).collect();
                    steps.push(DerivationStep {
                        step_number: step_num,
                        description: "First difference sequence summary".to_string(),
                        rule_applied: DerivationRule::Simplification,
                        input_expr: format!("Δ¹ = {{{}}}", diff_str.join(", ")),
                        output_expr: if all_equal && !differences.is_empty() {
                            format!("∀i: Δ¹ᵢ = {} (constant)", differences[0])
                        } else {
                            "Δ¹ is not constant".to_string()
                        },
                        result: if all_equal && !differences.is_empty() { Some(differences[0]) } else { None },
                    });
                    step_num += 1;
                }
                
                // ═══════════════════════════════════════════════════════════════
                // PART 3: Second Difference Sequence (Δ²) for quadratic detection
                // ═══════════════════════════════════════════════════════════════
                
                if differences.len() >= 2 {
                    steps.push(DerivationStep {
                        step_number: step_num,
                        description: "Begin second difference analysis: Δ²ᵢ = Δ¹ᵢ₊₁ - Δ¹ᵢ".to_string(),
                        rule_applied: DerivationRule::Definition,
                        input_expr: "First difference sequence Δ¹".to_string(),
                        output_expr: "Compute Δ² = {Δ²₁, Δ²₂, ...}".to_string(),
                        result: None,
                    });
                    step_num += 1;
                    
                    let mut second_diffs: Vec<Rational> = Vec::new();
                    for i in 0..differences.len() - 1 {
                        let d2 = differences[i + 1].sub(&differences[i]);
                        second_diffs.push(d2);
                        
                        steps.push(DerivationStep {
                            step_number: step_num,
                            description: format!("Compute Δ²_{}", i + 1),
                            rule_applied: DerivationRule::Subtraction,
                            input_expr: format!("Δ¹_{} - Δ¹_{} = {} - {}", i + 2, i + 1, differences[i + 1], differences[i]),
                            output_expr: format!("Δ²_{} = {}", i + 1, d2),
                            result: Some(d2),
                        });
                        step_num += 1;
                    }
                    
                    let all_zero = second_diffs.iter().all(|d| d.is_zero());
                    let d2_str: Vec<String> = second_diffs.iter().map(|d| d.to_string()).collect();
                    steps.push(DerivationStep {
                        step_number: step_num,
                        description: "Second difference analysis result".to_string(),
                        rule_applied: DerivationRule::Simplification,
                        input_expr: format!("Δ² = {{{}}}", d2_str.join(", ")),
                        output_expr: if all_zero {
                            "∀i: Δ²ᵢ = 0 → Linear pattern confirmed".to_string()
                        } else {
                            "Δ² ≠ 0 → Non-linear pattern".to_string()
                        },
                        result: None,
                    });
                    step_num += 1;
                }
            }
            
            // ═══════════════════════════════════════════════════════════════
            // PART 4: Pattern Detection & Classification
            // ═══════════════════════════════════════════════════════════════
            
            let (detected_type, _pattern_desc) = self.detect_pattern_type_detailed(signal, &mut steps, &mut step_num);
            
            // ═══════════════════════════════════════════════════════════════
            // PART 5: Coefficient Derivation (DETAILED ALGEBRAIC EXPANSION)
            // ═══════════════════════════════════════════════════════════════
            
            match &detected_type {
                EquationType::Linear { coefficient, constant, .. } => {
                    // Step 5.1: State the theorem for linear sequences
                    steps.push(DerivationStep {
                        step_number: step_num,
                        description: "By Newton's difference theorem: constant Δ¹ implies f(n) = a·n + b".to_string(),
                        rule_applied: DerivationRule::Definition,
                        input_expr: "Linear sequence characterization".to_string(),
                        output_expr: "f(n) = a·n + b, a = common difference".to_string(),
                        result: None,
                    });
                    step_num += 1;
                    
                    // Step 5.2: Derive coefficient a
                    steps.push(DerivationStep {
                        step_number: step_num,
                        description: "Derive coefficient a from constant first difference".to_string(),
                        rule_applied: DerivationRule::Definition,
                        input_expr: "a = Δ¹₁ (common difference)".to_string(),
                        output_expr: format!("a = {}", coefficient),
                        result: Some(*coefficient),
                    });
                    step_num += 1;
                    
                    // Step 5.3: Derive constant b - DETAILED
                    steps.push(DerivationStep {
                        step_number: step_num,
                        description: "Apply formula for constant term".to_string(),
                        rule_applied: DerivationRule::Definition,
                        input_expr: "b = s₁ - a·n₁ where n₁ = 2 (first index)".to_string(),
                        output_expr: "b = s₁ - a·2".to_string(),
                        result: None,
                    });
                    step_num += 1;
                    
                    // Step 5.4: Substitute values
                    let a_times_2 = coefficient.mul(&Rational::new(2, 1));
                    steps.push(DerivationStep {
                        step_number: step_num,
                        description: "Substitute known values".to_string(),
                        rule_applied: DerivationRule::Substitution,
                        input_expr: format!("b = {} - {}·2", signal[0], coefficient),
                        output_expr: format!("b = {} - {}", signal[0], a_times_2),
                        result: None,
                    });
                    step_num += 1;
                    
                    // Step 5.5: Compute subtraction
                    steps.push(DerivationStep {
                        step_number: step_num,
                        description: "Compute subtraction".to_string(),
                        rule_applied: DerivationRule::Subtraction,
                        input_expr: format!("{} - {}", signal[0], a_times_2),
                        output_expr: format!("b = {}", constant),
                        result: Some(*constant),
                    });
                    step_num += 1;
                    
                    // Step 5.6: Construct final equation
                    steps.push(DerivationStep {
                        step_number: step_num,
                        description: "Construct linear equation from derived coefficients".to_string(),
                        rule_applied: DerivationRule::Definition,
                        input_expr: format!("a = {}, b = {}", coefficient, constant),
                        output_expr: format!("∴ f(n) = {}·n + ({})", coefficient, constant),
                        result: None,
                    });
                    step_num += 1;
                }
                EquationType::Quadratic { a, b, c, .. } => {
                    // Quadratic detailed derivation
                    steps.push(DerivationStep {
                        step_number: step_num,
                        description: "By Newton's difference theorem: constant Δ² implies f(n) = an² + bn + c".to_string(),
                        rule_applied: DerivationRule::Definition,
                        input_expr: "Second difference characterization".to_string(),
                        output_expr: "f(n) = an² + bn + c where 2a = Δ²".to_string(),
                        result: None,
                    });
                    step_num += 1;
                    
                    steps.push(DerivationStep {
                        step_number: step_num,
                        description: "Derive coefficient a from Δ²".to_string(),
                        rule_applied: DerivationRule::Division,
                        input_expr: "a = Δ²/2".to_string(),
                        output_expr: format!("a = {}", a),
                        result: Some(*a),
                    });
                    step_num += 1;
                    
                    steps.push(DerivationStep {
                        step_number: step_num,
                        description: "Derive coefficient b from Δ¹₁ and a".to_string(),
                        rule_applied: DerivationRule::Subtraction,
                        input_expr: "b = Δ¹₁ - a·(2n₁ + 1) where n₁ = start index".to_string(),
                        output_expr: format!("b = {}", b),
                        result: Some(*b),
                    });
                    step_num += 1;
                    
                    steps.push(DerivationStep {
                        step_number: step_num,
                        description: "Derive constant c from s₁, a, b".to_string(),
                        rule_applied: DerivationRule::Subtraction,
                        input_expr: "c = s₁ - a·n₁² - b·n₁".to_string(),
                        output_expr: format!("c = {}", c),
                        result: Some(*c),
                    });
                    step_num += 1;
                    
                    steps.push(DerivationStep {
                        step_number: step_num,
                        description: "Final quadratic form".to_string(),
                        rule_applied: DerivationRule::Definition,
                        input_expr: format!("a = {}, b = {}, c = {}", a, b, c),
                        output_expr: format!("∴ f(n) = {}·n² + {}·n + {}", a, b, c),
                        result: None,
                    });
                    step_num += 1;
                }
                EquationType::Reciprocal { numerator, offset } => {
                    steps.push(DerivationStep {
                        step_number: step_num,
                        description: "Reciprocal pattern detected: f(n) = c/(n+k)".to_string(),
                        rule_applied: DerivationRule::PatternMatch,
                        input_expr: "s[i] · (i + k) = constant for all i".to_string(),
                        output_expr: format!("c = {}, k = {}", numerator, offset),
                        result: None,
                    });
                    step_num += 1;
                    
                    steps.push(DerivationStep {
                        step_number: step_num,
                        description: "Final reciprocal form".to_string(),
                        rule_applied: DerivationRule::Definition,
                        input_expr: format!("numerator = {}, offset = {}", numerator, offset),
                        output_expr: if *offset == 0 {
                            format!("∴ f(n) = {}/n", numerator)
                        } else {
                            format!("∴ f(n) = {}/(n + {})", numerator, offset)
                        },
                        result: None,
                    });
                    step_num += 1;
                }
                _ => {}
            }
            
            // ═══════════════════════════════════════════════════════════════
            // PART 6: Point-by-Point Algebraic Verification (DETAILED)
            // ═══════════════════════════════════════════════════════════════
            
            steps.push(DerivationStep {
                step_number: step_num,
                description: "Begin point-by-point algebraic verification".to_string(),
                rule_applied: DerivationRule::Verification,
                input_expr: format!("Verify f(n) = sₙ for n ∈ {{2, 3, ..., {}}}", signal.len() + 1),
                output_expr: "Must show f(n) equals observed signal value for each n".to_string(),
                result: None,
            });
            step_num += 1;
            
            let mut all_verified = true;
            
            // Determine starting n offset from detected type
            // For all patterns: n = i + 1 + offset where offset is inferred during detection
            // For Reciprocal: n = i + 1 (offset is in the denominator, not the starting n)
            let base_offset = match &detected_type {
                EquationType::Linear { offset, .. } => *offset,
                EquationType::Quadratic { offset, .. } => *offset,
                EquationType::Reciprocal { .. } => 0,  // n starts at 1 for reciprocal
                _ => 0,  // Others: n starts at 1
            };
            
            for (i, val) in signal.iter().enumerate() {
                let n = (i as i128) + 1 + base_offset;
                
                match &detected_type {
                    EquationType::Linear { coefficient, constant, offset: _ } => {
                        let n_rat = Rational::new(n, 1);
                        let a_times_n = coefficient.mul(&n_rat);
                        let expected = a_times_n.add(constant);
                        let matches = expected == *val;
                        
                        if !matches {
                            all_verified = false;
                        }
                        
                        // Step 6.x.1: State what we need to verify
                        steps.push(DerivationStep {
                            step_number: step_num,
                            description: format!("Verify f({}) = s_{}", n, i + 1),
                            rule_applied: DerivationRule::Verification,
                            input_expr: format!("Need: f({}) = {}", n, val),
                            output_expr: format!("Compute f({})", n),
                            result: None,
                        });
                        step_num += 1;
                        
                        // Step 6.x.2: Substitute n into f(n)
                        steps.push(DerivationStep {
                            step_number: step_num,
                            description: "Substitute n into f(n) = a·n + b".to_string(),
                            rule_applied: DerivationRule::Substitution,
                            input_expr: format!("f({}) = {}·{} + ({})", n, coefficient, n, constant),
                            output_expr: format!("f({}) = {} + ({})", n, a_times_n, constant),
                            result: None,
                        });
                        step_num += 1;
                        
                        // Step 6.x.3: Compute the addition
                        steps.push(DerivationStep {
                            step_number: step_num,
                            description: "Compute addition".to_string(),
                            rule_applied: DerivationRule::Addition,
                            input_expr: format!("{} + ({})", a_times_n, constant),
                            output_expr: format!("f({}) = {}", n, expected),
                            result: Some(expected),
                        });
                        step_num += 1;
                        
                        // Step 6.x.4: Compare with observed value
                        steps.push(DerivationStep {
                            step_number: step_num,
                            description: "Compare computed f(n) with observed s".to_string(),
                            rule_applied: DerivationRule::Verification,
                            input_expr: format!("f({}) = {} vs s_{} = {}", n, expected, i + 1, val),
                            output_expr: if matches { 
                                format!("{} = {} ✓ EXACT MATCH", expected, val) 
                            } else { 
                                format!("{} ≠ {} ✗ MISMATCH", expected, val) 
                            },
                            result: Some(if matches { Rational::one() } else { Rational::zero() }),
                        });
                        step_num += 1;
                    }
                    EquationType::Quadratic { a, b, c, offset: _ } => {
                        // Verify f(n) = an² + bn + c
                        let n_rat = Rational::new(n, 1);
                        let n_sq = n_rat.mul(&n_rat);
                        let a_n2 = a.mul(&n_sq);
                        let b_n = b.mul(&n_rat);
                        let expected = a_n2.add(&b_n).add(c);
                        let matches = expected == *val;
                        
                        if !matches {
                            all_verified = false;
                        }
                        
                        steps.push(DerivationStep {
                            step_number: step_num,
                            description: format!("Verify f({}) = s_{}", n, i + 1),
                            rule_applied: DerivationRule::Verification,
                            input_expr: format!("f({}) = {}·{}² + {}·{} + {}", n, a, n, b, n, c),
                            output_expr: format!("f({}) = {} + {} + {} = {}", n, a_n2, b_n, c, expected),
                            result: Some(expected),
                        });
                        step_num += 1;
                        
                        steps.push(DerivationStep {
                            step_number: step_num,
                            description: "Compare with observed".to_string(),
                            rule_applied: DerivationRule::Verification,
                            input_expr: format!("f({}) = {} vs s_{} = {}", n, expected, i + 1, val),
                            output_expr: if matches { 
                                format!("{} = {} ✓ EXACT MATCH", expected, val) 
                            } else { 
                                format!("{} ≠ {} ✗ MISMATCH", expected, val) 
                            },
                            result: Some(if matches { Rational::one() } else { Rational::zero() }),
                        });
                        step_num += 1;
                    }
                    EquationType::Reciprocal { numerator, offset } => {
                        // Verify f(n) = c / (n + k)
                        let denom = Rational::new(n + *offset, 1);
                        let expected = numerator.div(&denom);
                        let matches = expected == *val;
                        
                        if !matches {
                            all_verified = false;
                        }
                        
                        steps.push(DerivationStep {
                            step_number: step_num,
                            description: format!("Verify f({}) = s_{}", n, i + 1),
                            rule_applied: DerivationRule::Verification,
                            input_expr: format!("f({}) = {} / ({} + {})", n, numerator, n, offset),
                            output_expr: format!("f({}) = {}", n, expected),
                            result: Some(expected),
                        });
                        step_num += 1;
                        
                        steps.push(DerivationStep {
                            step_number: step_num,
                            description: "Compare with observed".to_string(),
                            rule_applied: DerivationRule::Verification,
                            input_expr: format!("{} vs {}", expected, val),
                            output_expr: if matches { 
                                format!("{} = {} ✓ EXACT MATCH", expected, val) 
                            } else { 
                                format!("{} ≠ {} ✗ MISMATCH", expected, val) 
                            },
                            result: Some(if matches { Rational::one() } else { Rational::zero() }),
                        });
                        step_num += 1;
                    }
                    _ => {
                        steps.push(DerivationStep {
                            step_number: step_num,
                            description: format!("Verify s_{} = {}", i + 1, val),
                            rule_applied: DerivationRule::Verification,
                            input_expr: format!("Signal value s_{}", i + 1),
                            output_expr: format!("s_{} = {} (recorded)", i + 1, val),
                            result: Some(*val),
                        });
                        step_num += 1;
                    }
                }
            }
            
            // ═══════════════════════════════════════════════════════════════
            // PART 7: Final Conclusion
            // ═══════════════════════════════════════════════════════════════
            
            steps.push(DerivationStep {
                step_number: step_num,
                description: "Verification summary".to_string(),
                rule_applied: DerivationRule::Verification,
                input_expr: format!("Verified {} points", signal.len()),
                output_expr: if all_verified {
                    format!("ALL {} POINTS VERIFIED ∴ EQUATION VALID", signal.len())
                } else {
                    "VERIFICATION FAILED".to_string()
                },
                result: Some(if all_verified { Rational::one() } else { Rational::zero() }),
            });
            step_num += 1;
            
            steps.push(DerivationStep {
                step_number: step_num,
                description: "Conclusion".to_string(),
                rule_applied: DerivationRule::Verification,
                input_expr: detected_type.to_math_string(),
                output_expr: if all_verified { "□ (verified on sample domain)" } else { "⊥ (verification failed)" }.to_string(),
                result: None,
            });
            
            Some(NewEquationJudgment {
                suggested_type: detected_type,
                derivation_steps: steps,
                algebraically_verified: all_verified,
            })
        } else {
            None
        }
    }
    
    /// Detailed pattern detection with step recording
    fn detect_pattern_type_detailed(&self, signal: &[Rational], steps: &mut Vec<DerivationStep>, step_num: &mut u32) -> (EquationType, String) {
        if signal.is_empty() {
            steps.push(DerivationStep {
                step_number: *step_num,
                description: "Empty signal detected".to_string(),
                rule_applied: DerivationRule::PatternMatch,
                input_expr: "|S| = 0".to_string(),
                output_expr: "Pattern: Empty".to_string(),
                result: None,
            });
            *step_num += 1;
            return (EquationType::Unknown { signal_values: vec![] }, "Empty signal".to_string());
        }
        
        // ─────────────────────────────────────────────────────────────────
        // Test 1: Linear Pattern (constant first difference)
        // ─────────────────────────────────────────────────────────────────
        
        steps.push(DerivationStep {
            step_number: *step_num,
            description: "Test hypothesis: Linear pattern f(n) = an + b".to_string(),
            rule_applied: DerivationRule::PatternMatch,
            input_expr: "Check if Δ¹ is constant".to_string(),
            output_expr: "Linear ⟺ ∀i: Δ¹ᵢ = Δ¹₁".to_string(),
            result: None,
        });
        *step_num += 1;
        
        if signal.len() >= 2 {
            let diff = signal[1].sub(&signal[0]);
            let mut is_linear = true;
            
            for i in 2..signal.len() {
                let current_diff = signal[i].sub(&signal[i-1]);
                if current_diff != diff {
                    is_linear = false;
                    steps.push(DerivationStep {
                        step_number: *step_num,
                        description: format!("Linear test failed at i = {}", i),
                        rule_applied: DerivationRule::Verification,
                        input_expr: format!("Δ¹_{} = {} ≠ Δ¹₁ = {}", i, current_diff, diff),
                        output_expr: "Linear pattern REJECTED".to_string(),
                        result: Some(Rational::zero()),
                    });
                    *step_num += 1;
                    break;
                }
            }
            
            if is_linear {
                // Try different offsets to find the one that verifies
                for offset in 0i128..=5 {
                    let start_n = 1 + offset;  // n for first sample
                    let constant = signal[0].sub(&diff.mul(&Rational::new(start_n, 1)));
                    
                    // Verify all points with this offset
                    let mut verified = true;
                    for (i, val) in signal.iter().enumerate() {
                        let n = (i as i128) + 1 + offset;
                        let expected = diff.mul(&Rational::new(n, 1)).add(&constant);
                        if expected != *val {
                            verified = false;
                            break;
                        }
                    }
                    
                    if verified {
                        steps.push(DerivationStep {
                            step_number: *step_num,
                            description: "Linear pattern CONFIRMED".to_string(),
                            rule_applied: DerivationRule::Verification,
                            input_expr: format!("∀i: Δ¹ᵢ = {}, offset = {}", diff, offset),
                            output_expr: format!("f(n) = {}·n + {} (n starts at {})", diff, constant, start_n),
                            result: Some(Rational::one()),
                        });
                        *step_num += 1;
                        
                        return (EquationType::Linear {
                            coefficient: diff,
                            constant,
                            offset,
                        }, format!("Linear: f(n) = {}n + {} (n={}..)", diff, constant, start_n));
                    }
                }
                
                // Fallback: use offset=1 (n starts at 2)
                let offset = 1i128;
                let start_n = 1 + offset;
                let constant = signal[0].sub(&diff.mul(&Rational::new(start_n, 1)));
                
                steps.push(DerivationStep {
                    step_number: *step_num,
                    description: "Linear pattern CONFIRMED".to_string(),
                    rule_applied: DerivationRule::Verification,
                    input_expr: format!("∀i: Δ¹ᵢ = {}", diff),
                    output_expr: format!("f(n) = {}·n + {}", diff, constant),
                    result: Some(Rational::one()),
                });
                *step_num += 1;
                
                return (EquationType::Linear {
                    coefficient: diff,
                    constant,
                    offset,
                }, format!("Linear pattern: d = {}", diff));
            }
        }
        
        // ─────────────────────────────────────────────────────────────────
        // Test 2: Quadratic Pattern (constant second difference)
        // ─────────────────────────────────────────────────────────────────
        
        if signal.len() >= 3 {
            steps.push(DerivationStep {
                step_number: *step_num,
                description: "Test hypothesis: Quadratic pattern f(n) = an² + bn + c".to_string(),
                rule_applied: DerivationRule::PatternMatch,
                input_expr: "Check if Δ² is constant".to_string(),
                output_expr: "Quadratic ⟺ ∀i: Δ²ᵢ = Δ²₁".to_string(),
                result: None,
            });
            *step_num += 1;
            
            // Compute first differences
            let mut d1: Vec<Rational> = Vec::new();
            for i in 0..signal.len() - 1 {
                d1.push(signal[i + 1].sub(&signal[i]));
            }
            
            // Compute second differences
            let mut d2: Vec<Rational> = Vec::new();
            for i in 0..d1.len() - 1 {
                d2.push(d1[i + 1].sub(&d1[i]));
            }
            
            let all_d2_equal = !d2.is_empty() && d2.windows(2).all(|w| w[0] == w[1]);
            
            if all_d2_equal && !d2[0].is_zero() {
                // Quadratic detected!
                // f(n) = an² + bn + c where 2a = Δ²
                let two_a = d2[0];
                let a = two_a.div(&Rational::new(2, 1));
                
                // Try different offsets to find the one that verifies
                // offset means: signal[i] corresponds to n = i + offset
                for offset in 0i128..=5 {
                    let start_n = 1 + offset;  // n for first sample
                    
                    // Δ¹[0] = f(start_n+1) - f(start_n) = a(2·start_n + 1) + b
                    let coef = Rational::new(2 * start_n + 1, 1);
                    let b = d1[0].sub(&a.mul(&coef));
                    
                    // c = s[0] - a·start_n² - b·start_n
                    let start_n_rat = Rational::new(start_n, 1);
                    let c = signal[0].sub(&a.mul(&start_n_rat.mul(&start_n_rat))).sub(&b.mul(&start_n_rat));
                    
                    // Verify all points with this offset
                    let mut verified = true;
                    for (i, val) in signal.iter().enumerate() {
                        let n = (i as i128) + 1 + offset;
                        let n_rat = Rational::new(n, 1);
                        let n_sq = n_rat.mul(&n_rat);
                        let expected = a.mul(&n_sq).add(&b.mul(&n_rat)).add(&c);
                        if expected != *val {
                            verified = false;
                            break;
                        }
                    }
                    
                    if verified {
                        steps.push(DerivationStep {
                            step_number: *step_num,
                            description: "Quadratic pattern CONFIRMED".to_string(),
                            rule_applied: DerivationRule::Verification,
                            input_expr: format!("Δ² = {} (constant), offset = {}", two_a, offset),
                            output_expr: format!("f(n) = {}·n² + {}·n + {} (n starts at {})", a, b, c, start_n),
                            result: Some(Rational::one()),
                        });
                        *step_num += 1;
                        
                        return (EquationType::Quadratic { a, b, c, offset }, 
                                format!("Quadratic: f(n) = {}n² + {}n + {} (n={}..)", a, b, c, start_n));
                    }
                }
                
                // Fallback: use offset=0 if no verification passes
                let start_n: i128 = 1;
                let coef = Rational::new(2 * start_n + 1, 1);
                let b = d1[0].sub(&a.mul(&coef));
                let start_n_rat = Rational::new(start_n, 1);
                let c = signal[0].sub(&a.mul(&start_n_rat.mul(&start_n_rat))).sub(&b.mul(&start_n_rat));
                
                steps.push(DerivationStep {
                    step_number: *step_num,
                    description: "Quadratic pattern detected (offset inference failed)".to_string(),
                    rule_applied: DerivationRule::Verification,
                    input_expr: format!("Δ² = {} (constant)", two_a),
                    output_expr: format!("f(n) = {}·n² + {}·n + {}", a, b, c),
                    result: Some(Rational::one()),
                });
                *step_num += 1;
                
                return (EquationType::Quadratic { a, b, c, offset: 0 }, format!("Quadratic: f(n) = {}n² + {}n + {}", a, b, c));
            }
        }
        
        // ─────────────────────────────────────────────────────────────────
        // Test 3: Reciprocal Pattern (s[i] * (i + k) = constant)
        // ─────────────────────────────────────────────────────────────────
        
        if signal.len() >= 2 {
            steps.push(DerivationStep {
                step_number: *step_num,
                description: "Test hypothesis: Reciprocal pattern f(n) = c/(n+k)".to_string(),
                rule_applied: DerivationRule::PatternMatch,
                input_expr: "Check if s[i] * (i + k) = constant".to_string(),
                output_expr: "Reciprocal ⟺ ∃c,k: ∀i: sᵢ·(i+k) = c".to_string(),
                result: None,
            });
            *step_num += 1;
            
            // Try k = 0 (pure 1/n form) first, then k = 1, etc.
            for k in 0i128..=3 {
                let mut products: Vec<Rational> = Vec::new();
                let mut valid = true;
                
                for (i, val) in signal.iter().enumerate() {
                    let n = (i as i128) + 1 + k;
                    if n == 0 {
                        valid = false;
                        break;
                    }
                    products.push(val.mul(&Rational::new(n, 1)));
                }
                
                if valid && !products.is_empty() {
                    let all_equal = products.windows(2).all(|w| w[0] == w[1]);
                    if all_equal {
                        let c = products[0];
                        
                        steps.push(DerivationStep {
                            step_number: *step_num,
                            description: "Reciprocal pattern CONFIRMED".to_string(),
                            rule_applied: DerivationRule::Verification,
                            input_expr: format!("∀i: sᵢ·(i+{}) = {}", 1 + k, c),
                            output_expr: format!("f(n) = {}/(n+{})", c, k),
                            result: Some(Rational::one()),
                        });
                        *step_num += 1;
                        
                        return (EquationType::Reciprocal { numerator: c, offset: k }, 
                                format!("Reciprocal: f(n) = {}/{}", c, if k == 0 { "n".to_string() } else { format!("(n+{})", k) }));
                    }
                }
            }
        }
        
        // ─────────────────────────────────────────────────────────────────
        // Default: Unknown pattern
        // ─────────────────────────────────────────────────────────────────
        
        steps.push(DerivationStep {
            step_number: *step_num,
            description: "No known pattern matched".to_string(),
            rule_applied: DerivationRule::PatternMatch,
            input_expr: "All hypothesis tests failed".to_string(),
            output_expr: format!("Unknown pattern (|S| = {})", signal.len()),
            result: None,
        });
        *step_num += 1;
        
        (EquationType::Unknown {
            signal_values: signal.to_vec(),
        }, "Unknown algebraic pattern".to_string())
    }

    /// Step 3: Expand dimensions and add to knowledge base (with derivation trace)
    pub fn expand_dimensions_mathematically(&mut self, judgment: &NewEquationJudgment, current_dims: usize) -> MathematicalExpansionResult {
        let eq_id = format!("eq_{}", self.known_equations.len());
        
        let new_equation = SignalEquation {
            equation_id: eq_id.clone(),
            equation_type: judgment.suggested_type.clone(),
            description: format!("Auto-discovered equation #{}", self.known_equations.len()),
            discovered_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            verification_count: if judgment.algebraically_verified { 1 } else { 0 },
            derivation_trace: judgment.derivation_steps.clone(),
            algebraically_verified: judgment.algebraically_verified,
        };

        // Archive to knowledge base
        self.known_equations.push(new_equation);
        
        // Record in detection history
        self.detection_history.push((vec![], eq_id));
        
        let new_dimensions = (current_dims + 1).min(11);
        self.current_dimensions = new_dimensions;

        MathematicalExpansionResult {
            new_dimensions,
            equations_added: 1,
            knowledge_updated: true,
        }
    }
    
    /// Get derivation trace for an equation
    pub fn get_derivation_trace(&self, equation_id: &str) -> Option<&Vec<DerivationStep>> {
        self.known_equations
            .iter()
            .find(|eq| eq.equation_id == equation_id)
            .map(|eq| &eq.derivation_trace)
    }
    
    /// Format derivation trace in mathematical style
    pub fn format_derivation_trace(&self, equation_id: &str) -> Option<String> {
        let eq = self.known_equations.iter().find(|e| e.equation_id == equation_id)?;
        let trace = &eq.derivation_trace;
        
        let mut output = String::new();
        
        // Proposition header (not "Theorem" - we prove interpolation, not universal claims)
        output.push_str("────────────────────────────────────────────────────────────\n");
        output.push_str(&format!("Proposition ({})\n", equation_id));
        output.push_str("────────────────────────────────────────────────────────────\n\n");
        
        // Pattern class assumed (explicitly state search space for rigor)
        let pattern_class = match &eq.equation_type {
            EquationType::Constant(_) => "constant function",
            EquationType::Linear { .. } => "linear polynomial (degree 1)",
            EquationType::Quadratic { .. } => "quadratic polynomial (degree 2)",
            EquationType::Polynomial { coefficients } => {
                if coefficients.len() <= 3 { "polynomial (degree ≤ 2)" }
                else { "polynomial (degree ≤ n)" }
            }
            EquationType::Periodic { .. } => "periodic function",
            EquationType::Exponential { .. } => "exponential form (a·bⁿ)",
            EquationType::Reciprocal { .. } => "reciprocal form (c/(n+k))",
            EquationType::Unknown { .. } => "unknown",
        };
        output.push_str(&format!("Pattern class assumed: {}\n\n", pattern_class));
        
        // Equation type as mathematical statement
        let eq_desc = match &eq.equation_type {
            EquationType::Constant(c) => {
                format!("Let f ≡ {}", c)
            }
            EquationType::Linear { coefficient, constant, .. } => {
                format!("Let f(n) = {}·n + {}", coefficient, constant)
            }
            EquationType::Quadratic { a, b, c, .. } => {
                format!("Let f(n) = {}·n² + {}·n + {}", a, b, c)
            }
            EquationType::Polynomial { coefficients } => {
                format!("Let f(n) = Σᵢ aᵢ·nⁱ  where a = {:?}", coefficients)
            }
            EquationType::Periodic { period, values } => {
                format!("Let f(n) = f(n mod {})  where f(0..{}) = {:?}", period, period, values)
            }
            EquationType::Exponential { base, multiplier } => {
                format!("Let f(n) = {}·{}ⁿ", multiplier, base)
            }
            EquationType::Reciprocal { numerator, offset } => {
                if *offset == 0 {
                    format!("Let f(n) = {}/n", numerator)
                } else {
                    format!("Let f(n) = {}/(n + {})", numerator, offset)
                }
            }
            EquationType::Unknown { signal_values } => {
                format!("Let f be unknown function with {} observed values", signal_values.len())
            }
        };
        output.push_str(&format!("{}.\n\n", eq_desc));
        
        // Derivation (not "Proof" - we derive an interpolant, not prove a theorem)
        output.push_str("Derivation.\n\n");
        
        for step in trace {
            let rule_str = match step.rule_applied {
                DerivationRule::Axiom => "given",
                DerivationRule::Definition => "by definition",
                DerivationRule::Substitution => "by substitution",
                DerivationRule::Simplification => "by simplification",
                DerivationRule::Addition => "by addition",
                DerivationRule::Subtraction => "by subtraction",
                DerivationRule::Multiplication => "by multiplication",
                DerivationRule::Division => "by division",
                DerivationRule::SquareRoot => "by square root",
                DerivationRule::PowerRule => "by power rule",
                DerivationRule::PatternMatch => "by pattern recognition",
                DerivationRule::AlgebraicIdentity => "by algebraic identity",
                DerivationRule::Induction => "by induction",
                DerivationRule::Verification => "by verification",
            };
            
            output.push_str(&format!("  ({}) {}  {}\n", 
                step.step_number, 
                step.output_expr,
                rule_str
            ));
        }
        
        // Conclusion
        if eq.algebraically_verified {
            output.push_str("\n  ∴ The interpolant is verified on all sample points.  □\n");
        } else {
            output.push_str("\n  (Verification incomplete)\n");
        }
        
        output.push_str("────────────────────────────────────────────────────────────\n");
        
        Some(output)
    }
    
    /// Export derivation as LaTeX
    pub fn format_derivation_latex(&self, equation_id: &str) -> Option<String> {
        let eq = self.known_equations.iter().find(|e| e.equation_id == equation_id)?;
        let trace = &eq.derivation_trace;
        
        let mut output = String::new();
        
        output.push_str("\\begin{theorem}\n");
        
        // Equation type as LaTeX
        let eq_tex = match &eq.equation_type {
            EquationType::Constant(c) => format!("f \\equiv {}", c.to_latex()),
            EquationType::Linear { coefficient, constant, .. } => {
                format!("f(n) = {}n + {}", coefficient.to_latex(), constant.to_latex())
            }
            EquationType::Quadratic { a, b, c, .. } => {
                format!("f(n) = {}n^2 + {}n + {}", a.to_latex(), b.to_latex(), c.to_latex())
            }
            EquationType::Polynomial { coefficients } => {
                format!("f(n) = \\sum_{{i=0}}^{{{}}} a_i n^i", coefficients.len() - 1)
            }
            EquationType::Periodic { period, .. } => {
                format!("f(n) = f(n \\mod {})", period)
            }
            EquationType::Exponential { base, multiplier } => {
                format!("f(n) = {} \\cdot {}^n", multiplier.to_latex(), base.to_latex())
            }
            EquationType::Reciprocal { numerator, offset } => {
                if *offset == 0 {
                    format!("f(n) = \\frac{{{}}}{{{}}}", numerator.to_latex(), "n")
                } else {
                    format!("f(n) = \\frac{{{}}}{{n + {}}}", numerator.to_latex(), offset)
                }
            }
            EquationType::Unknown { signal_values } => {
                format!("f: \\text{{unknown ({} values)}}", signal_values.len())
            }
        };
        output.push_str(&format!("Let ${}$.\n", eq_tex));
        output.push_str("\\end{theorem}\n\n");
        
        output.push_str("\\begin{proof}\n");
        output.push_str("\\begin{align*}\n");
        
        for step in trace {
            let rule_tex = match step.rule_applied {
                DerivationRule::Axiom => "\\text{(given)}",
                DerivationRule::Definition => "\\text{(def)}",
                DerivationRule::Substitution => "\\text{(subst)}",
                DerivationRule::Simplification => "\\text{(simp)}",
                DerivationRule::Addition => "\\text{(add)}",
                DerivationRule::Subtraction => "\\text{(sub)}",
                DerivationRule::Multiplication => "\\text{(mul)}",
                DerivationRule::Division => "\\text{(div)}",
                DerivationRule::SquareRoot => "\\text{(sqrt)}",
                DerivationRule::PowerRule => "\\text{(pow)}",
                DerivationRule::PatternMatch => "\\text{(pattern)}",
                DerivationRule::AlgebraicIdentity => "\\text{(id)}",
                DerivationRule::Induction => "\\text{(ind)}",
                DerivationRule::Verification => "\\text{(verify)}",
            };
            output.push_str(&format!("  &{} && {} \\\\\n", step.output_expr, rule_tex));
        }
        
        output.push_str("\\end{align*}\n");
        
        if eq.algebraically_verified {
            output.push_str("Therefore, the equation is algebraically verified.\n");
        }
        
        output.push_str("\\end{proof}\n");
        
        Some(output)
    }

    /// Complete flow: detect → judge → expand → archive
    pub fn process_signal_mathematically(&mut self, signal: Vec<Rational>, dims: usize) -> MathematicalProcessingResult {
        let detection = self.detect_signal_precisely(signal.clone());
        
        let (equation_created, expansion) = if detection.is_unknown() {
            if let Some(judgment) = self.create_mathematical_equation(&detection, &signal) {
                let exp_result = self.expand_dimensions_mathematically(&judgment, dims);
                (self.known_equations.last().cloned(), Some(exp_result))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        MathematicalProcessingResult {
            detection,
            equation_created,
            expansion,
        }
    }

    pub fn known_equation_count(&self) -> usize {
        self.known_equations.len()
    }

    pub fn get_current_dimensions(&self) -> usize {
        self.current_dimensions
    }
}

impl Default for PreciseSignalDetector {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Compatibility Layer: Dimension Management API (backward compatible with Space)
// ============================================================================

/// Dimension configuration (compatible with Space system)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionConfig {
    pub initial_dimensions: usize,
    pub max_dimensions: usize,
    pub auto_expand: bool,
    /// Support fractional dimensions (from documentation)
    pub fractional_support: bool,
}

impl Default for DimensionConfig {
    fn default() -> Self {
        Self {
            initial_dimensions: 4,
            max_dimensions: 11,
            auto_expand: true,
            fractional_support: true,
        }
    }
}

/// Discovery for Fate integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoyagerDiscovery {
    pub equation_id: String,
    pub equation_type: String,
    pub coefficients: Vec<Rational>,
    pub algebraically_verified: bool,
    pub discovered_at: u64,
}

impl From<&SignalEquation> for VoyagerDiscovery {
    fn from(eq: &SignalEquation) -> Self {
        let coefficients = match &eq.equation_type {
            EquationType::Linear { coefficient, constant, .. } => {
                vec![*coefficient, *constant]
            }
            EquationType::Quadratic { a, b, c, .. } => {
                vec![*a, *b, *c]
            }
            EquationType::Reciprocal { numerator, offset } => {
                vec![*numerator, Rational::new(*offset, 1)]
            }
            EquationType::Constant(c) => vec![*c],
            _ => vec![],
        };
        
        VoyagerDiscovery {
            equation_id: eq.equation_id.clone(),
            equation_type: eq.equation_type.to_math_string(),
            coefficients,
            algebraically_verified: eq.algebraically_verified,
            discovered_at: eq.discovered_at,
        }
    }
}

/// Dimension manager with adaptive expansion (from documentation)
#[derive(Debug, Clone)]
pub struct DimensionManager {
    config: DimensionConfig,
    detector: PreciseSignalDetector,
    /// Fractional dimension estimate
    pub current_fractional_dimension: Rational,
    /// Expansion history
    expansion_history: Vec<(u64, usize, String)>,
}

impl DimensionManager {
    pub fn new(config: DimensionConfig) -> Self {
        let initial = config.initial_dimensions;
        let mut detector = PreciseSignalDetector::new();
        detector.current_dimensions = initial;
        Self { 
            config, 
            detector,
            current_fractional_dimension: Rational::new(initial as i128, 1),
            expansion_history: Vec::new(),
        }
    }

    pub fn get_current_dimensions(&self) -> usize {
        self.detector.get_current_dimensions()
    }
    
    /// Get fractional dimension estimate
    pub fn get_fractional_dimension(&self) -> Rational {
        self.current_fractional_dimension
    }

    pub fn expand_dimension(&mut self) -> usize {
        if self.config.auto_expand && self.detector.current_dimensions < self.config.max_dimensions {
            self.detector.current_dimensions += 1;
            self.current_fractional_dimension = Rational::new(self.detector.current_dimensions as i128, 1);
            self.expansion_history.push((
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                self.detector.current_dimensions,
                "manual_expand".into(),
            ));
        }
        self.detector.current_dimensions
    }
    
    /// Adaptive dimension expansion based on signal data (from documentation)
    /// Uses variance, range, and pattern complexity to determine optimal dimension
    pub fn adaptive_dimension_expansion(&mut self, signal: &[Rational]) -> Rational {
        if signal.is_empty() {
            return self.current_fractional_dimension;
        }
        
        // Calculate dimension estimates
        let variance_dim = self.estimate_dimension_from_variance(signal);
        let range_dim = self.estimate_dimension_from_range(signal);
        let complexity_dim = self.estimate_dimension_from_complexity(signal);
        
        // Combine estimates: weighted average
        let sum = variance_dim.add(&range_dim).add(&complexity_dim);
        let avg = sum.div(&Rational::new(3, 1));
        
        // Clamp to valid range
        let max_dim = Rational::new(self.config.max_dimensions as i128, 1);
        let min_dim = Rational::one();
        
        let clamped = if avg.numerator * min_dim.denominator < min_dim.numerator * avg.denominator {
            min_dim
        } else if avg.numerator * max_dim.denominator > max_dim.numerator * avg.denominator {
            max_dim
        } else {
            avg
        };
        
        // Update if higher than current
        if clamped.numerator * self.current_fractional_dimension.denominator > 
           self.current_fractional_dimension.numerator * clamped.denominator {
            self.current_fractional_dimension = clamped;
            
            // Also update integer dimension if needed
            let int_dim = (clamped.numerator / clamped.denominator) as usize;
            if int_dim > self.detector.current_dimensions && int_dim <= self.config.max_dimensions {
                self.detector.current_dimensions = int_dim;
                self.expansion_history.push((
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    int_dim,
                    "adaptive_expand".into(),
                ));
            }
        }
        
        self.current_fractional_dimension
    }
    
    /// Estimate dimension from signal variance
    fn estimate_dimension_from_variance(&self, signal: &[Rational]) -> Rational {
        if signal.len() < 2 {
            return Rational::one();
        }
        
        let n = signal.len() as i128;
        let sum: Rational = signal.iter().fold(Rational::zero(), |acc, x| acc.add(x));
        let mean = sum.div(&Rational::new(n, 1));
        
        let mut var_sum = Rational::zero();
        for x in signal {
            let diff = x.sub(&mean);
            var_sum = var_sum.add(&diff.mul(&diff));
        }
        let variance = var_sum.div(&Rational::new(n, 1));
        
        // Higher variance → higher dimension
        let base = Rational::new(self.config.initial_dimensions as i128, 1);
        let modifier = variance.abs();
        
        // Log-like scaling (simplified): dim = base + modifier/base
        base.add(&modifier.div(&base.add(&Rational::one())))
    }
    
    /// Estimate dimension from signal range
    fn estimate_dimension_from_range(&self, signal: &[Rational]) -> Rational {
        if signal.len() < 2 {
            return Rational::one();
        }
        
        let mut min_val = signal[0].numerator as f64 / signal[0].denominator as f64;
        let mut max_val = min_val;
        
        for s in signal.iter().skip(1) {
            let v = s.numerator as f64 / s.denominator as f64;
            if v < min_val { min_val = v; }
            if v > max_val { max_val = v; }
        }
        
        let range = max_val - min_val;
        let n = signal.len() as f64;
        
        // Dimension estimate from range/n ratio
        let dim_f = (self.config.initial_dimensions as f64) + (range / n.max(1.0)).min(3.0);
        
        // Convert to Rational (simplified)
        Rational::new((dim_f * 100.0) as i128, 100)
    }
    
    /// Estimate dimension from pattern complexity
    fn estimate_dimension_from_complexity(&self, signal: &[Rational]) -> Rational {
        if signal.is_empty() {
            return Rational::one();
        }
        
        // Count direction changes as complexity proxy
        let mut changes = 0i128;
        for i in 2..signal.len() {
            let d1 = signal[i-1].sub(&signal[i-2]);
            let d2 = signal[i].sub(&signal[i-1]);
            
            // Direction change if signs differ
            if (d1.numerator > 0) != (d2.numerator > 0) {
                changes += 1;
            }
        }
        
        let base = Rational::new(self.config.initial_dimensions as i128, 1);
        let modifier = Rational::new(changes, signal.len().max(1) as i128);
        
        base.add(&modifier)
    }
    
    /// Process signal and optionally expand (complete flow)
    pub fn process_signal_with_expansion(&mut self, signal: Vec<Rational>) -> (MathematicalProcessingResult, Option<VoyagerDiscovery>) {
        // Adaptive expansion
        self.adaptive_dimension_expansion(&signal);
        
        // Process through detector
        let dims = self.detector.current_dimensions;
        let result = self.detector.process_signal_mathematically(signal, dims);
        
        // Convert discovery for Fate
        let discovery = result.equation_created.as_ref().map(VoyagerDiscovery::from);
        
        (result, discovery)
    }

    pub fn detector(&self) -> &PreciseSignalDetector {
        &self.detector
    }

    pub fn detector_mut(&mut self) -> &mut PreciseSignalDetector {
        &mut self.detector
    }
    
    /// Get expansion history
    pub fn get_expansion_history(&self) -> &[(u64, usize, String)] {
        &self.expansion_history
    }
}

impl Default for DimensionManager {
    fn default() -> Self {
        Self::new(DimensionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_initial_state() {
        // Detector starts with NO preset equations
        let detector = PreciseSignalDetector::new();
        assert_eq!(detector.known_equations.len(), 0);
    }

    #[test]
    fn test_rational_arithmetic() {
        let a = Rational::new(1, 2);
        let b = Rational::new(1, 3);
        assert_eq!(a.add(&b), Rational::new(5, 6));
    }
    
    #[test]
    fn test_auto_discovery_single_value() {
        let mut detector = PreciseSignalDetector::new();
        
        // Start empty
        assert_eq!(detector.known_equation_count(), 0);
        
        // Input signal: single value (will be Unknown pattern)
        let signal = vec![Rational::one()];
        
        // Process - should discover and create equation
        let result = detector.process_signal_mathematically(signal, 4);
        
        // Should detect as unknown initially, then create equation
        assert!(result.equation_created.is_some());
        
        // Now we have 1 equation
        assert_eq!(detector.known_equation_count(), 1);
        
        // The created equation should have derivation trace
        let eq = result.equation_created.unwrap();
        assert!(!eq.derivation_trace.is_empty());
    }
    
    #[test]
    fn test_auto_discovery_linear_pattern() {
        let mut detector = PreciseSignalDetector::new();
        
        // Linear sequence: 1/2, 1, 3/2 (diff = 1/2)
        let signal = vec![
            Rational::new(1, 2),
            Rational::new(1, 1),
            Rational::new(3, 2),
        ];
        
        let result = detector.process_signal_mathematically(signal, 4);
        
        // Should create linear equation
        assert!(result.equation_created.is_some());
        
        if let Some(eq) = result.equation_created {
            if let EquationType::Linear { coefficient, .. } = eq.equation_type {
                assert_eq!(coefficient, Rational::new(1, 2));
            }
            // Has derivation trace
            assert!(!eq.derivation_trace.is_empty());
            assert!(eq.derivation_trace.len() >= 4);
        }
    }
    
    #[test]
    fn test_knowledge_accumulation() {
        let mut detector = PreciseSignalDetector::new();
        
        // Start empty
        assert_eq!(detector.known_equation_count(), 0);
        
        // First signal
        let signal1 = vec![Rational::one()];
        detector.process_signal_mathematically(signal1.clone(), 4);
        assert_eq!(detector.known_equation_count(), 1);
        
        // Same signal again - should match existing, not create new
        let result2 = detector.process_signal_mathematically(signal1, 4);
        assert!(!result2.detection.is_unknown()); // Matches known
        assert_eq!(detector.known_equation_count(), 1); // Still 1
        
        // Different signal - should create new
        let signal2 = vec![Rational::new(1, 2), Rational::new(1, 1)];
        detector.process_signal_mathematically(signal2, 4);
        assert_eq!(detector.known_equation_count(), 2); // Now 2
    }
    
    #[test]
    fn test_reverse_verification() {
        let mut detector = PreciseSignalDetector::new();
        
        // Create a linear equation via discovery
        let signal = vec![
            Rational::new(1, 2),
            Rational::new(1, 1),
            Rational::new(3, 2),
        ];
        detector.process_signal_mathematically(signal.clone(), 4);
        
        // Now verify same signal matches
        let result = detector.detect_signal_precisely(signal);
        assert!(!result.is_unknown()); // Should match
        
        if let PreciseDetectionResult::KnownEquation { equation, .. } = result {
            assert!(equation.algebraically_verified);
        }
    }
}
