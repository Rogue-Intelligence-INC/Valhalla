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
      dot: 'status-dot active'
    },
    research: { 
      label: 'Research',
      className: 'tag-research',
      dot: 'status-dot research'
    },
    experimental: { 
      label: 'Experimental',
      className: 'tag-experimental',
      dot: 'status-dot experimental'
    },
  }

  const config = statusConfig[status]

  return (
    <Link href={href} className="block group">
      <article className="card h-full flex flex-col relative overflow-hidden">
        {/* Gradient accent on hover */}
        <div className="absolute inset-0 bg-gradient-to-br from-accent-primary/5 via-transparent to-accent-secondary/5 opacity-0 group-hover:opacity-100 transition-opacity duration-500" />
        
        <div className="relative">
          {/* Header */}
          <div className="flex items-start justify-between mb-5">
            <div className="w-11 h-11 rounded-xl bg-bg-tertiary flex items-center justify-center text-text-secondary group-hover:text-accent-primary group-hover:bg-accent-primary/10 transition-all duration-300">
              {icon}
            </div>
            <span className={`tag ${config.className} flex items-center gap-1.5`}>
              <span className={config.dot} />
              {config.label}
            </span>
          </div>

          {/* Title & Subtitle */}
          <h3 className="text-lg font-semibold text-text-primary mb-1 group-hover:text-accent-primary transition-colors">
            {title}
          </h3>
          <p className="text-xs font-mono text-text-tertiary mb-3">{subtitle}</p>

          {/* Description */}
          <p className="text-sm text-text-secondary leading-relaxed mb-5 line-clamp-3">
            {description}
          </p>

          {/* Features */}
          <div className="flex flex-wrap gap-1.5 mt-auto">
            {features.slice(0, 4).map((feature, index) => (
              <span 
                key={index} 
                className="text-xs px-2 py-1 rounded-md bg-bg-tertiary text-text-tertiary"
              >
                {feature}
              </span>
            ))}
          </div>
        </div>

        {/* Footer */}
        <div className="relative mt-5 pt-4 border-t border-border flex items-center justify-between">
          <span className="text-sm text-text-tertiary group-hover:text-accent-primary transition-colors">
            Learn more
          </span>
          <svg 
            className="w-4 h-4 text-text-tertiary group-hover:text-accent-primary group-hover:translate-x-1 transition-all" 
            fill="none" 
            viewBox="0 0 24 24" 
            stroke="currentColor"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 8l4 4m0 0l-4 4m4-4H3" />
          </svg>
        </div>
      </article>
    </Link>
  )
}
