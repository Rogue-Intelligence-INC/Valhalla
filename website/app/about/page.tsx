import Header from '@/components/Header'
import Footer from '@/components/Footer'

export default function AboutPage() {
  return (
    <>
      <Header />
      
      <main className="pt-20">
        {/* Hero */}
        <section className="bg-gradient-to-b from-paper-200 to-paper-100 py-16">
          <div className="max-w-4xl mx-auto px-6">
            <h1 className="text-4xl md:text-5xl font-serif font-bold text-ink-900 mb-4">
              About Us
            </h1>
            <p className="text-xl text-ink-600">
              Pioneering the future of adaptive computing.
            </p>
          </div>
        </section>

        {/* Content */}
        <section className="py-12 bg-paper-100">
          <div className="max-w-4xl mx-auto px-6">
            {/* Mission */}
            <div className="prose max-w-none mb-12">
              <h2>Our Mission</h2>
              <p>
                <strong>Rogue Intelligence LNC</strong> is a research organization dedicated to advancing 
                the frontiers of programming language design, operating system architecture, and 
                mathematical computing.
              </p>
              <p>
                We believe that the fundamental assumptions underlying modern computing—established 
                decades ago—deserve to be questioned. Our research explores what becomes possible 
                when we design systems that learn, adapt, and evolve.
              </p>
            </div>

            {/* Philosophy */}
            <div className="card mb-12 bg-paper-200 border-l-4 border-l-accent-primary">
              <h3 className="font-serif text-xl font-semibold text-ink-900 mb-4">
                Our Technical Philosophy
              </h3>
              <div className="space-y-4">
                <div>
                  <h4 className="font-semibold text-ink-800">Learn, Don't Assume</h4>
                  <p className="text-sm text-ink-600">
                    Systems should discover optimal configurations through observation, not predetermined rules.
                  </p>
                </div>
                <div>
                  <h4 className="font-semibold text-ink-800">Both, Not Either</h4>
                  <p className="text-sm text-ink-600">
                    Refuse false dichotomies. Combine power with simplicity, performance with precision.
                  </p>
                </div>
                <div>
                  <h4 className="font-semibold text-ink-800">Prevent, Don't React</h4>
                  <p className="text-sm text-ink-600">
                    Design systems that fix problems before they cause failures through proactive optimization.
                  </p>
                </div>
                <div>
                  <h4 className="font-semibold text-ink-800">Adapt, Don't Hardcode</h4>
                  <p className="text-sm text-ink-600">
                    No magic numbers. Everything should be dynamic, determined by the actual environment.
                  </p>
                </div>
              </div>
            </div>

            {/* Research Areas */}
            <div className="mb-12">
              <h2 className="text-2xl font-serif font-bold text-ink-900 mb-6">Research Areas</h2>
              <div className="grid md:grid-cols-2 gap-6">
                <div className="card">
                  <h4 className="font-semibold text-ink-800 mb-2">Programming Languages</h4>
                  <p className="text-sm text-ink-600">
                    Self-hosting compilers, architecture-aware optimization, and the Wave language project.
                  </p>
                </div>
                <div className="card">
                  <h4 className="font-semibold text-ink-800 mb-2">Operating Systems</h4>
                  <p className="text-sm text-ink-600">
                    Self-learning bootloaders, adaptive memory management, and the Val-OS project.
                  </p>
                </div>
                <div className="card">
                  <h4 className="font-semibold text-ink-800 mb-2">Mathematical Computing</h4>
                  <p className="text-sm text-ink-600">
                    Exact rational arithmetic, symbolic computation, and the S-wind framework.
                  </p>
                </div>
                <div className="card">
                  <h4 className="font-semibold text-ink-800 mb-2">Algorithm Architecture</h4>
                  <p className="text-sm text-ink-600">
                    Cross-disciplinary system design, the Valhalla unified architecture, and Fate coordination.
                  </p>
                </div>
              </div>
            </div>

            {/* Achievements */}
            <div className="mb-12">
              <h2 className="text-2xl font-serif font-bold text-ink-900 mb-6">Key Achievements</h2>
              <div className="space-y-4">
                <div className="flex items-start gap-4">
                  <div className="w-8 h-8 rounded-full bg-green-100 flex items-center justify-center flex-shrink-0">
                    <span className="text-green-600">✓</span>
                  </div>
                  <div>
                    <h4 className="font-semibold text-ink-800">Self-Hosting Wave Compiler</h4>
                    <p className="text-sm text-ink-600">
                      Complete technical independence with 2.8× performance over Python.
                    </p>
                  </div>
                </div>
                <div className="flex items-start gap-4">
                  <div className="w-8 h-8 rounded-full bg-green-100 flex items-center justify-center flex-shrink-0">
                    <span className="text-green-600">✓</span>
                  </div>
                  <div>
                    <h4 className="font-semibold text-ink-800">Val-OS Desktop Environment</h4>
                    <p className="text-sm text-ink-600">
                      Full desktop with hybrid CLI/GUI interface, running on minimal resources.
                    </p>
                  </div>
                </div>
                <div className="flex items-start gap-4">
                  <div className="w-8 h-8 rounded-full bg-green-100 flex items-center justify-center flex-shrink-0">
                    <span className="text-green-600">✓</span>
                  </div>
                  <div>
                    <h4 className="font-semibold text-ink-800">Exact Algebraic Framework</h4>
                    <p className="text-sm text-ink-600">
                      Zero-error mathematical computing with verified physical constants and theorems.
                    </p>
                  </div>
                </div>
                <div className="flex items-start gap-4">
                  <div className="w-8 h-8 rounded-full bg-green-100 flex items-center justify-center flex-shrink-0">
                    <span className="text-green-600">✓</span>
                  </div>
                  <div>
                    <h4 className="font-semibold text-ink-800">Valhalla Architecture</h4>
                    <p className="text-sm text-ink-600">
                      Original cross-disciplinary algorithm with high uniqueness rating.
                    </p>
                  </div>
                </div>
              </div>
            </div>

            {/* Contact */}
            <div className="card bg-accent-primary text-white">
              <h3 className="font-serif text-xl font-semibold mb-4">Get in Touch</h3>
              <p className="text-paper-300 mb-6">
                We welcome collaboration with researchers, institutions, and industry partners 
                who share our vision for advancing computational science.
              </p>
              <div className="space-y-2 text-sm">
                <p>
                  <strong>Email:</strong>{' '}
                  <a href="mailto:contact@rogue-intelligence.com" className="text-paper-200 hover:text-white border-b-0">
                    contact@rogue-intelligence.com
                  </a>
                </p>
                <p>
                  <strong>GitHub:</strong>{' '}
                  <a href="https://github.com/rogue-intelligence" className="text-paper-200 hover:text-white border-b-0" target="_blank" rel="noopener noreferrer">
                    github.com/rogue-intelligence
                  </a>
                </p>
              </div>
            </div>
          </div>
        </section>
      </main>

      <Footer />
    </>
  )
}
