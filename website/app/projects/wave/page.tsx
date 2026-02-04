import Header from '@/components/Header'
import Footer from '@/components/Footer'
import Link from 'next/link'

export default function WaveProjectPage() {
  return (
    <>
      <Header />
      
      <main className="pt-20">
        {/* Hero */}
        <section className="bg-gradient-to-b from-bg-secondary to-bg-primary py-16">
          <div className="max-w-4xl mx-auto px-6">
            <div className="flex items-center gap-2 text-sm text-text-tertiary mb-4">
              <Link href="/projects" className="hover:text-accent-primary">Projects</Link>
              <span>/</span>
              <span>Wave</span>
            </div>
            
            <div className="flex items-center gap-4 mb-4">
              <div className="w-12 h-12 rounded-xl bg-accent-primary/10 flex items-center justify-center">
                <svg className="w-6 h-6 text-accent-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M2 12c0-3 2.5-6 6-6s4 3 4 6 2.5 6 6 6 6-3 6-6" />
                </svg>
              </div>
              <span className="px-2 py-1 text-xs font-medium bg-green-500/10 text-green-500 rounded">Active</span>
            </div>
            
            <h1 className="text-4xl md:text-5xl font-bold text-text-primary mb-4">Wave</h1>
            <p className="text-xl text-text-secondary mb-6">
              Adaptive Programming Language with Runtime Optimization
            </p>
            
            <div className="flex flex-wrap gap-3">
              <a href="https://github.com/Rogue-Intelligence-INC/Valhalla/tree/main/wave" target="_blank" rel="noopener noreferrer" className="btn btn-primary">
                <svg className="w-4 h-4" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>
                View Source
              </a>
            </div>
          </div>
        </section>

        {/* Overview */}
        <section className="py-16">
          <div className="max-w-4xl mx-auto px-6">
            <h2 className="text-2xl font-bold text-text-primary mb-6">Overview</h2>
            <p className="text-text-secondary mb-4 leading-relaxed">
              Wave is a self-hosting programming language that compiles directly to native machine code. 
              Unlike interpreted languages, Wave generates ELF64 executables without any external toolchain 
              (no LLVM, no GCC). The compiler itself is written in Wave, achieving complete technical independence.
            </p>
            <p className="text-text-secondary leading-relaxed">
              The language features runtime self-optimization: code performance improves based on actual 
              execution patterns, automatically adapting to workload characteristics and hardware capabilities.
            </p>
          </div>
        </section>

        {/* How It Works */}
        <section className="py-16 bg-bg-secondary">
          <div className="max-w-4xl mx-auto px-6">
            <h2 className="text-2xl font-bold text-text-primary mb-8">How It Works</h2>
            
            {/* Compilation Pipeline */}
            <div className="mb-12">
              <h3 className="text-xl font-semibold text-text-primary mb-4">Compilation Pipeline</h3>
              <div className="bg-bg-primary border border-border rounded-xl p-6 mb-4">
                <pre className="text-sm text-text-secondary font-mono overflow-x-auto">{`┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ Source Code │───▶│   Parser    │───▶│  Analyzer   │───▶│  Codegen    │
│   (.wave)   │    │  (Lexer+AST)│    │ (Type+Opt)  │    │ (x86-64)    │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
                                                                │
                                                                ▼
                                                         ┌─────────────┐
                                                         │ ELF64 Binary│
                                                         │  (~9KB avg) │
                                                         └─────────────┘`}</pre>
              </div>
              <div className="grid md:grid-cols-2 gap-4">
                <div className="p-4 bg-bg-primary border border-border rounded-xl">
                  <h4 className="font-semibold text-text-primary mb-2">1. Lexical Analysis</h4>
                  <p className="text-sm text-text-secondary">
                    Source tokenization with position tracking. Single-pass scanning with O(n) complexity.
                  </p>
                </div>
                <div className="p-4 bg-bg-primary border border-border rounded-xl">
                  <h4 className="font-semibold text-text-primary mb-2">2. Parsing & AST</h4>
                  <p className="text-sm text-text-secondary">
                    Recursive descent parser builds Abstract Syntax Tree. Supports functions, loops, conditionals.
                  </p>
                </div>
                <div className="p-4 bg-bg-primary border border-border rounded-xl">
                  <h4 className="font-semibold text-text-primary mb-2">3. Semantic Analysis</h4>
                  <p className="text-sm text-text-secondary">
                    Type inference, scope resolution, and optimization opportunity detection.
                  </p>
                </div>
                <div className="p-4 bg-bg-primary border border-border rounded-xl">
                  <h4 className="font-semibold text-text-primary mb-2">4. Native Codegen</h4>
                  <p className="text-sm text-text-secondary">
                    Direct x86-64 machine code emission. Generates complete ELF64 executable with headers.
                  </p>
                </div>
              </div>
            </div>

            {/* Runtime Optimization */}
            <div className="mb-12">
              <h3 className="text-xl font-semibold text-text-primary mb-4">Runtime Optimization System</h3>
              <p className="text-text-secondary mb-4">
                Wave includes an optional runtime optimizer that monitors execution and improves code dynamically:
              </p>
              <div className="bg-bg-primary border border-border rounded-xl p-6 mb-4">
                <pre className="text-sm text-text-secondary font-mono overflow-x-auto">{`┌────────────────────────────────────────────────────────────────┐
│                    Runtime Optimization Loop                    │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│   Execute ──▶ Monitor ──▶ Analyze ──▶ Optimize ──▶ Execute    │
│      │                                     │                   │
│      │         ┌─────────────────┐         │                   │
│      └────────▶│ Profile Data    │◀────────┘                   │
│                │ - Hot paths     │                              │
│                │ - Branch stats  │                              │
│                │ - Memory access │                              │
│                └─────────────────┘                              │
│                                                                │
│   When marginal gains < threshold ──▶ Collapse to static code  │
└────────────────────────────────────────────────────────────────┘`}</pre>
              </div>
              <div className="space-y-3">
                <div className="p-4 bg-bg-primary border border-border rounded-xl">
                  <h4 className="font-semibold text-text-primary mb-1">Hot Path Detection</h4>
                  <p className="text-sm text-text-secondary">
                    Identifies frequently executed code paths and prioritizes their optimization.
                  </p>
                </div>
                <div className="p-4 bg-bg-primary border border-border rounded-xl">
                  <h4 className="font-semibold text-text-primary mb-1">Branch Prediction Hints</h4>
                  <p className="text-sm text-text-secondary">
                    Learns branch probabilities and reorders code for better CPU pipeline utilization.
                  </p>
                </div>
                <div className="p-4 bg-bg-primary border border-border rounded-xl">
                  <h4 className="font-semibold text-text-primary mb-1">Automatic Staticization</h4>
                  <p className="text-sm text-text-secondary">
                    When optimization reaches diminishing returns, dynamic code "collapses" to optimized static code.
                  </p>
                </div>
              </div>
            </div>

            {/* Memory Management */}
            <div className="mb-12">
              <h3 className="text-xl font-semibold text-text-primary mb-4">Memory Management</h3>
              <p className="text-text-secondary mb-4">
                Wave uses a four-pool memory allocation system. Each pool is optimized for different access patterns:
              </p>
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-border">
                      <th className="py-3 px-4 text-left font-semibold text-text-primary">Pool</th>
                      <th className="py-3 px-4 text-left font-semibold text-text-primary">Optimized For</th>
                      <th className="py-3 px-4 text-left font-semibold text-text-primary">Use Case</th>
                    </tr>
                  </thead>
                  <tbody className="text-text-secondary text-sm">
                    <tr className="border-b border-border/50">
                      <td className="py-3 px-4 font-mono">Pool 0</td>
                      <td className="py-3 px-4">Large, dense allocations</td>
                      <td className="py-3 px-4">Compiled code, GPU buffers, large arrays</td>
                    </tr>
                    <tr className="border-b border-border/50">
                      <td className="py-3 px-4 font-mono">Pool 1</td>
                      <td className="py-3 px-4">Indexed, relational data</td>
                      <td className="py-3 px-4">Strings, hash maps, graph structures</td>
                    </tr>
                    <tr className="border-b border-border/50">
                      <td className="py-3 px-4 font-mono">Pool 2</td>
                      <td className="py-3 px-4">Parallel partitioned access</td>
                      <td className="py-3 px-4">I/O buffers, network packets, work queues</td>
                    </tr>
                    <tr className="border-b border-border/50">
                      <td className="py-3 px-4 font-mono">Pool 3</td>
                      <td className="py-3 px-4">Fast allocation/deallocation</td>
                      <td className="py-3 px-4">Temporary variables, stack frames, short-lived objects</td>
                    </tr>
                  </tbody>
                </table>
              </div>
              <p className="text-sm text-text-tertiary mt-4">
                Pool selection is automatic based on allocation size, access pattern hints, and runtime profiling.
              </p>
            </div>
          </div>
        </section>

        {/* Performance */}
        <section className="py-16">
          <div className="max-w-4xl mx-auto px-6">
            <h2 className="text-2xl font-bold text-text-primary mb-8">Performance</h2>
            
            <div className="grid md:grid-cols-4 gap-6 mb-8">
              <div className="p-6 bg-bg-secondary border border-border rounded-xl text-center">
                <div className="text-3xl font-bold text-accent-primary mb-2">19x</div>
                <div className="text-sm text-text-secondary">vs Python</div>
              </div>
              <div className="p-6 bg-bg-secondary border border-border rounded-xl text-center">
                <div className="text-3xl font-bold text-accent-primary mb-2">&lt;3ms</div>
                <div className="text-sm text-text-secondary">Compile Time</div>
              </div>
              <div className="p-6 bg-bg-secondary border border-border rounded-xl text-center">
                <div className="text-3xl font-bold text-accent-primary mb-2">~35KB</div>
                <div className="text-sm text-text-secondary">Compiler Size</div>
              </div>
              <div className="p-6 bg-bg-secondary border border-border rounded-xl text-center">
                <div className="text-3xl font-bold text-accent-primary mb-2">&lt;1MB</div>
                <div className="text-sm text-text-secondary">Runtime Memory</div>
              </div>
            </div>

            <div className="bg-bg-secondary border border-border rounded-xl p-6">
              <h3 className="font-semibold text-text-primary mb-4">Benchmark Results</h3>
              <pre className="text-sm text-text-secondary font-mono overflow-x-auto">{`Test: Recursive Fibonacci + String Operations + Array Sort

Wave (self-hosted):     0.001s    Memory: <1MB
Python 3.13:            0.019s    Memory: ~30MB

Speedup:                19x faster
Memory efficiency:      30x lower`}</pre>
            </div>
          </div>
        </section>

        {/* Language Features */}
        <section className="py-16 bg-bg-secondary">
          <div className="max-w-4xl mx-auto px-6">
            <h2 className="text-2xl font-bold text-text-primary mb-8">Language Features</h2>
            
            <div className="bg-bg-primary border border-border rounded-xl overflow-hidden mb-6">
              <div className="px-4 py-2 border-b border-border bg-bg-secondary">
                <span className="text-sm text-text-tertiary">example.wave</span>
              </div>
              <pre className="p-4 overflow-x-auto text-sm text-text-secondary">{`// Functions with recursion
fn fibonacci n {
    when n <= 1 {
        return n
    }
    return fibonacci(n - 1) + fibonacci(n - 2)
}

// Variables and loops
fn main {
    let result = 0
    let i = 0
    
    loop {
        when i >= 10 { break }
        result = result + fibonacci(i)
        i = i + 1
    }
    
    out "Sum of first 10 fibonacci: "
    out result
}

// Pipeline syntax for data flow
let processed = data -> filter(x > 0) -> map(x * 2) -> sum()`}</pre>
            </div>

            <div className="grid md:grid-cols-2 gap-4">
              <div className="p-4 bg-bg-primary border border-border rounded-xl">
                <h4 className="font-semibold text-text-primary mb-2">Core Features</h4>
                <ul className="text-sm text-text-secondary space-y-1">
                  <li>• Dynamic typing with inference</li>
                  <li>• First-class functions</li>
                  <li>• Recursion support</li>
                  <li>• Block scoping</li>
                </ul>
              </div>
              <div className="p-4 bg-bg-primary border border-border rounded-xl">
                <h4 className="font-semibold text-text-primary mb-2">Advanced Features</h4>
                <ul className="text-sm text-text-secondary space-y-1">
                  <li>• Pipeline operator (-&gt;)</li>
                  <li>• Direct memory access (poke/peek)</li>
                  <li>• System calls (syscall)</li>
                  <li>• Platform adaptation</li>
                </ul>
              </div>
            </div>
          </div>
        </section>

        {/* Back link */}
        <section className="py-8">
          <div className="max-w-4xl mx-auto px-6">
            <Link href="/projects" className="text-accent-primary hover:underline">
              ← Back to all projects
            </Link>
          </div>
        </section>
      </main>

      <Footer />
    </>
  )
}
