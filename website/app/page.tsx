'use client'

import { useState, useEffect } from 'react'
import Header from '@/components/Header'
import Footer from '@/components/Footer'
import ProjectCard from '@/components/ProjectCard'
import Link from 'next/link'
import { loadSiteContent, SiteContent, defaultSiteContent } from '@/lib/content'

// Icons
const WaveIcon = () => (
  <svg className="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}>
    <path d="M2 12c0-3 2.5-6 6-6s4 3 4 6 2.5 6 6 6 6-3 6-6" />
  </svg>
)

const OSIcon = () => (
  <svg className="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}>
    <rect x="2" y="3" width="20" height="14" rx="2" />
    <path d="M8 21h8M12 17v4" />
  </svg>
)

const MathIcon = () => (
  <svg className="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}>
    <path d="M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4zM17 14v6M14 17h6" />
  </svg>
)

const LockIcon = () => (
  <svg className="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}>
    <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
    <path d="M7 11V7a5 5 0 0 1 10 0v4" />
  </svg>
)

const CompressIcon = () => (
  <svg className="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}>
    <path d="M4 14h6m-6 4h6M4 6h16M4 10h16M14 14h6m-6 4h6" />
  </svg>
)

const ArchIcon = () => (
  <svg className="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}>
    <circle cx="12" cy="12" r="3" />
    <path d="M12 2v4M12 18v4M2 12h4M18 12h4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" />
  </svg>
)

// Feature Icons
const RocketIcon = () => (
  <svg className="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}>
    <path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z" />
    <path d="m12 15-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z" />
    <path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0" />
    <path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5" />
  </svg>
)

const BoltIcon = () => (
  <svg className="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}>
    <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
  </svg>
)

const TargetIcon = () => (
  <svg className="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}>
    <circle cx="12" cy="12" r="10" />
    <circle cx="12" cy="12" r="6" />
    <circle cx="12" cy="12" r="2" />
  </svg>
)

