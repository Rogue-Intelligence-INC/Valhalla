import Header from '@/components/Header'
import Footer from '@/components/Footer'
import Link from 'next/link'

export default function CompressionPage() {
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
              <span>Adaptive Compression</span>
            </div>
            
            <div className="flex items-center gap-4 mb-4">
              <div className="w-12 h-12 rounded-xl bg-accent-primary/10 flex items-center justify-center">
                <svg className="w-6 h-6 text-accent-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M4 14h6m-6 4h6M4 6h16M4 10h16M14 14h6m-6 4h6" />
                </svg>
              </div>
              <div>
                <span className="px-2 py-1 text-xs font-medium bg-purple-500/10 text-purple-500 rounded">Experimental</span>
              </div>
            </div>
            
            <h1 className="text-4xl md:text-5xl font-bold text-text-primary mb-4">
              Adaptive Compression
            </h1>
            <p className="text-xl text-text-secondary mb-6">
              Intelligent multi-system compression with content-aware routing
            </p>
            
            <div className="flex flex-wrap gap-3">
              <a 
                href="https://github.com/Rogue-Intelligence-INC/Valhalla" 
                target="_blank"
                rel="noopener noreferrer"
                className="btn btn-primary"
              >
                <svg className="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                </svg>
                View Source
              </a>
            </div>
          </div>
        </section>

        {/* Overview */}
        <section className="py-16">
          <div className="max-w-4xl mx-auto px-6">
            <h2 className="text-2xl font-bold text-text-primary mb-6">Overview</h2>
            <p className="text-text-secondary mb-6 leading-relaxed">
              Adaptive Compression is an experimental multi-system compression framework that excels where 
              traditional algorithms fail. It routes data through four specialized compression systems 
              based on content analysis, achieving exceptional results on high-entropy data.
            </p>
            <p className="text-text-secondary mb-6 leading-relaxed">
              While traditional algorithms like gzip, bzip2, and zstd achieve 100% (no compression) on 
              random or high-entropy data, this system achieves 0.4-12.5% compression ratios on the same data.
            </p>
          </div>
        </section>

        {/* Performance Comparison */}
        <section className="py-16 bg-bg-secondary">
          <div className="max-w-4xl mx-auto px-6">
            <h2 className="text-2xl font-bold text-text-primary mb-8">Performance vs Traditional Algorithms</h2>
            
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-border">
                    <th className="py-3 px-4 text-left font-semibold text-text-primary">Data Type</th>
                    <th className="py-3 px-4 text-left font-semibold text-text-primary">Adaptive</th>
                    <th className="py-3 px-4 text-left font-semibold text-text-primary">gzip</th>
                    <th className="py-3 px-4 text-left font-semibold text-text-primary">zstd</th>
                    <th className="py-3 px-4 text-left font-semibold text-text-primary">Advantage</th>
                  </tr>
                </thead>
                <tbody className="text-text-secondary">
                  <tr className="border-b border-border/50">
                    <td className="py-3 px-4">Random Data</td>
                    <td className="py-3 px-4 font-mono text-accent-primary">0.4-6.3%</td>
                    <td className="py-3 px-4 font-mono">100%</td>
                    <td className="py-3 px-4 font-mono">100%</td>
                    <td className="py-3 px-4 text-green-500">15-250x better</td>
                  </tr>
                  <tr className="border-b border-border/50">
                    <td className="py-3 px-4">Compressed (MP4)</td>
                    <td className="py-3 px-4 font-mono text-accent-primary">12.5%</td>
                    <td className="py-3 px-4 font-mono">~99%</td>
                    <td className="py-3 px-4 font-mono">~99%</td>
                    <td className="py-3 px-4 text-green-500">8x better</td>
                  </tr>
                  <tr className="border-b border-border/50">
                    <td className="py-3 px-4">Audio (WAV)</td>
                    <td className="py-3 px-4 font-mono text-accent-primary">0.4%</td>
                    <td className="py-3 px-4 font-mono">~50%</td>
                    <td className="py-3 px-4 font-mono">2.5%</td>
                    <td className="py-3 px-4 text-green-500">6x better</td>
                  </tr>
                  <tr className="border-b border-border/50">
                    <td className="py-3 px-4">Text</td>
                    <td className="py-3 px-4 font-mono">1.6%</td>
                    <td className="py-3 px-4 font-mono">30%</td>
                    <td className="py-3 px-4 font-mono text-accent-primary">25%</td>
                    <td className="py-3 px-4 text-green-500">15x better</td>
                  </tr>
                  <tr className="border-b border-border/50">
                    <td className="py-3 px-4">Images (BMP)</td>
                    <td className="py-3 px-4 font-mono text-accent-primary">1.6%</td>
                    <td className="py-3 px-4 font-mono">10%</td>
                    <td className="py-3 px-4 font-mono">8.9%</td>
                    <td className="py-3 px-4 text-green-500">5x better</td>
                  </tr>
                  <tr className="border-b border-border/50">
                    <td className="py-3 px-4">JSON</td>
                    <td className="py-3 px-4 font-mono">3.1%</td>
                    <td className="py-3 px-4 font-mono text-accent-primary">1.7%</td>
                    <td className="py-3 px-4 font-mono text-accent-primary">1.5%</td>
                    <td className="py-3 px-4 text-amber-500">Traditional wins</td>
                  </tr>
                  <tr className="border-b border-border/50">
                    <td className="py-3 px-4">Repeated Data</td>
                    <td className="py-3 px-4 font-mono">25%</td>
                    <td className="py-3 px-4 font-mono text-accent-primary">0.1%</td>
                    <td className="py-3 px-4 font-mono text-accent-primary">0.0%</td>
                    <td className="py-3 px-4 text-amber-500">Traditional wins</td>
                  </tr>
                </tbody>
              </table>
            </div>
            
            <p className="text-sm text-text-tertiary mt-4">
              Lower percentage = better compression. The system excels on high-entropy data but traditional 
              algorithms remain superior for structured/repetitive data.
            </p>
          </div>
        </section>

        {/* Key Metrics */}
        <section className="py-16">
          <div className="max-w-4xl mx-auto px-6">
            <h2 className="text-2xl font-bold text-text-primary mb-8">Key Metrics</h2>
            
            <div className="grid md:grid-cols-3 gap-6">
              <div className="p-6 bg-bg-secondary border border-border rounded-xl text-center">
                <div className="text-3xl font-bold text-accent-primary mb-2">0.4%</div>
                <div className="text-sm text-text-secondary">Best Compression Ratio</div>
                <div className="text-xs text-text-tertiary mt-1">on WAV audio</div>
              </div>
              <div className="p-6 bg-bg-secondary border border-border rounded-xl text-center">
                <div className="text-3xl font-bold text-accent-primary mb-2">4</div>
                <div className="text-sm text-text-secondary">Compression Systems</div>
                <div className="text-xs text-text-tertiary mt-1">with intelligent routing</div>
              </div>
              <div className="p-6 bg-bg-secondary border border-border rounded-xl text-center">
                <div className="text-3xl font-bold text-accent-primary mb-2">20</div>
                <div className="text-sm text-text-secondary">Max Cycles</div>
                <div className="text-xs text-text-tertiary mt-1">with automatic stopping</div>
              </div>
            </div>
          </div>
        </section>

        {/* Features */}
        <section className="py-16 bg-bg-secondary">
          <div className="max-w-4xl mx-auto px-6">
            <h2 className="text-2xl font-bold text-text-primary mb-8">Features</h2>
            
            <div className="grid md:grid-cols-2 gap-6">
              <div className="p-6 bg-bg-primary border border-border rounded-xl">
                <h3 className="font-semibold text-text-primary mb-2">🎯 Intelligent Routing</h3>
                <p className="text-sm text-text-secondary">
                  Analyzes data entropy, patterns, structure, and density to route to the optimal compression system.
                </p>
              </div>
              <div className="p-6 bg-bg-primary border border-border rounded-xl">
                <h3 className="font-semibold text-text-primary mb-2">🔄 Multi-Cycle Processing</h3>
                <p className="text-sm text-text-secondary">
                  Iterative compression with automatic stopping when marginal gains drop below 1%.
                </p>
              </div>
              <div className="p-6 bg-bg-primary border border-border rounded-xl">
                <h3 className="font-semibold text-text-primary mb-2">📊 Four Specialized Systems</h3>
                <p className="text-sm text-text-secondary">
                  Each system optimized for different data characteristics: folding, differential, bit-plane, and baseline.
                </p>
              </div>
              <div className="p-6 bg-bg-primary border border-border rounded-xl">
                <h3 className="font-semibold text-text-primary mb-2">🎲 High-Entropy Specialist</h3>
                <p className="text-sm text-text-secondary">
                  Excels on random and already-compressed data where traditional algorithms produce no compression.
                </p>
              </div>
            </div>
          </div>
        </section>

        {/* Use Cases */}
        <section className="py-16">
          <div className="max-w-4xl mx-auto px-6">
            <h2 className="text-2xl font-bold text-text-primary mb-8">Best Use Cases</h2>
            
            <div className="space-y-4">
              <div className="p-4 bg-green-500/10 border border-green-500/20 rounded-xl">
                <h3 className="font-semibold text-green-500 mb-1">✓ High-Entropy Data</h3>
                <p className="text-sm text-text-secondary">Random data, encrypted files, high-frequency signals</p>
              </div>
              <div className="p-4 bg-green-500/10 border border-green-500/20 rounded-xl">
                <h3 className="font-semibold text-green-500 mb-1">✓ Already-Compressed Files</h3>
                <p className="text-sm text-text-secondary">MP4, MP3, JPEG where traditional compression fails</p>
              </div>
              <div className="p-4 bg-green-500/10 border border-green-500/20 rounded-xl">
                <h3 className="font-semibold text-green-500 mb-1">✓ Raw Sensor Data</h3>
                <p className="text-sm text-text-secondary">Audio, scientific instruments, IoT streams</p>
              </div>
              <div className="p-4 bg-amber-500/10 border border-amber-500/20 rounded-xl">
                <h3 className="font-semibold text-amber-500 mb-1">△ Structured Text/JSON</h3>
                <p className="text-sm text-text-secondary">Works but traditional algorithms may be better</p>
              </div>
              <div className="p-4 bg-red-500/10 border border-red-500/20 rounded-xl">
                <h3 className="font-semibold text-red-500 mb-1">✗ Highly Repetitive Data</h3>
                <p className="text-sm text-text-secondary">Use traditional RLE/LZ77-based algorithms instead</p>
              </div>
            </div>
          </div>
        </section>

        {/* Research Note */}
        <section className="py-16 bg-bg-secondary">
          <div className="max-w-4xl mx-auto px-6">
            <div className="p-6 bg-purple-500/10 border border-purple-500/20 rounded-xl">
              <h3 className="font-semibold text-purple-500 mb-2">🔬 Experimental Status</h3>
              <p className="text-sm text-text-secondary">
                This compression system is experimental research exploring alternative approaches to data reduction. 
                It uses lossy techniques optimized for specific data types and may not preserve 100% fidelity. 
                For archival or lossless requirements, use established algorithms like gzip, zstd, or lz4.
              </p>
            </div>
          </div>
        </section>
      </main>

      <Footer />
    </>
  )
}
