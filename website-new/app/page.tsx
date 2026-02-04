'use client'

import { useState, useEffect } from 'react'
import Header from '@/components/Header'
import Footer from '@/components/Footer'
import ProjectCard from '@/components/ProjectCard'
import WaveBackground from '@/components/WaveBackground'
import Link from 'next/link'

// Icons - Minimal, Geometric
const WaveIcon = () => (
  <svg className="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5}>
    <path d="M2 12c0-3 2.5-6 6-6s4 3 4 6 2.5 6 6 6 6-3 6-6" />
  </svg>
)

const OSIcon = () => (
  <svg className="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5}>
    <rect x="2" y="3" width="20" height="14" />
    <path d="M8 21h8M12 17v4" />
  </svg>
)

const MathIcon = () => (
  <svg className="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5}>
    <path d="M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4zM17 14v6M14 17h6" />
  </svg>
)

const ArchIcon = () => (
  <svg className="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5}>
    <rect x="3" y="3" width="18" height="18" />
    <path d="M3 9h18M9 3v18" />
  </svg>
)

const CryptoIcon = () => (
  <svg className="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5}>
    <rect x="3" y="11" width="18" height="10" />
    <path d="M7 11V7a5 5 0 0110 0v4" />
  </svg>
)

const CompressIcon = () => (
  <svg className="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5}>
    <path d="M4 12h16M12 4v16M8 8l4-4 4 4M8 16l4 4 4-4" />
  </svg>
)

