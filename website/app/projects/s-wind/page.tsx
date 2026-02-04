import Header from '@/components/Header'
import Footer from '@/components/Footer'
import Link from 'next/link'

export default function SWindProjectPage() {
  return (
    <>
      <Header />
      
      <main className="pt-20">
        {/* Hero */}
        <section className="bg-gradient-to-b from-paper-200 to-paper-100 py-16">
          <div className="max-w-4xl mx-auto px-6">
            <div className="flex items-center gap-2 text-sm text-ink-500 mb-4">
              <Link href="/projects" className="hover:text-accent-primary">Research</Link>
              <span>/</span>
              <span>S-wind</span>
            </div>
            <div className="flex items-start gap-4 mb-6">
              <div className="w-16 h-16 rounded-lg bg-accent-primary/10 flex items-center justify-center text-accent-primary flex-shrink-0">
                <svg className="w-8 h-8" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4zM17 14v6M14 17h6" />
                </svg>
              </div>
              <div>
                <h1 className="text-4xl md:text-5xl font-serif font-bold text-ink-900">
                  S-wind
                </h1>
                <p className="text-xl text-ink-600 mt-2">
                  Exact Algebraic Exploration Framework
                </p>
              </div>
            </div>
            <div className="flex flex-wrap gap-2">
              <span className="bg-amber-50 text-amber-700 border border-amber-200 px-3 py-1 rounded text-sm font-medium">
                Experimental
              </span>
              <span className="academic-tag">Exact Arithmetic</span>
              <span className="academic-tag">i128/i128 Rationals</span>
              <span className="academic-tag">Zero Error</span>
            </div>
          </div>
        </section>

        {/* Content */}
        <section className="py-12 bg-paper-100">
          <div className="max-w-4xl mx-auto px-6">
            {/* Overview */}
            <div className="prose max-w-none mb-12">
              <h2>Overview</h2>
              <p>
                <strong>S-wind</strong> is a spontaneous algebraic exploration framework for mathematics 
                and physics research. It operates entirely in the algebraic domain using <strong>exact 
                rational arithmetic</strong> (i128/i128), guaranteeing zero rounding error.
              </p>
              <p>
                Unlike traditional numerical computing which accumulates floating-point errors, S-wind 
                maintains perfect precision through all calculations, making it ideal for theoretical 
                physics simulations and mathematical constant computation.
              </p>
            </div>

            {/* Key Principles */}
            <div className="card mb-8">
              <h3 className="font-serif text-xl font-semibold text-ink-900 mb-4">Key Principles</h3>
              <table className="w-full text-sm">
                <thead>
                  <tr className="text-left">
                    <th className="py-2">Principle</th>
                    <th className="py-2">Description</th>
                  </tr>
                </thead>
                <tbody>
                  <tr><td className="py-2 font-medium">Exact Arithmetic</td><td>i128/i128 rationals with GCD reduction, zero floating-point error</td></tr>
                  <tr><td className="py-2 font-medium">Continuous Exploration</td><td>Runs indefinitely, not on-demand batch processing</td></tr>
                  <tr><td className="py-2 font-medium">Constraint-Driven</td><td>Discoveries emerge from algebraic structure, not optimization targets</td></tr>
                  <tr><td className="py-2 font-medium">Self-Evolving</td><td>Discoveries trigger internal space evolution</td></tr>
                  <tr><td className="py-2 font-medium">Extensible</td><td>New physics laws can be added dynamically</td></tr>
                </tbody>
              </table>
            </div>

            {/* Architecture */}
            <div className="mb-12">
              <h2 className="text-2xl font-serif font-bold text-ink-900 mb-6">Architecture</h2>
              <div className="architecture-diagram text-xs">
{`┌─────────────────────────────────────────────────────────────────────────┐
│                          S-wind Architecture                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐               │
│  │    Fate     │────▶│   Voyager   │────▶│  Knowledge  │               │
│  │   (Core)    │     │  (Detect)   │     │    Base     │               │
│  └─────────────┘     └─────────────┘     └─────────────┘               │
│         │                   │                   │                       │
│         ▼                   ▼                   ▼                       │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐               │
│  │    Space    │     │    Quad     │     │  Symbolic   │               │
│  │ (Adaptive)  │     │  (Physics)  │     │   (Expr)    │               │
│  └─────────────┘     └─────────────┘     └─────────────┘               │
│                                                                         │
│  Time System: signal_tick (deformation signal count), NOT natural time  │
└─────────────────────────────────────────────────────────────────────────┘`}
              </div>
            </div>

            {/* Code Examples */}
            <div className="mb-12">
              <h2 className="text-2xl font-serif font-bold text-ink-900 mb-6">Code Examples</h2>
              
              <div className="space-y-6">
                <div className="card">
                  <h4 className="font-semibold text-ink-800 mb-3">Exact Rational Arithmetic</h4>
                  <pre className="text-sm">{`use s_wind::Rational;

let a = Rational::new(1, 3);  // 1/3
let b = Rational::new(1, 6);  // 1/6
let sum = a + b;              // 1/2 (exact, not 0.49999...)

assert_eq!(sum, Rational::new(1, 2));`}</pre>
                </div>

                <div className="card">
                  <h4 className="font-semibold text-ink-800 mb-3">Spontaneous Exploration</h4>
                  <pre className="text-sm">{`use s_wind::controller::{ExplorationController, ExplorationEvent};

let controller = ExplorationController::new();
controller.start_exploration();

// System runs indefinitely:
// - Fate generates/receives signals
// - Voyager detects patterns
// - Unknown patterns → new equations → archived
// - Knowledge base grows continuously`}</pre>
                </div>

                <div className="card">
                  <h4 className="font-semibold text-ink-800 mb-3">Discovery-Driven Container Creation</h4>
                  <pre className="text-sm">{`use s_wind::space::{SWindSpace, DimensionChange};

let mut space = SWindSpace::new();

// When Fate discovers an unknown phenomenon
let discovery = space.on_discovery(&signal_data, "planck_closure");

// A NEW container is created with:
// - Unique ID based on discovery
// - Fractional dimension adapted to the pattern
// - Dimension can INCREASE or DECREASE`}</pre>
                </div>
              </div>
            </div>

            {/* Quad System */}
            <div className="mb-12">
              <h2 className="text-2xl font-serif font-bold text-ink-900 mb-6">Quad Physics System</h2>
              <p className="text-ink-600 mb-6">
                Four fundamental physics simulation components using exact rational arithmetic:
              </p>
              <div className="grid md:grid-cols-2 gap-6">
                <div className="card">
                  <h4 className="font-semibold text-ink-800 mb-2">I. Gravitational Field</h4>
                  <p className="text-sm text-ink-600 mb-2">Physics: F = GM/r², rₛ = 2GM/c²</p>
                  <p className="text-xs text-ink-500">Schwarzschild radius, escape velocity, event horizon detection</p>
                </div>
                <div className="card">
                  <h4 className="font-semibold text-ink-800 mb-2">II. Quantum Field System</h4>
                  <p className="text-sm text-ink-600 mb-2">Physics: Eₙ = ℏω(n + ½)</p>
                  <p className="text-xs text-ink-500">Zero-point energy, mode energy, quantum field signals</p>
                </div>
                <div className="card">
                  <h4 className="font-semibold text-ink-800 mb-2">III. Thermal Radiation</h4>
                  <p className="text-sm text-ink-600 mb-2">Physics: P = σAT⁴</p>
                  <p className="text-xs text-ink-500">Stefan-Boltzmann law, radiated power computation</p>
                </div>
                <div className="card">
                  <h4 className="font-semibold text-ink-800 mb-2">IV. Fundamental Coupling</h4>
                  <p className="text-sm text-ink-600 mb-2">Physics: αₑff = (αg + αem + αs)/3</p>
                  <p className="text-xs text-ink-500">Gravitational, electromagnetic, strong force coordination</p>
                </div>
              </div>
            </div>

            {/* Knowledge Base */}
            <div className="mb-12">
              <h2 className="text-2xl font-serif font-bold text-ink-900 mb-6">Pre-loaded Knowledge Base</h2>
              <div className="card">
                <div className="grid md:grid-cols-2 gap-6">
                  <div>
                    <h4 className="font-semibold text-ink-800 mb-2">Physical Laws (17)</h4>
                    <ul className="text-sm text-ink-600 space-y-1">
                      <li>• Newton's Laws & Gravitation</li>
                      <li>• Maxwell's Equations</li>
                      <li>• Schrödinger Equation</li>
                      <li>• Einstein Field Equations</li>
                      <li>• Planck's Radiation Law</li>
                      <li>• Heisenberg Uncertainty</li>
                    </ul>
                  </div>
                  <div>
                    <h4 className="font-semibold text-ink-800 mb-2">Constants & Theorems</h4>
                    <ul className="text-sm text-ink-600 space-y-1">
                      <li>• 20 Physical constants (NIST CODATA)</li>
                      <li>• 15 Mathematical constants</li>
                      <li>• 16 Fundamental theorems</li>
                      <li>• Extensible discovery archive</li>
                    </ul>
                  </div>
                </div>
              </div>
            </div>

            {/* Benchmarks */}
            <div className="mb-12">
              <h2 className="text-2xl font-serif font-bold text-ink-900 mb-6">Verification Results</h2>
              <table className="w-full text-sm">
                <thead>
                  <tr>
                    <th className="text-left py-2">Constant/Theorem</th>
                    <th className="text-left py-2">Computed</th>
                    <th className="text-left py-2">Status</th>
                  </tr>
                </thead>
                <tbody>
                  <tr>
                    <td className="py-2">ζ(2) S₂₀</td>
                    <td className="font-mono text-xs">17299975731542641/10838475198270720</td>
                    <td className="text-green-600">✓ Converging to π²/6</td>
                  </tr>
                  <tr>
                    <td className="py-2">Twin Prime C₂</td>
                    <td className="font-mono text-xs">exact fraction (9 primes)</td>
                    <td className="text-green-600">✓ Exact</td>
                  </tr>
                  <tr>
                    <td className="py-2">Planck C(2) = 1</td>
                    <td className="font-mono text-xs">Horizon Limit verified</td>
                    <td className="text-green-600">✓ Verified</td>
                  </tr>
                  <tr>
                    <td className="py-2">Self-Duality F(x)=F̃(1/x)</td>
                    <td className="font-mono text-xs">Algebraically proven</td>
                    <td className="text-green-600">✓ Verified</td>
                  </tr>
                </tbody>
              </table>
            </div>

            {/* Back link */}
            <div className="border-t border-paper-300 pt-8">
              <Link href="/projects" className="text-accent-link font-medium hover:underline">
                ← Back to all projects
              </Link>
            </div>
          </div>
        </section>
      </main>

      <Footer />
    </>
  )
}
