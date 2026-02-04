import type { Metadata } from 'next'
import './globals.css'

export const metadata: Metadata = {
  title: 'Project Valhalla | Rogue Intelligence',
  description: 'Where exact mathematics meets physics-inspired computing. Research in programming languages, operating systems, cryptography, and mathematical computing.',
  keywords: ['Wave Language', 'Val-OS', 'S-wind', 'Valhalla', 'Val-Crypt', 'exact computation', 'adaptive systems', 'self-hosting', 'Fate optimization'],
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
                  if (theme === 'light') {
                    document.documentElement.classList.remove('dark');
                  } else {
                    // Default to dark mode
                    document.documentElement.classList.add('dark');
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
