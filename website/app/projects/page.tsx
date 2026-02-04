import Header from '@/components/Header'
import Footer from '@/components/Footer'
import ProjectCard from '@/components/ProjectCard'

// Icons
const WaveIcon = () => (
  <svg className="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M2 12c0-3 2.5-6 6-6s4 3 4 6 2.5 6 6 6 6-3 6-6" />
  </svg>
)

const OSIcon = () => (
  <svg className="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <rect x="2" y="3" width="20" height="14" rx="2" />
    <path d="M8 21h8M12 17v4" />
  </svg>
)

const MathIcon = () => (
  <svg className="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4zM17 14v6M14 17h6" />
  </svg>
)

const LockIcon = () => (
  <svg className="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
    <path d="M7 11V7a5 5 0 0 1 10 0v4" />
  </svg>
)

const CompressIcon = () => (
  <svg className="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M4 14h6m-6 4h6M4 6h16M4 10h16M14 14h6m-6 4h6" />
  </svg>
)

const ArchIcon = () => (
  <svg className="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <circle cx="12" cy="12" r="3" />
    <path d="M12 2v4M12 18v4M2 12h4M18 12h4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" />
  </svg>
)

export default function ProjectsPage() {
  const projects = [
    {
      title: 'S-wind',
      subtitle: 'Autonomous Discovery Engine',
      description: 'Pattern recognition and mathematical formula discovery from raw numerical data. Input any sequence, get algebraic formulas with complete derivations. Uses exact rational arithmetic for 99.97% precision with zero floating-point errors.',
      features: ['Pattern Discovery', 'Exact Arithmetic', 'Auto LaTeX Generation', '10,000+ ops/sec'],
      status: 'active' as const,
      href: '/projects/s-wind',
      icon: <MathIcon />,
    },
    {
      title: 'Wave',
      subtitle: 'Adaptive Programming Language',
      description: 'Self-hosting programming language with runtime optimization. Compiles directly to native machine code in under 3ms. Achieves 19x faster execution than Python with 30x lower memory usage. Zero external dependencies.',
      features: ['Self-Hosting Compiler', 'Runtime Optimization', '19x Python Speed', '<3ms Compile'],
      status: 'active' as const,
      href: '/projects/wave',
      icon: <WaveIcon />,
    },
    {
      title: 'Val-OS',
      subtitle: 'Intelligent Operating System',
      description: 'Self-learning operating system that adapts to your hardware. Features hybrid CLI/GUI interface, four-pool specialized memory management, and AI-powered file organization. No hardcoded values — everything is runtime-determined.',
      features: ['Self-Learning Boot', 'Hybrid Interface', 'AI File Manager', 'Adaptive Memory'],
      status: 'research' as const,
      href: '/projects/val-os',
      icon: <OSIcon />,
    },
    {
      title: 'Yaksha Cipher',
      subtitle: 'Symmetric Encryption Suite',
      description: 'High-performance symmetric encryption with GPU acceleration. Achieves 1.4 GB/s throughput on modern GPUs. Features ~50% avalanche effect (near-optimal), stateful authenticated encryption, and multiple transformation modes.',
      features: ['GPU Accelerated', '1.4 GB/s Throughput', 'Stateful AEAD', '~50% Avalanche'],
      status: 'active' as const,
      href: '/projects/yaksha-cipher',
      icon: <LockIcon />,
    },
    {
      title: 'Adaptive Compression',
      subtitle: 'Intelligent Data Compression',
      description: 'Multi-system compression with intelligent routing. Excels on high-entropy data where traditional algorithms fail — achieving 0.4-12.5% compression ratio vs 100% for gzip/zstd. Four specialized compression systems with automatic selection.',
      features: ['Intelligent Routing', 'High-Entropy Specialist', 'Multi-System', '0.4% Best Ratio'],
      status: 'experimental' as const,
      href: '/projects/compression',
      icon: <CompressIcon />,
    },
    {
      title: 'Valhalla Core',
      subtitle: 'Algorithm Foundation',
      description: 'The unified architecture powering all Valhalla projects. Provides adaptive resource allocation, cross-domain optimization, and intelligent system coordination. The foundation for building adaptive, self-improving systems.',
      features: ['Unified Architecture', 'Adaptive Allocation', 'Cross-Domain', 'Self-Improving'],
      status: 'research' as const,
      href: '/projects/valhalla',
      icon: <ArchIcon />,
    },
  ]

  const researchAreas = [
    {
      icon: '🔬',
      title: 'Autonomous Discovery',
      desc: 'Systems that find patterns and generate formulas without human guidance or prior assumptions.',
    },
    {
      icon: '⚙️',
      title: 'Adaptive Compilation',
      desc: 'Compilers that optimize based on runtime behavior and hardware characteristics.',
    },
    {
      icon: '🖥️',
      title: 'Intelligent Systems',
      desc: 'Operating systems and runtimes that learn and adapt instead of following fixed rules.',
    },
    {
      icon: '🔐',
      title: 'Applied Cryptography',
      desc: 'High-performance encryption with GPU acceleration and modern authenticated modes.',
    },
    {
      icon: '📦',
      title: 'Data Compression',
      desc: 'Intelligent compression that routes data to specialized systems based on content analysis.',
    },
    {
      icon: '🧮',
      title: 'Exact Computation',
      desc: 'Rational arithmetic and symbolic computation for scientific precision.',
    },
  ]

  return (
    <>
      <Header />
      
      <main className="pt-20">
        {/* Hero */}
        <section className="bg-gradient-to-b from-bg-secondary to-bg-primary py-16">
          <div className="max-w-6xl mx-auto px-6">
            <h1 className="text-4xl md:text-5xl font-bold text-text-primary mb-4">
              Research Projects
            </h1>
            <p className="text-xl text-text-secondary max-w-3xl">
              From autonomous mathematical discovery to intelligent operating systems — 
              a complete ecosystem of adaptive AI research and development.
            </p>
          </div>
        </section>

        {/* Projects Grid */}
        <section className="py-16 bg-bg-primary">
          <div className="max-w-6xl mx-auto px-6">
            <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
              {projects.map((project) => (
                <ProjectCard key={project.title} {...project} />
              ))}
            </div>
          </div>
        </section>

        {/* Research Areas */}
        <section className="py-16 bg-bg-secondary">
          <div className="max-w-6xl mx-auto px-6">
            <h2 className="text-3xl font-bold text-text-primary mb-8 text-center">
              Research Areas
            </h2>
            <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
              {researchAreas.map((area, i) => (
                <div key={i} className="p-6 bg-bg-primary border border-border rounded-xl">
                  <div className="text-3xl mb-3">{area.icon}</div>
                  <h3 className="font-semibold text-text-primary mb-2">{area.title}</h3>
                  <p className="text-sm text-text-secondary">{area.desc}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* Performance Highlights */}
        <section className="py-16 bg-bg-primary">
          <div className="max-w-6xl mx-auto px-6">
            <h2 className="text-3xl font-bold text-text-primary mb-8 text-center">
              Performance Highlights
            </h2>
            <div className="overflow-x-auto">
              <table className="w-full text-left">
                <thead>
                  <tr className="border-b border-border">
                    <th className="py-4 px-4 font-semibold text-text-primary">Project</th>
                    <th className="py-4 px-4 font-semibold text-text-primary">Key Metric</th>
                    <th className="py-4 px-4 font-semibold text-text-primary">Performance</th>
                    <th className="py-4 px-4 font-semibold text-text-primary">Comparison</th>
                  </tr>
                </thead>
                <tbody className="text-text-secondary">
                  <tr className="border-b border-border/50">
                    <td className="py-4 px-4 font-medium text-text-primary">S-wind</td>
                    <td className="py-4 px-4">Processing Speed</td>
                    <td className="py-4 px-4 font-mono text-accent-primary">10,000+ ops/sec</td>
                    <td className="py-4 px-4">2.1μs latency</td>
                  </tr>
                  <tr className="border-b border-border/50">
                    <td className="py-4 px-4 font-medium text-text-primary">Wave</td>
                    <td className="py-4 px-4">Execution Speed</td>
                    <td className="py-4 px-4 font-mono text-accent-primary">19x faster</td>
                    <td className="py-4 px-4">vs Python baseline</td>
                  </tr>
                  <tr className="border-b border-border/50">
                    <td className="py-4 px-4 font-medium text-text-primary">Wave</td>
                    <td className="py-4 px-4">Compile Time</td>
                    <td className="py-4 px-4 font-mono text-accent-primary">&lt;3ms</td>
                    <td className="py-4 px-4">~35KB compiler</td>
                  </tr>
                  <tr className="border-b border-border/50">
                    <td className="py-4 px-4 font-medium text-text-primary">Yaksha Cipher</td>
                    <td className="py-4 px-4">Encryption Throughput</td>
                    <td className="py-4 px-4 font-mono text-accent-primary">1.4 GB/s</td>
                    <td className="py-4 px-4">GPU accelerated</td>
                  </tr>
                  <tr className="border-b border-border/50">
                    <td className="py-4 px-4 font-medium text-text-primary">Yaksha Cipher</td>
                    <td className="py-4 px-4">Avalanche Effect</td>
                    <td className="py-4 px-4 font-mono text-accent-primary">~50%</td>
                    <td className="py-4 px-4">Near-optimal</td>
                  </tr>
                  <tr className="border-b border-border/50">
                    <td className="py-4 px-4 font-medium text-text-primary">Compression</td>
                    <td className="py-4 px-4">High-Entropy Ratio</td>
                    <td className="py-4 px-4 font-mono text-accent-primary">0.4-6.3%</td>
                    <td className="py-4 px-4">vs 100% traditional</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </section>
      </main>

      <Footer />
    </>
  )
}
