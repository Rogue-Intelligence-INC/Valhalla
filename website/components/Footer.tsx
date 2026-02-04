import Link from 'next/link'

export default function Footer() {
  const currentYear = new Date().getFullYear()

  const links = {
    research: [
      { name: 'Wave', href: '/projects/wave' },
      { name: 'Val-OS', href: '/projects/val-os' },
      { name: 'S-wind', href: '/projects/s-wind' },
      { name: 'Valhalla', href: '/projects/valhalla' },
    ],
    resources: [
      { name: 'Papers', href: '/papers' },
      { name: 'Blog', href: '/blog' },
      { name: 'Videos', href: '/videos' },
    ],
  }

  return (
    <footer className="border-t border-border bg-bg-secondary">
      <div className="container py-12 md:py-16">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-8 md:gap-12">
          {/* Brand */}
          <div className="col-span-2">
            <Link href="/" className="inline-flex items-center gap-3 mb-4">
              <div className="w-8 h-8 flex items-center justify-center bg-accent-primary/10 rounded-lg">
                <span className="font-display font-bold text-lg text-accent-primary">R</span>
              </div>
              <span className="font-semibold text-text-primary">Rogue Intelligence</span>
            </Link>
            <p className="text-sm text-text-tertiary max-w-sm leading-relaxed">
              Advancing the frontiers of programming languages, operating systems, and mathematical computing.
            </p>
          </div>

          {/* Research */}
          <div>
            <h4 className="text-sm font-semibold text-text-primary mb-4">Research</h4>
            <ul className="space-y-2.5">
              {links.research.map((link) => (
                <li key={link.name}>
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
            <h4 className="text-sm font-semibold text-text-primary mb-4">Resources</h4>
            <ul className="space-y-2.5">
              {links.resources.map((link) => (
                <li key={link.name}>
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
        <div className="mt-12 pt-8 border-t border-border flex flex-col sm:flex-row justify-between items-center gap-4">
          <p className="text-xs text-text-tertiary">
            © {currentYear} Rogue Intelligence LNC
          </p>
          <div className="flex items-center gap-6 text-xs text-text-tertiary">
            <Link href="/admin" className="hover:text-text-secondary transition-colors">
              Admin
            </Link>
          </div>
        </div>
      </div>
    </footer>
  )
}
