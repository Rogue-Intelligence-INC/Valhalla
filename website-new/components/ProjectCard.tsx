import Link from 'next/link'

interface ProjectCardProps {
  title: string
  subtitle: string
  description: string
  features: string[]
  status: 'active' | 'research' | 'experimental'
  href: string
  icon: React.ReactNode
}

export default function ProjectCard({
  title,
  subtitle,
  description,
  features,
  status,
  href,
  icon
}: ProjectCardProps) {
  const statusConfig = {
    active: { 
      label: 'Active',
      className: 'tag-active',
    },
    research: { 
      label: 'Research',
      className: 'tag-research',
    },
    experimental: { 
      label: 'Experimental',
      className: 'tag-experimental',
    },
  }

  const config = statusConfig[status]

  return (
    <Link href={href} className="block group">
      <article className="card h-full flex flex-col relative overflow-hidden border border-border hover:border-border-strong transition-all bg-bg-primary/80 backdrop-blur-sm">
        <div className="relative">
          {/* Header */}
          <div className="flex items-start justify-between mb-5">
            <div className="w-12 h-12 border border-border flex items-center justify-center text-text-secondary group-hover:text-text-primary group-hover:border-border-strong transition-all">
              {icon}
            </div>
            <span className={`tag ${config.className}`}>
              {config.label}
            </span>
          </div>

          {/* Title & Subtitle */}
          <h3 className="font-serif text-xl font-medium text-text-primary mb-1 group-hover:underline group-hover:underline-offset-4 transition-all">
            {title}
          </h3>
          <p className="text-xs font-mono text-text-tertiary mb-4 tracking-wider">{subtitle}</p>

          {/* Description */}
          <p className="text-sm text-text-secondary leading-relaxed mb-5 line-clamp-3">
            {description}
          </p>

          {/* Features */}
          <div className="flex flex-wrap gap-2 mt-auto">
            {features.slice(0, 4).map((feature, index) => (
              <span 
                key={index} 
                className="text-xs font-mono px-2 py-1 border border-border text-text-tertiary uppercase tracking-wider"
              >
                {feature}
              </span>
            ))}
          </div>
        </div>

        {/* Footer */}
        <div className="relative mt-6 pt-4 border-t border-border flex items-center justify-between">
          <span className="text-sm font-mono tracking-wider text-text-tertiary group-hover:text-text-primary transition-colors uppercase">
            Learn more
          </span>
          <svg 
            className="w-4 h-4 text-text-tertiary group-hover:text-text-primary group-hover:translate-x-1 transition-all" 
            fill="none" 
            viewBox="0 0 24 24" 
            stroke="currentColor"
            strokeWidth={1.5}
          >
            <path strokeLinecap="square" strokeLinejoin="miter" d="M5 12h14M12 5l7 7-7 7" />
          </svg>
        </div>
      </article>
    </Link>
  )
}
