import type { Metadata } from 'next'
import './globals.css'

export const metadata: Metadata = {
  title: 'Rogue Intelligence | Advanced Computing Research',
  description: 'Pioneering research in programming languages, operating systems, and mathematical computing. Building the future of adaptive intelligent systems.',
  keywords: ['Wave Language', 'Val-OS', 'S-wind', 'Valhalla', 'adaptive systems', 'exact computation'],
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <script
          dangerouslySetInnerHTML={{
            __html: `
              (function() {
                try {
                  var theme = localStorage.getItem('theme');
                  if (theme === 'dark' || (!theme && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
                    document.documentElement.classList.add('dark');
                  } else {
                    document.documentElement.classList.remove('dark');
                  }
                } catch (e) {}
              })();
            `,
          }}
        />
      </head>
      <body className="min-h-screen bg-bg-primary text-text-primary transition-colors">
        {children}
      </body>
    </html>
  )
}
