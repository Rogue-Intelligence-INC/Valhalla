import Header from '@/components/Header'
import Footer from '@/components/Footer'
import Link from 'next/link'

export default function ValOSProjectPage() {
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
              <span>Val-OS</span>
            </div>
            <div className="flex items-start gap-4 mb-6">
              <div className="w-16 h-16 rounded-lg bg-accent-primary/10 flex items-center justify-center text-accent-primary flex-shrink-0">
                <svg className="w-8 h-8" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <rect x="2" y="3" width="20" height="14" rx="2" />
                  <path d="M8 21h8M12 17v4" />
                </svg>
              </div>
              <div>
                <h1 className="text-4xl md:text-5xl font-serif font-bold text-ink-900">
                  Val-OS
                </h1>
                <p className="text-xl text-ink-600 mt-2">
                  Rethinking Operating System Design
                </p>
              </div>
            </div>
            <div className="flex flex-wrap gap-2">
              <span className="bg-blue-50 text-blue-700 border border-blue-200 px-3 py-1 rounded text-sm font-medium">
                Research Phase
              </span>
              <span className="academic-tag">v2.3</span>
              <span className="academic-tag">Hybrid UI</span>
              <span className="academic-tag">Self-Learning</span>
            </div>
          </div>
        </section>

        {/* Content */}
        <section className="py-12 bg-paper-100">
          <div className="max-w-4xl mx-auto px-6">
            {/* Overview */}
            <div className="prose max-w-none mb-12">
              <h2>The Problem with Traditional Operating Systems</h2>
              <p>
                Every operating system you use today—Windows, Linux, macOS—was built on the same assumptions:
              </p>
              <ol>
                <li><strong>Hardware is predictable</strong> → So we hardcode addresses and configurations</li>
                <li><strong>Memory usage is uniform</strong> → So we use one-size-fits-all allocators</li>
                <li><strong>Users need either simplicity OR power</strong> → So we separate GUI and CLI</li>
              </ol>
              <p>
                These assumptions made sense in 1970. They don't anymore. Val-OS is an experiment asking:
                <em> If we designed an operating system today, without historical baggage, what would it look like?</em>
              </p>
            </div>

            {/* Architecture */}
            <div className="card mb-8">
              <h3 className="font-serif text-xl font-semibold text-ink-900 mb-4">System Architecture</h3>
              <div className="architecture-diagram text-xs">
{`┌─────────────────────────────────────────────────────┐
│                 Val-OS v2.3                         │
├─────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────┐  │
│  │               Desktop Environment              │  │
│  │  Window Management · Taskbar · System Panel   │  │
│  └───────────────────────────────────────────────┘  │
│                        ↓                            │
│  ┌──────────┬──────────┬──────────┬──────────┐     │
│  │ Notebook │  Music   │  Video   │Minesweeper│    │
│  ├──────────┼──────────┼──────────┼──────────┤     │
│  │ Browser  │ Settings │ App Store│User Panel│     │
│  └──────────┴──────────┴──────────┴──────────┘     │
│                        ↓                            │
│  ┌───────────────────────────────────────────────┐  │
│  │              Fate Control Layer               │  │
│  │  Resource Detection · Pool Decisions · Collapse│  │
│  └───────────────────────────────────────────────┘  │
│                        ↓                            │
│  ┌──────────┬──────────┬──────────┬──────────┐     │
│  │BlackHole │MeshBrain │MultiNova │BaseForce │     │
│  └──────────┴──────────┴──────────┴──────────┘     │
└─────────────────────────────────────────────────────┘`}
              </div>
            </div>

            {/* Key Innovations */}
            <div className="mb-12">
              <h2 className="text-2xl font-serif font-bold text-ink-900 mb-6">Key Innovations</h2>
              
              <div className="space-y-6">
                {/* Hybrid Interface */}
                <div className="card">
                  <h4 className="font-semibold text-ink-800 mb-3">1. Hybrid Interface: Beyond GUI vs CLI</h4>
                  <div className="grid md:grid-cols-2 gap-4 mb-4">
                    <div>
                      <p className="text-sm text-ink-600 mb-2"><strong>Traditional CLI:</strong></p>
                      <pre className="text-xs">{`$ ls -la
total 48
drwxr-xr-x 12 user...
-rw-r--r--  1 user...`}</pre>
                    </div>
                    <div>
                      <p className="text-sm text-ink-600 mb-2"><strong>Val-OS Hybrid:</strong></p>
                      <pre className="text-xs">{`┌─────────────────────────┐
│ 📁 Current: /home/user  │
├─────────────────────────┤
│ 📄 file.txt    123 B    │
│ 📂 documents/           │
└─────────────────────────┘
  [Open] [Delete] ← Click!`}</pre>
                    </div>
                  </div>
                  <p className="text-sm text-ink-600">
                    Full 24-bit color, mouse support, works over SSH, zero latency—the best of both worlds.
                  </p>
                </div>

                {/* Self-Learning Bootloader */}
                <div className="card">
                  <h4 className="font-semibold text-ink-800 mb-3">2. Self-Learning Bootloader</h4>
                  <p className="text-sm text-ink-600 mb-3">
                    Traditional systems crash on hardware changes. Val-OS learns and adapts:
                  </p>
                  <pre className="text-xs mb-3">{`First boot:
1. Detect all hardware → Learn what exists
2. Analyze memory layout → Decide best addresses
3. Try initializing devices → Remember what works
4. Save knowledge → Use it next time

Nth boot:
Almost instant—the system already "knows" everything`}</pre>
                  <p className="text-sm text-ink-600">
                    No manual configuration, automatic recovery, faster boot times over time.
                  </p>
                </div>

                {/* Four Memory Pools */}
                <div className="card">
                  <h4 className="font-semibold text-ink-800 mb-3">3. Four-Pool Memory Management</h4>
                  <p className="text-sm text-ink-600 mb-3">
                    Different data types have different patterns. We use specialized pools:
                  </p>
                  <table className="w-full text-sm mb-3">
                    <thead>
                      <tr className="text-left">
                        <th className="py-1">Pool</th>
                        <th className="py-1">Data Type</th>
                        <th className="py-1">Strategy</th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr><td className="py-1">Pool 1</td><td>Code (large, long-lived)</td><td>Dense packing</td></tr>
                      <tr><td className="py-1">Pool 2</td><td>Strings (frequent create/destroy)</td><td>Indexed lookup</td></tr>
                      <tr><td className="py-1">Pool 3</td><td>Buffers (parallel access)</td><td>Partitioned</td></tr>
                      <tr><td className="py-1">Pool 4</td><td>Temp vars (short-lived)</td><td>Fast recycle</td></tr>
                    </tbody>
                  </table>
                  <p className="text-sm text-ink-600">
                    Automatic routing based on allocation patterns. Proactive Collapse optimization prevents fragmentation.
                  </p>
                </div>

                {/* Zero Hardcoding */}
                <div className="card">
                  <h4 className="font-semibold text-ink-800 mb-3">4. Zero Hardcoded Values</h4>
                  <p className="text-sm text-ink-600 mb-3">
                    No magic numbers. Everything is decided at runtime:
                  </p>
                  <ul className="text-sm text-ink-600 list-disc list-inside">
                    <li>Memory addresses: Based on available RAM</li>
                    <li>Pool sizes: Adjusted by actual usage patterns</li>
                    <li>Thresholds: Self-tuned from observed performance</li>
                  </ul>
                </div>
              </div>
            </div>

            {/* Comparison Table */}
            <div className="mb-12">
              <h2 className="text-2xl font-serif font-bold text-ink-900 mb-6">Technical Comparison</h2>
              <div className="overflow-x-auto">
                <table>
                  <thead>
                    <tr>
                      <th>Aspect</th>
                      <th>Traditional OS</th>
                      <th>Val-OS</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr><td>Interface</td><td>CLI OR GUI</td><td>Hybrid (both)</td></tr>
                    <tr><td>Boot</td><td>Fixed config</td><td>Self-learning</td></tr>
                    <tr><td>Memory</td><td>One allocator</td><td>Four specialized pools</td></tr>
                    <tr><td>Config</td><td>Hardcoded values</td><td>Runtime decisions</td></tr>
                    <tr><td>Errors</td><td>React after crash</td><td>Prevent before crash</td></tr>
                    <tr><td>Adaptation</td><td>Manual reconfiguration</td><td>Automatic</td></tr>
                  </tbody>
                </table>
              </div>
            </div>

            {/* Built-in Apps */}
            <div className="mb-12">
              <h2 className="text-2xl font-serif font-bold text-ink-900 mb-6">Built-in Applications</h2>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div className="card text-center">
                  <span className="text-2xl">📝</span>
                  <p className="text-sm font-medium mt-2">Notebook</p>
                </div>
                <div className="card text-center">
                  <span className="text-2xl">🎵</span>
                  <p className="text-sm font-medium mt-2">Music Player</p>
                </div>
                <div className="card text-center">
                  <span className="text-2xl">🎬</span>
                  <p className="text-sm font-medium mt-2">Video Player</p>
                </div>
                <div className="card text-center">
                  <span className="text-2xl">💣</span>
                  <p className="text-sm font-medium mt-2">Minesweeper</p>
                </div>
                <div className="card text-center">
                  <span className="text-2xl">🌐</span>
                  <p className="text-sm font-medium mt-2">Browser</p>
                </div>
                <div className="card text-center">
                  <span className="text-2xl">⚙️</span>
                  <p className="text-sm font-medium mt-2">Settings</p>
                </div>
                <div className="card text-center">
                  <span className="text-2xl">📦</span>
                  <p className="text-sm font-medium mt-2">App Store</p>
                </div>
                <div className="card text-center">
                  <span className="text-2xl">👤</span>
                  <p className="text-sm font-medium mt-2">User Panel</p>
                </div>
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