export default function HomePage() {
  const [content, setContent] = useState<SiteContent>(defaultSiteContent)

  useEffect(() => {
    setContent(loadSiteContent())
  }, [])

  const highlights = [
    { icon: <RocketIcon />, title: 'AI-Powered', desc: 'Adaptive intelligence' },
    { icon: <BoltIcon />, title: 'High Performance', desc: 'GPU accelerated' },
    { icon: <TargetIcon />, title: 'Production Ready', desc: 'Battle-tested' },
  ]

  const projects = [
    {
      title: 'S-wind',
      subtitle: 'Autonomous Discovery Engine',
      description: 'Pattern recognition and formula discovery from raw data. Processes 10,000+ ops/sec with 99.97% precision using exact arithmetic.',
      features: ['Pattern Discovery', 'Exact Arithmetic', 'Auto LaTeX', '2.1μs Latency'],
      status: 'active' as const,
      href: '/projects/s-wind',
      icon: <MathIcon />,
    },
    {
      title: 'Wave',
      subtitle: 'Adaptive Programming Language',
      description: 'Self-hosting compiler with runtime optimization. 19x faster than Python, 30x lower memory. Compiles in <3ms.',
      features: ['Self-Hosting', 'Runtime Optimization', '19x Speed', 'Zero Dependencies'],
      status: 'active' as const,
      href: '/projects/wave',
      icon: <WaveIcon />,
    },
    {
      title: 'Val-OS',
      subtitle: 'Intelligent Operating System',
      description: 'Self-learning OS with adaptive resource management. Hybrid CLI/GUI, four-pool memory system, zero hardcoded values.',
      features: ['Self-Learning', 'Hybrid UI', 'Adaptive Memory', 'AI File Management'],
      status: 'research' as const,
      href: '/projects/val-os',
      icon: <OSIcon />,
    },
    {
      title: 'Yaksha Cipher',
      subtitle: 'Symmetric Encryption Suite',
      description: 'GPU-accelerated encryption with 1.4 GB/s throughput. ~50% avalanche effect, stateful authenticated encryption.',
      features: ['GPU Accelerated', '1.4 GB/s', 'AEAD Protocol', '~50% Avalanche'],
      status: 'active' as const,
      href: '/projects/yaksha-cipher',
      icon: <LockIcon />,
    },
    {
      title: 'Adaptive Compression',
      subtitle: 'Intelligent Data Compression',
      description: 'Multi-system compression achieving 0.4-12.5% ratio on high-entropy data where traditional algorithms fail (100%).',
      features: ['Intelligent Routing', 'High-Entropy Specialist', 'Multi-Cycle', 'Adaptive'],
      status: 'experimental' as const,
      href: '/projects/compression',
      icon: <CompressIcon />,
    },
    {
      title: 'Valhalla Core',
      subtitle: 'Algorithm Foundation',
      description: 'The unified architecture powering all projects. Adaptive resource allocation, cross-domain optimization, and intelligent coordination.',
      features: ['Unified Architecture', 'Cross-Domain', 'Adaptive', 'Foundation'],
      status: 'research' as const,
      href: '/projects/valhalla',
      icon: <ArchIcon />,
    },
  ]

  return (
    <>
      <Header />
      
      <main className="pt-16">
        {/* Hero Section */}
        <section className="relative min-h-[90vh] flex items-center justify-center overflow-hidden">
          <div className="absolute inset-0 bg-grid opacity-50" />
          <div className="absolute inset-0 bg-gradient-radial" />
          
          <div className="absolute top-1/4 right-1/4 w-64 h-64 bg-accent-primary/10 rounded-full blur-3xl animate-float" />
          <div className="absolute bottom-1/4 left-1/4 w-96 h-96 bg-accent-secondary/10 rounded-full blur-3xl animate-float delay-300" />
          
          <div className="container relative">
            <div className="max-w-4xl mx-auto text-center">
              <div className="animate-fade-in-up">
                <span className="inline-flex items-center gap-2 px-4 py-2 text-sm font-medium text-accent-primary bg-accent-primary/10 border border-accent-primary/20 rounded-full mb-8">
                  <span className="w-2 h-2 bg-accent-primary rounded-full animate-pulse" />
                  AI Research & Development Platform
                </span>
              </div>
              
              <h1 className="animate-fade-in-up delay-100 text-display-sm md:text-display font-bold mb-4">
                <span className="text-gradient">Valhalla</span>
              </h1>
              
              <p className="animate-fade-in-up delay-150 text-2xl md:text-3xl text-text-secondary mb-2">
                AI Incubator & Algorithm Research
              </p>
              
              <p className="animate-fade-in-up delay-200 text-lg md:text-xl text-text-tertiary leading-relaxed mb-10 max-w-2xl mx-auto">
                Autonomous discovery, adaptive systems, and intelligent computing.
                From programming languages to operating systems, encryption to compression.
              </p>

              <div className="animate-fade-in-up delay-300 flex flex-wrap justify-center gap-4 mb-10">
                {highlights.map((h, i) => (
                  <div 
                    key={i}
                    className="flex items-center gap-3 px-5 py-3 bg-bg-secondary/80 backdrop-blur-sm border border-border rounded-xl hover:border-accent-primary/50 transition-all"
                  >
                    <div className="text-accent-primary">{h.icon}</div>
                    <div className="text-left">
                      <div className="font-semibold text-text-primary text-sm">{h.title}</div>
                      <div className="text-xs text-text-tertiary">{h.desc}</div>
                    </div>
                  </div>
                ))}
              </div>
              
              <div className="animate-fade-in-up delay-400 flex flex-wrap justify-center gap-4">
                <Link href="/projects" className="btn btn-primary">
                  Explore Projects
                  <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 8l4 4m0 0l-4 4m4-4H3" />
                  </svg>
                </Link>
                <a href="https://github.com/Rogue-Intelligence-INC/Valhalla" className="btn btn-secondary" target="_blank" rel="noopener noreferrer">
                  <svg className="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                  </svg>
                  GitHub
                </a>
              </div>
            </div>
          </div>
        </section>

        {/* Metrics Bar */}
        <section className="border-y border-border bg-bg-secondary/50 backdrop-blur-sm">
          <div className="container py-8">
            <div className="grid grid-cols-2 md:grid-cols-5 gap-6 md:gap-8">
              <div className="text-center">
                <div className="text-2xl md:text-3xl font-bold text-gradient mb-1">6</div>
                <div className="text-sm text-text-tertiary">Core Projects</div>
              </div>
              <div className="text-center">
                <div className="text-2xl md:text-3xl font-bold text-gradient mb-1">1.4 GB/s</div>
                <div className="text-sm text-text-tertiary">Encryption Speed</div>
              </div>
              <div className="text-center">
                <div className="text-2xl md:text-3xl font-bold text-gradient mb-1">19x</div>
                <div className="text-sm text-text-tertiary">vs Python Speed</div>
              </div>
              <div className="text-center">
                <div className="text-2xl md:text-3xl font-bold text-gradient mb-1">99.97%</div>
                <div className="text-sm text-text-tertiary">Precision</div>
              </div>
              <div className="text-center">
                <div className="text-2xl md:text-3xl font-bold text-gradient mb-1">2.1μs</div>
                <div className="text-sm text-text-tertiary">Latency</div>
              </div>
            </div>
          </div>
        </section>

        {/* Projects Section */}
        <section className="py-20 md:py-28">
          <div className="container">
            <div className="text-center mb-16">
              <h2 className="text-2xl md:text-3xl font-bold mb-4">Projects</h2>
              <p className="text-text-secondary max-w-2xl mx-auto">
                From autonomous discovery to intelligent systems — a complete AI research ecosystem
              </p>
            </div>

            <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
              {projects.map((project, index) => (
                <div 
                  key={project.title} 
                  className="animate-fade-in-up"
                  style={{ animationDelay: `${index * 100}ms` }}
                >
                  <ProjectCard {...project} />
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* Capabilities Overview */}
        <section className="py-20 md:py-28 bg-bg-secondary">
          <div className="container">
            <div className="text-center mb-16">
              <h2 className="text-2xl md:text-3xl font-bold mb-4">Capabilities</h2>
              <p className="text-text-secondary">What Valhalla projects can do</p>
            </div>

            <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6 max-w-6xl mx-auto">
              {[
                { 
                  emoji: '🔍', 
                  title: 'Pattern Discovery', 
                  desc: 'Automatically find mathematical relationships in any numerical data',
                  metric: '10,000+ ops/sec'
                },
                { 
                  emoji: '⚡', 
                  title: 'High-Speed Compilation', 
                  desc: 'Self-hosting compiler with native code generation in milliseconds',
                  metric: '<3ms compile time'
                },
                { 
                  emoji: '🔐', 
                  title: 'Secure Encryption', 
                  desc: 'GPU-accelerated symmetric encryption with authenticated modes',
                  metric: '1.4 GB/s throughput'
                },
                { 
                  emoji: '🧠', 
                  title: 'Adaptive Systems', 
                  desc: 'Self-learning OS and runtime optimization that improves with use',
                  metric: 'Zero configuration'
                },
              ].map((cap, i) => (
                <div key={i} className="p-6 bg-bg-primary border border-border rounded-2xl hover:border-accent-primary/50 transition-all">
                  <div className="text-4xl mb-4">{cap.emoji}</div>
                  <h3 className="font-semibold mb-2">{cap.title}</h3>
                  <p className="text-sm text-text-secondary mb-4">{cap.desc}</p>
                  <div className="text-xs font-mono text-accent-primary">{cap.metric}</div>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* CTA Section */}
        <section className="py-20 md:py-28 relative overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-br from-accent-primary to-accent-secondary opacity-90" />
          <div className="absolute inset-0 bg-grid opacity-10" />
          
          <div className="container relative text-center">
            <h2 className="text-2xl md:text-3xl font-bold text-white mb-4">
              Ready to Explore?
            </h2>
            <p className="text-white/80 mb-8 max-w-2xl mx-auto">
              Discover the full potential of adaptive AI systems and intelligent computing.
            </p>
            <div className="flex flex-wrap justify-center gap-4">
              <Link 
                href="/projects" 
                className="btn bg-white text-accent-primary hover:bg-white/90"
              >
                View All Projects
              </Link>
              <a 
                href="https://rogue-intelligence-inc.github.io/Valhalla/" 
                target="_blank"
                rel="noopener noreferrer"
                className="btn bg-white/10 text-white border border-white/20 hover:bg-white/20"
              >
                Technical Docs
              </a>
            </div>
          </div>
        </section>
      </main>

      <Footer />
    </>
  )
}