export default function HomePage() {
  const projects = [
    {
      title: 'Wave',
      subtitle: 'Self-Hosting Programming Language',
      description: 'Rule-derived compilation through the Unified Field (i, e, r). Self-hosting compiler achieves 2.8× speedup over Python with zero external dependencies. Features Fate Architecture-Tagged Optimization.',
      features: ['Self-Hosting', '2.8× Speed', 'Fate Optimization', 'Zero Dependencies'],
      status: 'active' as const,
      href: '/projects/wave',
      icon: <WaveIcon />,
    },
    {
      title: 'S-wind',
      subtitle: 'Autonomous Algebraic Discovery',
      description: 'Exhaustive algebraic exploration over ℚ using Extended Precision Rationals (EPR). Achieves mathematical closure certification with self-loop pattern discovery. Zero floating-point—pure exact arithmetic.',
      features: ['EPR Arithmetic', 'Closure Proof', 'LaTeX Output', 'OEIS Integration'],
      status: 'experimental' as const,
      href: '/projects/s-wind',
      icon: <MathIcon />,
    },
    {
      title: 'Val-OS',
      subtitle: 'Wave Language Operating System',
      description: 'Experimental OS kernel written entirely in Wave. Fate+Quad memory management with zero hardcoded rules. Desktop environment, multi-user support, and built-in app store.',
      features: ['Wave Kernel', 'Fate+Quad Memory', 'Desktop Environment', 'App Store'],
      status: 'research' as const,
      href: '/projects/val-os',
      icon: <OSIcon />,
    },
    {
      title: 'Val-Crypt',
      subtitle: 'MeshBrain 2D GPU Encryption',
      description: 'Neural network-inspired encryption with 2D spatial processing. GPU acceleration via wgpu compute shaders (Vulkan/Metal/DX12). Achieves 42% avalanche effect—near the ideal 50%.',
      features: ['42% Avalanche', 'GPU Accelerated', '2D Spatial', 'CPU Parallel'],
      status: 'research' as const,
      href: '/projects/val-crypt',
      icon: <CryptoIcon />,
    },
    {
      title: 'Valhalla Core',
      subtitle: 'Unified Algorithm Architecture',
      description: 'Foundational quad-system: BlackHole (gravitational compression), MeshBrain (cognitive network), MultiNova (stellar parallel), BaseForce (coordination). Unified by Fate: weight = intensity × quality.',
      features: ['Quad System', 'Fate Coordination', 'Physics-Inspired', 'Cross-Disciplinary'],
      status: 'research' as const,
      href: '/projects/valhalla',
      icon: <ArchIcon />,
    },
    {
      title: 'Valhalla Compression',
      subtitle: 'Intelligent Quad-System Processing',
      description: 'Feature extraction (entropy, pattern, structure, density) drives Fate routing decisions. Automatic mode switching: learning (expansion) vs compression (contraction) based on data characteristics.',
      features: ['Fate Routing', 'Feature Extraction', 'Dual Mode', 'Adaptive'],
      status: 'experimental' as const,
      href: '/projects/compression',
      icon: <CompressIcon />,
    },
  ]

  const metrics = [
    { value: '6', label: 'Core Projects' },
    { value: '2.8×', label: 'Performance Gain' },
    { value: '100%', label: 'Self-Hosting' },
    { value: '0', label: 'Rounding Errors' },
  ]

  const philosophy = [
    { 
      title: 'Learn, Don\'t Assume', 
      description: 'Let systems discover optimal configurations through observation rather than predetermined rules.'
    },
    { 
      title: 'Both, Not Either', 
      description: 'Refuse false dichotomies—combine power with simplicity, exactness with performance.'
    },
    { 
      title: 'Prevent, Don\'t React', 
      description: 'Design systems that fix problems before they cause failures, not after.'
    },
    { 
      title: 'Adapt, Don\'t Hardcode', 
      description: 'Everything should be dynamic and determined at runtime based on actual conditions.'
    },
  ]

  const quadSystems = [
    { 
      name: 'BlackHole',
      domain: 'Extreme Data',
      formula: 'F_g = GM/r²',
      desc: 'Gravitational field transformation. Event horizon compression for extreme data scenarios.'
    },
    { 
      name: 'MeshBrain',
      domain: 'Cognitive Network',
      formula: 'membrane_state = f(signal, key)',
      desc: 'Neural membrane states with distributed processing. 51.2% avalanche effect—highest in system.'
    },
    { 
      name: 'MultiNova',
      domain: 'Large-Scale',
      formula: 'Q = 176L/(176-35L)',
      desc: 'Nova burst energy algorithms. Stellar evolution for large-scale parallel processing.'
    },
    { 
      name: 'BaseForce',
      domain: 'Coordination',
      formula: 'signal × α + history × (1-α)',
      desc: 'Three-system coordination with activity threshold control. Balances the quad.'
    },
  ]

  return (
    <>
      {/* Global Wave Background */}
      <WaveBackground />
      
      <Header />
      
      <main className="pt-16">
        {/* Hero Section with Animated Background */}
        <section className="relative min-h-[90vh] flex items-center overflow-hidden border-b border-border bg-animated-grid">
          {/* Moving Blur Orbs */}
          <div className="blur-orbs">
            <div className="blur-orb blur-orb-1" />
            <div className="blur-orb blur-orb-2" />
            <div className="blur-orb blur-orb-3" />
          </div>
          
          <div className="container content-relative">
            <div className="max-w-4xl">
              <div className="animate-fade-in-up">
                <span className="inline-block px-3 py-1 text-xs font-mono tracking-widest uppercase border border-border-strong mb-8">
                  Project Valhalla
                </span>
              </div>
              
              <h1 className="animate-fade-in-up delay-100 font-serif text-display-sm md:text-display font-light mb-4 tracking-tight">
                Rogue Intelligence
              </h1>
              <p className="animate-fade-in-up delay-100 text-xl md:text-2xl text-text-tertiary font-light mb-6 font-serif italic">
                LNC
              </p>
              
              <p className="animate-fade-in-up delay-200 text-lg text-text-secondary leading-relaxed mb-10 max-w-2xl">
                Where exact mathematics meets physics-inspired computing. Pioneering adaptive intelligence through self-hosting languages, 
                intelligent operating systems, and exact mathematical frameworks.
              </p>
              
              <div className="animate-fade-in-up delay-300 flex flex-wrap gap-4">
                <Link href="#projects" className="btn btn-primary">
                  Explore Research
                  <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
                  </svg>
                </Link>
                <Link href="/videos" className="btn btn-secondary">
                  <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                    <rect x="3" y="3" width="18" height="18" strokeWidth={1.5} />
                  </svg>
                  Watch Videos
                </Link>
              </div>
            </div>
          </div>
        </section>

        {/* Metrics Bar */}
        <section className="border-b border-border bg-bg-secondary">
          <div className="container py-12">
            <div className="grid grid-cols-2 md:grid-cols-4 gap-8 md:gap-12">
              {metrics.map((metric, index) => (
                <div key={index} className="text-center">
                  <div className="text-3xl md:text-4xl font-serif font-light text-text-primary mb-2">
                    {metric.value}
                  </div>
                  <div className="text-xs font-mono tracking-wider text-text-tertiary uppercase">{metric.label}</div>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* Core Formula Section */}
        <section className="py-20 md:py-28 border-b border-border relative bg-animated-grid">
          <div className="blur-orbs" style={{ opacity: 0.5 }}>
            <div className="blur-orb blur-orb-2" />
          </div>
          
          <div className="container content-relative">
            <div className="max-w-3xl mx-auto text-center">
              <h2 className="font-serif text-2xl md:text-3xl font-light mb-6">
                The Fate Formula
              </h2>
              <div className="diagram inline-block text-lg mb-8">
                weight = intensity × quality
              </div>
              <p className="text-text-secondary leading-relaxed">
                The unified coordination mechanism across all Valhalla systems. Adaptive routing decisions based on 
                signal intensity and quality metrics, enabling self-optimizing behavior without hardcoded rules.
              </p>
            </div>
          </div>
        </section>

        {/* Architecture Diagram */}
        <section className="py-20 md:py-28 bg-bg-secondary border-b border-border">
          <div className="container">
            <div className="text-center mb-12">
              <h2 className="font-serif text-2xl md:text-3xl font-light mb-4">Core Architecture</h2>
            </div>
            
            <div className="diagram max-w-4xl mx-auto">
{`┌─────────────────────────────────────────────────────────────────────┐
│                        PROJECT VALHALLA                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    FATE COORDINATION                         │   │
│  │               weight = intensity × quality                   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              │                                      │
│  ┌──────────┬──────────┬──────────┬──────────┐                     │
│  │ BlackHole │ MeshBrain │ MultiNova │ BaseForce │                  │
│  │  F=GM/r²  │  Neural   │  Q=f(L)  │  Balance │                   │
│  │  Extreme  │ Cognitive │  Stellar │ Coordinate│                   │
│  └──────────┴──────────┴──────────┴──────────┘                     │
│                              │                                      │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                      APPLICATIONS                             │  │
│  │      Wave · Val-OS · S-wind · Val-Crypt · Compression        │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘`}
            </div>
          </div>
        </section>

        {/* Quad System Detail */}
        <section className="py-20 md:py-28 border-b border-border relative bg-animated-grid">
          <div className="blur-orbs" style={{ opacity: 0.4 }}>
            <div className="blur-orb blur-orb-1" />
            <div className="blur-orb blur-orb-3" />
          </div>
          
          <div className="container content-relative">
            <div className="text-center mb-16">
              <h2 className="font-serif text-2xl md:text-3xl font-light mb-4">The Quad System</h2>
              <p className="text-text-secondary max-w-2xl mx-auto">
                Four physics-inspired subsystems working in concert, each handling different computational domains.
              </p>
            </div>
            
            <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6 max-w-5xl mx-auto">
              {quadSystems.map((system, index) => (
                <div key={index} className="border border-border p-5 bg-bg-primary/80 backdrop-blur-sm">
                  <div className="font-mono text-xs tracking-wider text-text-tertiary mb-2 uppercase">
                    {system.domain}
                  </div>
                  <h3 className="font-serif text-xl mb-2">{system.name}</h3>
                  <code className="text-xs text-text-tertiary block mb-3 font-mono">
                    {system.formula}
                  </code>
                  <p className="text-xs text-text-secondary leading-relaxed">{system.desc}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* Projects Section */}
        <section id="projects" className="py-20 md:py-28 border-b border-border bg-bg-secondary">
          <div className="container">
            <div className="text-center mb-16">
              <h2 className="font-serif text-2xl md:text-3xl font-light mb-4">Research Projects</h2>
              <p className="text-text-secondary max-w-2xl mx-auto">
                Our research spans programming languages, operating systems, cryptography, and mathematical computing—
                unified by the principle of adaptive intelligence through exact computation.
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

        {/* Philosophy Section */}
        <section className="py-20 md:py-28 border-b border-border relative bg-animated-grid">
          <div className="blur-orbs" style={{ opacity: 0.3 }}>
            <div className="blur-orb blur-orb-2" />
            <div className="blur-orb blur-orb-3" />
          </div>
          
          <div className="container content-relative">
            <div className="text-center mb-16">
              <h2 className="font-serif text-2xl md:text-3xl font-light mb-4">Philosophy</h2>
            </div>
            
            <div className="grid md:grid-cols-2 gap-8 max-w-4xl mx-auto">
              {philosophy.map((item, index) => (
                <div key={index} className="border border-border p-6 bg-bg-primary/80 backdrop-blur-sm">
                  <div className="flex items-start gap-4">
                    <div className="flex-shrink-0 w-10 h-10 border border-border-strong flex items-center justify-center font-mono text-sm">
                      {String(index + 1).padStart(2, '0')}
                    </div>
                    <div>
                      <h3 className="font-serif text-lg mb-2">{item.title}</h3>
                      <p className="text-sm text-text-secondary leading-relaxed">{item.description}</p>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* Technical Specs */}
        <section className="py-20 md:py-28 bg-bg-secondary border-b border-border">
          <div className="container">
            <div className="text-center mb-12">
              <h2 className="font-serif text-2xl md:text-3xl font-light mb-4">Technical Specifications</h2>
            </div>
            
            <div className="max-w-2xl mx-auto">
              <table className="w-full">
                <tbody>
                  <tr>
                    <td className="font-mono text-sm text-text-tertiary py-3 border-b border-border">Languages</td>
                    <td className="text-sm py-3 border-b border-border">Rust, Wave, C</td>
                  </tr>
                  <tr>
                    <td className="font-mono text-sm text-text-tertiary py-3 border-b border-border">Arithmetic</td>
                    <td className="text-sm py-3 border-b border-border">Exact Rationals (i128/i128)</td>
                  </tr>
                  <tr>
                    <td className="font-mono text-sm text-text-tertiary py-3 border-b border-border">GPU Support</td>
                    <td className="text-sm py-3 border-b border-border">wgpu (Vulkan/Metal/DX12)</td>
                  </tr>
                  <tr>
                    <td className="font-mono text-sm text-text-tertiary py-3 border-b border-border">Platforms</td>
                    <td className="text-sm py-3 border-b border-border">Linux (primary), macOS, Windows</td>
                  </tr>
                  <tr>
                    <td className="font-mono text-sm text-text-tertiary py-3 border-b border-border">Self-Hosting</td>
                    <td className="text-sm py-3 border-b border-border">100%</td>
                  </tr>
                  <tr>
                    <td className="font-mono text-sm text-text-tertiary py-3">License</td>
                    <td className="text-sm py-3">MIT</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </section>

        {/* CTA Section */}
        <section className="py-20 md:py-28 bg-bg-inverse text-text-inverse relative overflow-hidden">
          <div className="absolute inset-0 bg-grid opacity-10" />
          
          <div className="container text-center relative z-10">
            <h2 className="font-serif text-2xl md:text-3xl font-light mb-4 text-white">
              Interested in Our Research?
            </h2>
            <p className="text-white/60 mb-10 max-w-2xl mx-auto">
              We welcome collaboration with researchers, institutions, and industry partners.
            </p>
            <div className="flex flex-wrap justify-center gap-4">
              <a 
                href="mailto:contact@rogue-intelligence.com" 
                className="btn bg-white text-black border-white hover:bg-transparent hover:text-white"
              >
                Contact Us
              </a>
              <Link 
                href="https://github.com/joulymars/valhalla"
                className="btn bg-transparent text-white border-white hover:bg-white hover:text-black"
              >
                View on GitHub
              </Link>
            </div>
          </div>
        </section>
      </main>

      <Footer />
    </>
  )
}
