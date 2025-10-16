import type { ThemeTokens } from './tokens';

export const baseTypography: ThemeTokens['typography'] = {
  fontFamily: "var(--font-sans, 'Inter', system-ui, sans-serif)",
  fontMono: "var(--font-mono, 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace)",
  lineHeight: 1.5,
  scale: {
    xs: '0.75rem',   // 12px
    sm: '0.8125rem', // 13px
    base: '0.875rem',// 14px
    lg: '1rem',      // 16px
    xl: '1.25rem',   // 20px
    display: '1.5rem', // 24px
  },
};

export const baseSpacing: ThemeTokens['spacing'] = {
  gapXs: '0.25rem',
  gapSm: '0.5rem',
  gapMd: '0.75rem',
  gapLg: '1rem',
};
