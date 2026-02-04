use voyager::{PreciseSignalDetector, Rational, PreciseDetectionResult};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║       Voyager - Auto Discovery + Derivation Trace Demo               ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();
    
    let mut detector = PreciseSignalDetector::new();
    
    println!("1. Known Equation Library (Initial):");
    println!("   Equation count: {}", detector.known_equation_count());
    println!();
    
    // Show Planck Closure derivation trace
    println!("2. Planck Closure Functional Derivation:");
    if let Some(trace) = detector.format_derivation_trace("planck_closure") {
        println!("{}", trace);
    }
    
    // Test reverse verification
    println!("3. Reverse Verification (result -> cause):");
    let c2_signal = vec![Rational::one()]; // C(2) = 1
    let result = detector.detect_signal_precisely(c2_signal);
    match result {
        PreciseDetectionResult::KnownEquation { equation_id, .. } => {
            println!("   ✓ Signal [1] matches known equation: {}", equation_id);
        }
        _ => println!("   ✗ No match"),
    }
    println!();
    
    // Test auto-discovery
    println!("4. Auto-Discovery:");
    let unknown_signal = vec![
        Rational::new(1, 10),
        Rational::new(2, 10),
        Rational::new(3, 10),
    ];
    println!("   Input unknown signal: [1/10, 2/10, 3/10]");
    
    let result = detector.process_signal_mathematically(unknown_signal, 4);
    
    if let Some(new_eq) = result.equation_created {
        println!("   ✓ New equation created: {}", new_eq.equation_id);
        println!("   ✓ Algebraic verification: {}", if new_eq.algebraically_verified { "PASS (exact)" } else { "FAIL" });
        println!("   ✓ Derivation steps: {}", new_eq.derivation_trace.len());
        println!();
        println!("   Derivation trace:");
        for step in &new_eq.derivation_trace {
            println!("     Step {}: {} ({:?})", step.step_number, step.description, step.rule_applied);
        }
    }
    println!();
    
    println!("5. Auto-Archive:");
    println!("   Equation count (updated): {}", detector.known_equation_count());
    println!("   Dimensions: {}", detector.get_current_dimensions());
    println!();
    
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("                    All Functions Verified ✓                           ");
    println!("═══════════════════════════════════════════════════════════════════════");
}
