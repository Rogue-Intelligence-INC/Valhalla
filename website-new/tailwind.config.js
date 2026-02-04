/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './pages/**/*.{js,ts,jsx,tsx,mdx}',
    './components/**/*.{js,ts,jsx,tsx,mdx}',
    './app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // Pure Monochrome Scale
        bg: {
          primary: 'var(--bg-primary)',
          secondary: 'var(--bg-secondary)',
          tertiary: 'var(--bg-tertiary)',
          elevated: 'var(--bg-elevated)',
          inverse: 'var(--bg-inverse)',
        },
        text: {
          primary: 'var(--text-primary)',
          secondary: 'var(--text-secondary)',
          tertiary: 'var(--text-tertiary)',
          inverse: 'var(--text-inverse)',
        },
        border: {
          DEFAULT: 'var(--border-default)',
          subtle: 'var(--border-subtle)',
          strong: 'var(--border-strong)',
        },
        accent: {
          primary: 'var(--accent-primary)',
          'primary-hover': 'var(--accent-primary-hover)',
          secondary: 'var(--accent-secondary)',
          success: 'var(--accent-success)',
          warning: 'var(--accent-warning)',
          danger: 'var(--accent-danger)',
        },
      },
      fontFamily: {
        // Classical Serif Fonts
        sans: ['EB Garamond', 'Georgia', 'Times New Roman', 'serif'],
        serif: ['Cormorant Garamond', 'Georgia', 'serif'],
        display: ['Cormorant Garamond', 'Georgia', 'serif'],
        mono: ['JetBrains Mono', 'Courier New', 'monospace'],
      },
      fontSize: {
        'display-lg': ['5rem', { lineHeight: '1.05', letterSpacing: '-0.03em' }],
        'display': ['4rem', { lineHeight: '1.1', letterSpacing: '-0.02em' }],
        'display-sm': ['2.75rem', { lineHeight: '1.15', letterSpacing: '-0.02em' }],
      },
      spacing: {
        '18': '4.5rem',
        '22': '5.5rem',
        'header': 'var(--header-height)',
      },
      maxWidth: {
        'content': 'var(--content-width)',
      },
      // No border radius - square design
      borderRadius: {
        'none': '0',
        'sm': '0',
        'DEFAULT': '0',
        'md': '0',
        'lg': '0',
        'xl': '0',
        '2xl': '0',
        '3xl': '0',
        'full': '0',
      },
      boxShadow: {
        'sm': '0 1px 0 rgba(0, 0, 0, 0.05)',
        'md': '0 4px 0 rgba(0, 0, 0, 0.05)',
        'lg': '0 8px 0 rgba(0, 0, 0, 0.08)',
        'glow': 'none',
        'glow-sm': 'none',
        'glow-lg': 'none',
      },
      backgroundImage: {
        'gradient-radial': 'none',
        'gradient-conic': 'none',
        'grid-pattern': `url("data:image/svg+xml,%3Csvg width='80' height='80' viewBox='0 0 80 80' xmlns='http://www.w3.org/2000/svg'%3E%3Cg fill='none' fill-rule='evenodd'%3E%3Cpath d='M0 0h80v80H0z' fill='%23888' fill-opacity='0.03'/%3E%3C/g%3E%3C/svg%3E")`,
      },
      animation: {
        'fade-in-up': 'fadeInUp 0.5s ease-out forwards',
        'fade-in': 'fadeIn 0.4s ease-out forwards',
        'float': 'float 8s ease-in-out infinite',
        'pulse-slow': 'pulse 4s ease-in-out infinite',
      },
      keyframes: {
        fadeInUp: {
          '0%': { opacity: '0', transform: 'translateY(30px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        float: {
          '0%, 100%': { transform: 'translateY(0)' },
          '50%': { transform: 'translateY(-5px)' },
        },
      },
      transitionDuration: {
        '400': '400ms',
      },
      transitionTimingFunction: {
        'smooth': 'linear',
      },
      letterSpacing: {
        'tightest': '-0.04em',
        'tighter': '-0.02em',
        'tight': '-0.01em',
        'wide': '0.05em',
        'wider': '0.1em',
        'widest': '0.15em',
      },
    },
  },
  plugins: [],
}
