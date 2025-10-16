/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './index.html',
    './src/**/*.{js,ts,jsx,tsx}',
  ],
  theme: {
    extend: {
      colors: {
        border: 'hsl(var(--color-border) / 1)',
        input: 'hsl(var(--color-input) / 1)',
        ring: 'hsl(var(--color-ring) / 1)',
        background: 'hsl(var(--color-background) / 1)',
        foreground: 'hsl(var(--color-foreground) / 1)',
        primary: {
          DEFAULT: 'hsl(var(--color-primary) / 1)',
          foreground: 'hsl(var(--color-primary-foreground) / 1)',
        },
        secondary: {
          DEFAULT: 'hsl(var(--color-secondary) / 1)',
          foreground: 'hsl(var(--color-secondary-foreground) / 1)',
        },
        destructive: {
          DEFAULT: 'hsl(var(--color-destructive) / 1)',
          foreground: 'hsl(var(--color-destructive-foreground) / 1)',
        },
        muted: {
          DEFAULT: 'hsl(var(--color-muted) / 1)',
          foreground: 'hsl(var(--color-muted-foreground) / 1)',
        },
        accent: {
          DEFAULT: 'hsl(var(--color-accent) / 1)',
          foreground: 'hsl(var(--color-accent-foreground) / 1)',
        },
        popover: {
          DEFAULT: 'hsl(var(--color-popover) / 1)',
          foreground: 'hsl(var(--color-popover-foreground) / 1)',
        },
        card: {
          DEFAULT: 'hsl(var(--color-card) / 1)',
          foreground: 'hsl(var(--color-card-foreground) / 1)',
        },
        surface: {
          base: 'hsl(var(--color-panel-right) / 1)',
          raised: 'hsl(var(--color-panel-left) / 1)',
          inset: 'hsl(var(--color-panel-inset) / 1)',
          border: 'hsl(var(--color-panel-border) / 1)',
        },
      },
    },
  },
}
