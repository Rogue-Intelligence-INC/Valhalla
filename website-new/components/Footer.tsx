import Link from 'next/link'

export default function Footer() {
  const currentYear = new Date().getFullYear()

  const links = {
    research: [
      { name: 'Wave', href: '/projects/wave' },
      { name: 'S-wind', href: '/projects/s-wind' },
      { name: 'Val-OS', href: '/projects/val-os' },
      { name: 'Val-Crypt', href: '/projects/val-crypt' },
      { name: 'Valhalla Core', href: '/projects/valhalla' },
      { name: 'Compression', href: '/projects/compression' },
    ],
    resources: [
      { name: 'Papers', href: '/papers' },
      { name: 'Blog', href: '/blog' },
      { name: 'Videos', href: '/videos' },
      { name: 'Documentation', href: '/docs' },
    ],
  }

  return (
    <footer className="border-t border-border bg-bg-secondary">
      <div className="container py-16 md:py-20">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-10 md:gap-16">
          {/* Brand */}
          <div className="col-span-2">
            <Link href="/" className="inline-flex items-center gap-3 mb-6">
              <div className="w-10 h-10 flex items-center justify-center border-2 border-border-strong">
                <span className="font-serif font-medium text-xl">V</span>
              </div>
              <div>
                <span className="font-mono text-sm tracking-wider uppercase block">Project Valhalla</span>
                <span className="font-serif text-xs text-text-tertiary italic">Rogue Intelligence LNC</span>
              </div>
            </Link>
            <p className="text-sm text-text-tertiary max-w-sm leading-relaxed">
              Where exact mathematics meets physics-inspired computing. Advancing the frontiers of programming languages, 
              operating systems, cryptography, and mathematical computing.
            </p>
          </div>

          {/* Research */}
          <div>
            <h4 className="text-xs font-mono tracking-wider uppercase text-text-primary mb-6">Research</h4>
            <ul className="space-y-3">
              {links.research.map((link) => (
                <li key={link.name} className="before:content-none">
                  <Link 
                    href={link.href}
                    className="text-sm text-text-tertiary hover:text-text-primary transition-colors"
                  >
                    {link.name}
                  </Link>
                </li>
              ))}
            </ul>
          </div>

          {/* Resources */}
          <div>
            <h4 className="text-xs font-mono tracking-wider uppercase text-text-primary mb-6">Resources</h4>
            <ul className="space-y-3">
              {links.resources.map((link) => (
                <li key={link.name} className="before:content-none">
                  <Link 
                    href={link.href}
                    className="text-sm text-text-tertiary hover:text-text-primary transition-colors"
                  >
                    {link.name}
                  </Link>
                </li>
              ))}
            </ul>
          </div>
        </div>

        {/* Bottom */}
        <div className="mt-16 pt-8 border-t border-border flex flex-col sm:flex-row justify-between items-center gap-4">
          <p className="text-xs font-mono text-text-tertiary tracking-wider">
            © {currentYear} ROGUE INTELLIGENCE LNC
          </p>
          <div className="flex items-center gap-8 text-xs font-mono text-text-tertiary tracking-wider">
            <Link href="/admin" className="hover:text-text-primary transition-colors uppercase">
              Admin
            </Link>
            <a 
              href="https://github.com/joulymars/valhalla" 
              target="_blank" 
              rel="noopener noreferrer"
              className="hover:text-text-primary transition-colors uppercase"
            >
              GitHub
            </a>
          </div>
        </div>
      </div>
    </footer>
  )
}
