import Header from '@/components/Header'
import Footer from '@/components/Footer'
import Link from 'next/link'

export default function YakshaCipherPage() {
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
              <span>Yaksha Cipher</span>
            </div>
            
            <div className="flex items-center gap-4 mb-4">
              <div className="w-12 h-12 rounded-xl bg-accent-primary/10 flex items-center justify-center">
                <svg className="w-6 h-6 text-accent-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
                  <path d="M7 11V7a5 5 0 0 1 10 0v4" />
                </svg>
              </div>
              <span className="px-2 py-1 text-xs font-medium bg-green-500/10 text-green-500 rounded">Active</span>
            </div>
            
            <h1 className="text-4xl md:text-5xl font-bold text-text-primary mb-4">Yaksha Cipher</h1>
            <p className="text-xl text-text-secondary mb-6">
              High-performance symmetric encryption with GPU acceleration
            </p>
            
            <div className="flex flex-wrap gap-3">
              <a href="https://github.com/Rogue-Intelligence-INC/Valhalla/tree/main/valhalla-crypto/yaksha-cipher" target="_blank" rel="noopener noreferrer" className="btn btn-primary">
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
              Yaksha Cipher is a symmetric encryption framework designed for high-throughput applications.
              It achieves up to 1.4 GB/s throughput through GPU acceleration while maintaining strong
              cryptographic properties including ~50% avalanche effect (near the theoretical optimum).
            </p>
            <p className="text-text-secondary leading-relaxed">
              The cipher supports three encryption modes optimized for different use cases, with
              stateful authenticated encryption (AEAD) for applications requiring integrity verification.
            </p>
          </div>
        </section>

        {/* How It Works */}
        <section className="py-16 bg-bg-secondary">
          <div className="max-w-4xl mx-auto px-6">
            <h2 className="text-2xl font-bold text-text-primary mb-8">How It Works</h2>
            
            {/* Encryption Pipeline */}
            <div className="mb-12">
              <h3 className="text-xl font-semibold text-text-primary mb-4">Two-Phase Encryption</h3>
              <div className="bg-bg-primary border border-border rounded-xl p-6 mb-4">
                <pre className="text-sm text-text-secondary font-mono overflow-x-auto">{`┌─────────────────────────────────────────────────────────────────┐
│                    Yaksha Encryption Pipeline                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Plaintext ──▶ [Phase 1: Parallel Transform] ──▶ Intermediate   │
│                      │                                          │
│                      ▼                                          │
│         ┌─────────────────────────┐                             │
│         │ For each byte (parallel):│                            │
│         │ 1. Derive position key   │                            │
│         │ 2. Compute entropy value │                            │
│         │ 3. Branch on entropy:    │                            │
│         │    High → Multiply + Rot │                            │
│         │    Low  → Add + Rot      │                            │
│         │ 4. XOR with key material │                            │
│         └─────────────────────────┘                             │
│                      │                                          │
│                      ▼                                          │
│  Intermediate ──▶ [Phase 2: Sequential Cascade] ──▶ Ciphertext  │
│                      │                                          │
│         ┌─────────────────────────┐                             │
│         │ For each byte (serial): │                             │
│         │ 1. XOR with cascade     │                             │
│         │ 2. Update cascade state │                             │
│         │ 3. Rotate cascade       │                             │
│         └─────────────────────────┘                             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘`}</pre>
              </div>
              
              <div className="grid md:grid-cols-2 gap-4">
                <div className="p-4 bg-bg-primary border border-border rounded-xl">
                  <h4 className="font-semibold text-text-primary mb-2">Phase 1: Parallel Transform</h4>
                  <p className="text-sm text-text-secondary">
                    Each byte is transformed independently using position-derived keys and entropy-based
                    branching. This phase is fully parallelizable across CPU cores or GPU threads.
                  </p>
                </div>
                <div className="p-4 bg-bg-primary border border-border rounded-xl">
                  <h4 className="font-semibold text-text-primary mb-2">Phase 2: Sequential Cascade</h4>
                  <p className="text-sm text-text-secondary">
                    A chained XOR operation propagates changes across all bytes, ensuring that any
                    input change affects the entire output (avalanche effect).
                  </p>
                </div>
              </div>
            </div>

            {/* Key Operations */}
            <div className="mb-12">
              <h3 className="text-xl font-semibold text-text-primary mb-4">Core Operations (ARX)</h3>
              <p className="text-text-secondary mb-4">
                Yaksha uses ARX operations (Add-Rotate-XOR), which are efficient on all CPUs and provide
                good diffusion properties:
              </p>
              <div className="bg-bg-primary border border-border rounded-xl p-6">
                <pre className="text-sm text-text-secondary font-mono overflow-x-auto">{`// Entropy-based transformation branch
let entropy = derive_entropy(key, position);  // Deterministic

if entropy > 127 {
    // High entropy: multiplicative mixing
    let pk_odd = pk | 1;  // Ensure odd (invertible mod 256)
    state = state.wrapping_mul(pk_odd);
    state = state.rotate_left(rotation);
} else {
    // Low entropy: additive mixing  
    state = state.wrapping_add(pk);
    state = state.rotate_right(rotation);
}

// Final XOR mixing
output = state ^ key_material;`}</pre>
              </div>
              <p className="text-sm text-text-tertiary mt-4">
                All operations are invertible mod 256, ensuring perfect decryption. Odd multiplication
                always has a modular inverse.
              </p>
            </div>

            {/* GPU Acceleration */}
            <div className="mb-12">
              <h3 className="text-xl font-semibold text-text-primary mb-4">GPU Acceleration</h3>
              <div className="bg-bg-primary border border-border rounded-xl p-6 mb-4">
                <pre className="text-sm text-text-secondary font-mono overflow-x-auto">{`┌────────────────────────────────────────────────────────────────┐
│                    CPU + GPU Hybrid Pipeline                    │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│   GPU (wgpu/Vulkan)              CPU (rayon parallel)          │
│   ┌─────────────────┐           ┌─────────────────────────┐    │
│   │ Container Params │           │ Phase 1: Per-byte       │    │
│   │ Computation      │──────────▶│ Transform (parallel)    │    │
│   │ (WGSL shader)    │           └─────────────────────────┘    │
│   └─────────────────┘                      │                   │
│                                            ▼                   │
│                                 ┌─────────────────────────┐    │
│                                 │ Phase 2: Cascade        │    │
│                                 │ (sequential)            │    │
│                                 └─────────────────────────┘    │
│                                                                │
└────────────────────────────────────────────────────────────────┘`}</pre>
              </div>
              <p className="text-text-secondary">
                The GPU computes container parameters (derived from key) in parallel, while the CPU
                handles the main encryption logic. This hybrid approach achieves 1.4 GB/s on RTX 4090.
              </p>
            </div>

            {/* AEAD Protocol */}
            <div className="mb-12">
              <h3 className="text-xl font-semibold text-text-primary mb-4">Authenticated Encryption (AEAD)</h3>
              <div className="bg-bg-primary border border-border rounded-xl p-6 mb-4">
                <pre className="text-sm text-text-secondary font-mono overflow-x-auto">{`┌─────────────────────────────────────────────────────────────┐
│                    AEAD Protocol Stack                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   YakshaKdf (Key Derivation)                                │
│   ├─ Derives session keys from master key                   │
│   ├─ Evolves internal state per operation                   │
│   └─ Provides session differentiation                       │
│                      │                                      │
│                      ▼                                      │
│   Yaksha Encryption (Deterministic Primitive)               │
│   ├─ Same key + plaintext = same ciphertext                │
│   └─ Session uniqueness from KDF, not cipher               │
│                      │                                      │
│                      ▼                                      │
│   YakshaMac (Authentication)                                │
│   ├─ Computes authentication tag                           │
│   ├─ Binds ciphertext + AAD                                │
│   └─ Detects tampering on decryption                       │
│                                                             │
└─────────────────────────────────────────────────────────────┘`}</pre>
              </div>
              <div className="p-4 bg-amber-500/10 border border-amber-500/20 rounded-xl">
                <h4 className="font-semibold text-amber-500 mb-1">Note on Determinism</h4>
                <p className="text-sm text-text-secondary">
                  The cipher primitive is deterministic (same input → same output). Session differentiation
                  comes from the stateful KDF layer, not from nonces in the cipher itself.
                </p>
              </div>
            </div>
          </div>
        </section>

        {/* Encryption Modes */}
        <section className="py-16">
          <div className="max-w-4xl mx-auto px-6">
            <h2 className="text-2xl font-bold text-text-primary mb-8">Encryption Modes</h2>
            
            <div className="space-y-6">
              <div className="p-6 bg-bg-secondary border border-border rounded-xl">
                <div className="flex items-center gap-3 mb-3">
                  <span className="text-2xl">⚡</span>
                  <h3 className="text-lg font-semibold text-text-primary">Core Mode</h3>
                  <span className="text-xs px-2 py-1 bg-blue-500/10 text-blue-500 rounded">Fastest</span>
                </div>
                <p className="text-text-secondary mb-3">
                  Basic encryption with fixed parameters. Highest throughput (~1000 MB/s CPU).
                </p>
                <div className="text-sm text-text-tertiary">
                  Best for: Bulk data encryption, streaming, when maximum speed is priority.
                </div>
              </div>
              
              <div className="p-6 bg-bg-secondary border border-border rounded-xl">
                <div className="flex items-center gap-3 mb-3">
                  <span className="text-2xl">🎯</span>
                  <h3 className="text-lg font-semibold text-text-primary">Parametric Mode</h3>
                  <span className="text-xs px-2 py-1 bg-green-500/10 text-green-500 rounded">GPU Accelerated</span>
                </div>
                <p className="text-text-secondary mb-3">
                  Key-derived transformation parameters. The "dimension" parameter (1.5-3.5) influences
                  rotation amounts and mixing factors throughout the transformation.
                </p>
                <div className="text-sm text-text-tertiary">
                  Best for: General purpose, GPU-available systems, balanced security/speed.
                </div>
              </div>
              
              <div className="p-6 bg-bg-secondary border border-border rounded-xl">
                <div className="flex items-center gap-3 mb-3">
                  <span className="text-2xl">🧠</span>
                  <h3 className="text-lg font-semibold text-text-primary">Neural Mode</h3>
                  <span className="text-xs px-2 py-1 bg-purple-500/10 text-purple-500 rounded">Complex</span>
                </div>
                <p className="text-text-secondary mb-3">
                  Inspired by neural network activation patterns. Uses "membrane thickness" and
                  "connectivity" parameters derived from key material to control transformations.
                </p>
                <div className="text-sm text-text-tertiary">
                  Best for: Applications requiring complex key-dependent behavior.
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Performance */}
        <section className="py-16 bg-bg-secondary">
          <div className="max-w-4xl mx-auto px-6">
            <h2 className="text-2xl font-bold text-text-primary mb-8">Performance</h2>
            
            <div className="grid md:grid-cols-3 gap-6 mb-8">
              <div className="p-6 bg-bg-primary border border-border rounded-xl text-center">
                <div className="text-3xl font-bold text-accent-primary mb-2">1.4 GB/s</div>
                <div className="text-sm text-text-secondary">GPU Throughput</div>
              </div>
              <div className="p-6 bg-bg-primary border border-border rounded-xl text-center">
                <div className="text-3xl font-bold text-accent-primary mb-2">~50%</div>
                <div className="text-sm text-text-secondary">Avalanche Effect</div>
              </div>
              <div className="p-6 bg-bg-primary border border-border rounded-xl text-center">
                <div className="text-3xl font-bold text-accent-primary mb-2">~1 GB/s</div>
                <div className="text-sm text-text-secondary">CPU (rayon)</div>
              </div>
            </div>

            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-border">
                    <th className="py-3 px-4 text-left font-semibold text-text-primary">Mode</th>
                    <th className="py-3 px-4 text-left font-semibold text-text-primary">CPU</th>
                    <th className="py-3 px-4 text-left font-semibold text-text-primary">GPU</th>
                    <th className="py-3 px-4 text-left font-semibold text-text-primary">Avalanche</th>
                  </tr>
                </thead>
                <tbody className="text-text-secondary">
                  <tr className="border-b border-border/50">
                    <td className="py-3 px-4">Core</td>
                    <td className="py-3 px-4 font-mono">~1000 MB/s</td>
                    <td className="py-3 px-4 font-mono">-</td>
                    <td className="py-3 px-4 font-mono">52.1%</td>
                  </tr>
                  <tr className="border-b border-border/50">
                    <td className="py-3 px-4">Parametric</td>
                    <td className="py-3 px-4 font-mono">~650 MB/s</td>
                    <td className="py-3 px-4 font-mono text-accent-primary">~1.4 GB/s</td>
                    <td className="py-3 px-4 font-mono">50.0%</td>
                  </tr>
                  <tr className="border-b border-border/50">
                    <td className="py-3 px-4">Neural</td>
                    <td className="py-3 px-4 font-mono">~800 MB/s</td>
                    <td className="py-3 px-4 font-mono text-accent-primary">~1.3 GB/s</td>
                    <td className="py-3 px-4 font-mono">53.1%</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </section>

        {/* Quick Start */}
        <section className="py-16">
          <div className="max-w-4xl mx-auto px-6">
            <h2 className="text-2xl font-bold text-text-primary mb-8">Quick Start</h2>
            
            <div className="bg-bg-secondary border border-border rounded-xl overflow-hidden">
              <div className="px-4 py-2 border-b border-border bg-bg-primary">
                <span className="text-sm text-text-tertiary">Rust</span>
              </div>
              <pre className="p-4 overflow-x-auto text-sm text-text-secondary">{`use yaksha_cipher::{core, parametric, neural, aead};

let key = b"your_secret_key!";
let data = b"Sensitive information";

// Basic encryption (Core mode)
let encrypted = core::encrypt(data, key);
let decrypted = core::decrypt(&encrypted, key);

// Parametric mode (GPU-accelerated when available)
let encrypted = parametric::encrypt(data, key);
let decrypted = parametric::decrypt(&encrypted, key);

// Neural mode
let encrypted = neural::encrypt(data, key);
let decrypted = neural::decrypt(&encrypted, key);

// AEAD (authenticated encryption)
let mut cipher = aead::YakshaAead::new(key);
let (ciphertext, tag) = cipher.encrypt(data, b"associated_data");
let plaintext = cipher.decrypt(&ciphertext, &tag, b"associated_data")?;`}</pre>
            </div>
          </div>
        </section>

        {/* Security Note */}
        <section className="py-16 bg-bg-secondary">
          <div className="max-w-4xl mx-auto px-6">
            <div className="p-6 bg-amber-500/10 border border-amber-500/20 rounded-xl">
              <h3 className="font-semibold text-amber-500 mb-2">⚠️ Research Status</h3>
              <p className="text-sm text-text-secondary">
                Yaksha Cipher demonstrates strong cryptographic properties (avalanche effect, key sensitivity)
                and has undergone internal testing. However, it has not received formal third-party security
                audits. For production use requiring certified security, consider established standards like
                AES-GCM or ChaCha20-Poly1305.
              </p>
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
