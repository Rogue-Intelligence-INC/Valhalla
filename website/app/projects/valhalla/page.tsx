import Header from '@/components/Header'
import Footer from '@/components/Footer'
import Link from 'next/link'

export default function ValhallaProjectPage() {
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
              <span>Valhalla</span>
            </div>
            <div className="flex items-start gap-4 mb-6">
              <div className="w-16 h-16 rounded-lg bg-accent-primary/10 flex items-center justify-center text-accent-primary flex-shrink-0">
                <svg className="w-8 h-8" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <circle cx="12" cy="12" r="3" />
                  <path d="M12 2v4M12 18v4M2 12h4M18 12h4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" />
                </svg>
              </div>
              <div>
                <h1 className="text-4xl md:text-5xl font-serif font-bold text-ink-900">
                  Valhalla
                </h1>
                <p className="text-xl text-ink-600 mt-2">
                  Unified Algorithm Architecture
                </p>
              </div>
            </div>
            <div className="flex flex-wrap gap-2">
              <span className="bg-blue-50 text-blue-700 border border-blue-200 px-3 py-1 rounded text-sm font-medium">
                Research Phase
              </span>
              <span className="academic-tag">Cross-Disciplinary</span>
              <span className="academic-tag">High Originality</span>
              <span className="academic-tag">Fate + Quad</span>
            </div>
          </div>
        </section>

        {/* Content */}
        <section className="py-12 bg-paper-100">
          <div className="max-w-4xl mx-auto px-6">
            {/* Overview */}
            <div className="prose max-w-none mb-12">
              <h2>Executive Summary</h2>
              <p>
                Valhalla is a <strong>cross-disciplinary unified algorithm architecture</strong> that 
                combines control theory, machine learning, complex systems science, and mathematics. 
                It provides the theoretical foundation for Wave, Val-OS, and S-wind.
              </p>
              <p>
                The core innovation is the <strong>Fate coordination layer</strong> with its unified 
                mathematical formula, the <strong>Quad System</strong> for specialized data processing, 
                and the principle of "data processing as deformation signal."
              </p>
            </div>

            {/* Core Formula */}
            <div className="card mb-8 bg-paper-200 border-2 border-accent-primary/20">
              <h3 className="font-serif text-xl font-semibold text-ink-900 mb-4 text-center">
                Core Mathematical Formula
              </h3>
              <div className="math-block text-lg">
                weight = intensity × quality_score
              </div>
              <p className="text-sm text-ink-600 text-center mt-4">
                Output: 4-dimensional preference vector [BlackHole, MeshBrain, MultiNova, BaseForce]
              </p>
            </div>

            {/* Architecture */}
            <div className="mb-12">
              <h2 className="text-2xl font-serif font-bold text-ink-900 mb-6">System Architecture</h2>
              <div className="architecture-diagram text-xs">
{`┌────────────────────────────────────────────────────────────────────┐
│                      VALHALLA UNIFIED ALGORITHM                    │
│  ┌────────────────────────────────────────────────────────────┐   │
│  │              FATE UNIFIED COORDINATION LAYER               │   │
│  │      Core Formula: weight = intensity × quality_score      │   │
│  │      Output: 4D preference vector [BH, MB, MN, BF]         │   │
│  └────────────────────────────────────────────────────────────┘   │
│                              ↑ Deformation Signals                 │
│  ┌────────────────────────────────────────────────────────────┐   │
│  │           QUAD SYSTEM - DATA PROCESSING LAYER              │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐      │   │
│  │  │BlackHole │ │MeshBrain │ │MultiNova │ │BaseForce │      │   │
│  │  │ Extreme  │ │Cognitive │ │Large-Scale│ │ Fragment │      │   │
│  │  │   Data   │ │   Data   │ │   Data   │ │Optimization│    │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘      │   │
│  └────────────────────────────────────────────────────────────┘   │
│                              ↑                                     │
│  ┌────────────────────────────────────────────────────────────┐   │
│  │       VOYAGER DIMENSION LAYER + SPACE CONTAINER LAYER      │   │
│  │  Fractional Dimensions | Unknown Detection | Math Library  │   │
│  └────────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────────┘`}
              </div>
            </div>

            {/* Quad System Detail */}
            <div className="mb-12">
              <h2 className="text-2xl font-serif font-bold text-ink-900 mb-6">The Quad System</h2>
              <p className="text-ink-600 mb-6">
                Four specialized processors, each optimized for different data characteristics:
              </p>
              <div className="grid md:grid-cols-2 gap-6">
                <div className="card border-l-4 border-l-purple-500">
                  <h4 className="font-semibold text-ink-800 mb-2">BlackHole</h4>
                  <p className="text-sm text-ink-600 mb-2">
                    <strong>Extreme Data Processing</strong>
                  </p>
                  <p className="text-xs text-ink-500">
                    Handles edge cases, anomalies, and high-intensity signals. 
                    Like a black hole, it absorbs and processes data that would overwhelm other systems.
                  </p>
                </div>
                <div className="card border-l-4 border-l-blue-500">
                  <h4 className="font-semibold text-ink-800 mb-2">MeshBrain</h4>
                  <p className="text-sm text-ink-600 mb-2">
                    <strong>Cognitive Data Processing</strong>
                  </p>
                  <p className="text-xs text-ink-500">
                    Handles pattern recognition, relationship mapping, and structured data. 
                    Mimics neural network-like processing for complex cognitive tasks.
                  </p>
                </div>
                <div className="card border-l-4 border-l-green-500">
                  <h4 className="font-semibold text-ink-800 mb-2">MultiNova</h4>
                  <p className="text-sm text-ink-600 mb-2">
                    <strong>Large-Scale Data Processing</strong>
                  </p>
                  <p className="text-xs text-ink-500">
                    Handles high-volume, distributed data. Like multiple supernovas, 
                    it processes massive parallel streams efficiently.
                  </p>
                </div>
                <div className="card border-l-4 border-l-orange-500">
                  <h4 className="font-semibold text-ink-800 mb-2">BaseForce</h4>
                  <p className="text-sm text-ink-600 mb-2">
                    <strong>Fragment Optimization</strong>
                  </p>
                  <p className="text-xs text-ink-500">
                    Handles small, frequent operations. Optimizes at the fundamental level 
                    for maximum efficiency in granular tasks.
                  </p>
                </div>
              </div>
            </div>

            {/* Disciplinary Analysis */}
            <div className="mb-12">
              <h2 className="text-2xl font-serif font-bold text-ink-900 mb-6">Disciplinary Classification</h2>
              <p className="text-ink-600 mb-4">
                Valhalla is a cross-disciplinary algorithm spanning multiple fields:
              </p>
              <table className="w-full text-sm">
                <thead>
                  <tr>
                    <th className="text-left py-2">Discipline</th>
                    <th className="text-left py-2">Valhalla Features</th>
                    <th className="text-left py-2">Contribution</th>
                  </tr>
                </thead>
                <tbody>
                  <tr>
                    <td className="py-2">Cybernetics / Control Theory</td>
                    <td>Feedback loops, Fate coordination, preference adjustment</td>
                    <td className="font-semibold">35%</td>
                  </tr>
                  <tr>
                    <td className="py-2">Machine Learning / AI</td>
                    <td>Self-learning, adaptation, pattern recognition</td>
                    <td className="font-semibold">30%</td>
                  </tr>
                  <tr>
                    <td className="py-2">Complex Systems Science</td>
                    <td>Multi-agent coordination, emergent behavior, Quad division</td>
                    <td className="font-semibold">20%</td>
                  </tr>
                  <tr>
                    <td className="py-2">Mathematics / Fractals</td>
                    <td>Fractional dimensions, Hausdorff estimation, unified formula</td>
                    <td className="font-semibold">15%</td>
                  </tr>
                </tbody>
              </table>
            </div>

            {/* Uniqueness Analysis */}
            <div className="mb-12">
              <h2 className="text-2xl font-serif font-bold text-ink-900 mb-6">Originality Analysis</h2>
              <p className="text-ink-600 mb-4">
                Comparison with existing algorithms shows high originality:
              </p>
              <div className="space-y-4">
                <div className="card">
                  <div className="flex justify-between items-start">
                    <div>
                      <h4 className="font-semibold text-ink-800">vs. Ensemble Learning</h4>
                      <p className="text-sm text-ink-600">Similar: Multi-model coordination, weighted voting</p>
                    </div>
                    <span className="text-xs bg-green-100 text-green-700 px-2 py-1 rounded">Valhalla: Dynamic self-learning weights</span>
                  </div>
                </div>
                <div className="card">
                  <div className="flex justify-between items-start">
                    <div>
                      <h4 className="font-semibold text-ink-800">vs. Reinforcement Learning</h4>
                      <p className="text-sm text-ink-600">Similar: Feedback loops, learning optimization</p>
                    </div>
                    <span className="text-xs bg-green-100 text-green-700 px-2 py-1 rounded">Valhalla: No predefined reward function</span>
                  </div>
                </div>
                <div className="card">
                  <div className="flex justify-between items-start">
                    <div>
                      <h4 className="font-semibold text-ink-800">vs. Multi-Agent Systems</h4>
                      <p className="text-sm text-ink-600">Similar: Multi-system coordination</p>
                    </div>
                    <span className="text-xs bg-green-100 text-green-700 px-2 py-1 rounded">Valhalla: Unified math formula, not message passing</span>
                  </div>
                </div>
                <div className="card">
                  <div className="flex justify-between items-start">
                    <div>
                      <h4 className="font-semibold text-ink-800">vs. Attention Mechanisms</h4>
                      <p className="text-sm text-ink-600">Similar: Weighted computation</p>
                    </div>
                    <span className="text-xs bg-green-100 text-green-700 px-2 py-1 rounded">Valhalla: Weights from physics-law signals</span>
                  </div>
                </div>
              </div>
            </div>

            {/* Key Innovations */}
            <div className="mb-12">
              <h2 className="text-2xl font-serif font-bold text-ink-900 mb-6">Key Innovations</h2>
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr>
                      <th className="text-left py-2">Innovation</th>
                      <th className="text-left py-2">Description</th>
                      <th className="text-left py-2">Uniqueness</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr>
                      <td className="py-2 font-medium">Unified Mathematical Formula</td>
                      <td>weight = intensity × quality_score</td>
                      <td className="text-green-600">First, cross-system universal</td>
                    </tr>
                    <tr>
                      <td className="py-2 font-medium">Data Processing as Deformation</td>
                      <td>Processing itself generates signals</td>
                      <td className="text-green-600">Novel theory, zero overhead</td>
                    </tr>
                    <tr>
                      <td className="py-2 font-medium">Quad Specialized Processing</td>
                      <td>Four physics-law based systems</td>
                      <td className="text-green-600">Unique design, precise division</td>
                    </tr>
                    <tr>
                      <td className="py-2 font-medium">Fractional Dimension Adaptation</td>
                      <td>Supports non-integer dimensions (e.g., 2.3D)</td>
                      <td className="text-green-600">Innovative application</td>
                    </tr>
                    <tr>
                      <td className="py-2 font-medium">Observation-Only Coordination</td>
                      <td>Fate observes but doesn't control</td>
                      <td className="text-green-600">Philosophical breakthrough</td>
                    </tr>
                  </tbody>
                </table>
              </div>
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
