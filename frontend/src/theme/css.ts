import type { ThemeTokens } from './tokens';

const radius = {
  xs: '0.25rem',
  sm: '0.35rem',
  md: '0.45rem',
  lg: '0.6rem',
};

const shadow = {
  sm: '0 1px 1px 0 rgb(15 23 42 / 0.05)',
  md: '0 4px 10px -4px rgb(15 23 42 / 0.12)',
};

export function applyThemeTokens(tokens: ThemeTokens) {
  const root = document.documentElement;
  const body = document.body;

  const setVar = (name: string, value: string | number) => {
    root.style.setProperty(`--${name}`, String(value));
  };

  // Surface
  setVar('surface-base', tokens.colors.surface.base);
  setVar('surface-raised', tokens.colors.surface.raised);
  setVar('surface-muted', tokens.colors.surface.muted);
  setVar('surface-border', tokens.colors.surface.border);
  setVar('surface-overlay', tokens.colors.surface.overlay);

  // Text
  setVar('text-primary', tokens.colors.text.primary);
  setVar('text-secondary', tokens.colors.text.secondary);
  setVar('text-muted', tokens.colors.text.muted);
  setVar('text-accent', tokens.colors.text.accent);
  setVar('text-negative', tokens.colors.text.negative);

  // Accent
  setVar('accent-primary', tokens.colors.accent.primary);
  setVar('accent-secondary', tokens.colors.accent.secondary);
  setVar('accent-tertiary', tokens.colors.accent.tertiary);

  // States
  setVar('state-info', tokens.colors.states.info);
  setVar('state-success', tokens.colors.states.success);
  setVar('state-warning', tokens.colors.states.warning);
  setVar('state-danger', tokens.colors.states.danger);

  // Syntax
  setVar('syntax-keyword', tokens.colors.syntax.keyword);
  setVar('syntax-string', tokens.colors.syntax.string);
  setVar('syntax-number', tokens.colors.syntax.number);
  setVar('syntax-comment', tokens.colors.syntax.comment);
  setVar('syntax-background', tokens.colors.syntax.background);

  // Chart palette
  tokens.colors.chart.forEach((color, index) => {
    setVar(`chart-${index + 1}`, color);
  });

  // Typography
  setVar('font-family', tokens.typography.fontFamily);
  setVar('font-mono', tokens.typography.fontMono);
  setVar('font-line-height', String(tokens.typography.lineHeight));
  setVar('font-size-xs', tokens.typography.scale.xs);
  setVar('font-size-sm', tokens.typography.scale.sm);
  setVar('font-size-base', tokens.typography.scale.base);
  setVar('font-size-lg', tokens.typography.scale.lg);
  setVar('font-size-xl', tokens.typography.scale.xl);
  setVar('font-size-display', tokens.typography.scale.display);

  // Spacing
  setVar('space-xs', tokens.spacing.gapXs);
  setVar('space-sm', tokens.spacing.gapSm);
  setVar('space-md', tokens.spacing.gapMd);
  setVar('space-lg', tokens.spacing.gapLg);

  // Radii & shadows
  setVar('radius-xs', radius.xs);
  setVar('radius-sm', radius.sm);
  setVar('radius-md', radius.md);
  setVar('radius-lg', radius.lg);
  setVar('shadow-sm', shadow.sm);
  setVar('shadow-md', shadow.md);

  // Tailwind-compatible aliases
  setVar('color-background', tokens.colors.surface.base);
  setVar('color-foreground', tokens.colors.text.primary);
  setVar('color-card', tokens.colors.surface.raised);
  setVar('color-card-foreground', tokens.colors.text.primary);
  setVar('color-muted', tokens.colors.surface.muted);
  setVar('color-muted-foreground', tokens.colors.text.muted);
  setVar('color-accent', tokens.colors.accent.primary);
  setVar('color-accent-foreground', tokens.colors.text.primary);
  setVar('color-primary', tokens.colors.accent.primary);
  setVar('color-primary-foreground', tokens.colors.text.primary);
  setVar('color-secondary', tokens.colors.accent.secondary);
  setVar('color-secondary-foreground', tokens.colors.text.primary);
  setVar('color-destructive', tokens.colors.states.danger);
  setVar('color-destructive-foreground', tokens.colors.text.primary);
  setVar('color-border', tokens.colors.surface.border);
  setVar('color-input', tokens.colors.surface.overlay);
  setVar('color-ring', tokens.colors.accent.primary);

  setVar('ring', tokens.colors.accent.primary);

  // Body styles
  root.style.colorScheme = tokens.mode;
  root.dataset.themeId = tokens.id;
  root.dataset.themeMode = tokens.mode;
  body.style.backgroundColor = tokens.colors.surface.base;
  body.style.color = tokens.colors.text.primary;
}
