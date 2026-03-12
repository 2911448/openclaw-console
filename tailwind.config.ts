import type { Config } from 'tailwindcss'

export default {
  content: ['./index.html', './src/**/*.{vue,ts,tsx}'],
  theme: {
    extend: {
      colors: {
        bg: '#0b0b12',
        panel: '#111122',
        panel2: '#0f1020',
        stroke: 'rgba(255,255,255,0.08)',
        text: 'rgba(255,255,255,0.92)',
        subtext: 'rgba(255,255,255,0.60)',
        purple: {
          400: '#a78bfa',
          500: '#8b5cf6',
          600: '#7c3aed',
        },
      },
      boxShadow: {
        soft: '0 10px 30px rgba(0,0,0,0.35)',
      },
      borderRadius: {
        xl2: '18px',
      },
    },
  },
  plugins: [],
} satisfies Config
