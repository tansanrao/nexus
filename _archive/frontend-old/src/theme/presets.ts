import type { ComponentProps } from 'react';
import { Theme as RadixTheme } from '@radix-ui/themes';

type RadixAccent = NonNullable<ComponentProps<typeof RadixTheme>['accentColor']>;
type RadixGray = NonNullable<ComponentProps<typeof RadixTheme>['grayColor']>;
type RadixPanelBackground = ComponentProps<typeof RadixTheme>['panelBackground'];

export type ThemeAppearance = 'light' | 'dark';
export type ThemeId = 'light' | 'dark' | 'solarized-light' | 'solarized-dark';

export type ThemeTokenName =
  | '--color-background'
  | '--color-foreground'
  | '--color-card'
  | '--color-card-foreground'
  | '--color-popover'
  | '--color-popover-foreground'
  | '--color-primary'
  | '--color-primary-foreground'
  | '--color-secondary'
  | '--color-secondary-foreground'
  | '--color-muted'
  | '--color-muted-foreground'
  | '--color-accent'
  | '--color-accent-foreground'
  | '--color-destructive'
  | '--color-destructive-foreground'
  | '--color-border'
  | '--color-input'
  | '--color-ring'
  | '--color-panel-left'
  | '--color-panel-right'
  | '--color-panel-inset'
  | '--color-panel-border';

type ThemeTokens = Record<ThemeTokenName, string>;

export type ThemePreset = {
  id: ThemeId;
  label: string;
  appearance: ThemeAppearance;
  radix: {
    accentColor: RadixAccent;
    grayColor: RadixGray;
    panelBackground?: RadixPanelBackground;
  };
  tokens: ThemeTokens;
};

const themeTokens = {
  light: {
    '--color-background': '36 42% 95%',
    '--color-foreground': '25 32% 15%',
    '--color-card': '36 52% 99%',
    '--color-card-foreground': '25 32% 16%',
    '--color-popover': '36 52% 99%',
    '--color-popover-foreground': '25 32% 16%',
    '--color-primary': '26 74% 46%',
    '--color-primary-foreground': '36 100% 98%',
    '--color-secondary': '35 28% 88%',
    '--color-secondary-foreground': '25 32% 18%',
    '--color-muted': '35 25% 86%',
    '--color-muted-foreground': '24 18% 38%',
    '--color-accent': '28 70% 92%',
    '--color-accent-foreground': '25 32% 18%',
    '--color-destructive': '358 70% 50%',
    '--color-destructive-foreground': '36 100% 98%',
    '--color-border': '30 20% 72%',
    '--color-input': '30 20% 72%',
    '--color-ring': '26 74% 46%',
    '--color-panel-left': '32 36% 94%',
    '--color-panel-right': '36 52% 98%',
    '--color-panel-inset': '30 28% 70%',
    '--color-panel-border': '28 20% 70%',
  },
  dark: {
    '--color-background': '222 47% 11%',
    '--color-foreground': '210 40% 98%',
    '--color-card': '222 47% 13%',
    '--color-card-foreground': '210 40% 98%',
    '--color-popover': '222 47% 13%',
    '--color-popover-foreground': '210 40% 98%',
    '--color-primary': '217 91% 60%',
    '--color-primary-foreground': '222 47% 11%',
    '--color-secondary': '215 28% 17%',
    '--color-secondary-foreground': '210 40% 98%',
    '--color-muted': '215 16% 47%',
    '--color-muted-foreground': '215 20% 65%',
    '--color-accent': '215 25% 27%',
    '--color-accent-foreground': '210 40% 98%',
    '--color-destructive': '0 84% 60%',
    '--color-destructive-foreground': '222 47% 11%',
    '--color-border': '215 16% 47%',
    '--color-input': '215 16% 47%',
    '--color-ring': '217 91% 60%',
    '--color-panel-left': '215 28% 25%',
    '--color-panel-right': '222 47% 13%',
    '--color-panel-inset': '215 25% 20%',
    '--color-panel-border': '215 16% 47%',
  },
  'solarized-light': {
    '--color-background': '44 87% 94%',
    '--color-foreground': '194 13% 45%',
    '--color-card': '44 87% 96%',
    '--color-card-foreground': '194 13% 43%',
    '--color-popover': '44 87% 96%',
    '--color-popover-foreground': '194 13% 43%',
    '--color-primary': '205 71% 52%',
    '--color-primary-foreground': '44 87% 96%',
    '--color-secondary': '44 45% 87%',
    '--color-secondary-foreground': '194 26% 32%',
    '--color-muted': '186 13% 60%',
    '--color-muted-foreground': '194 26% 32%',
    '--color-accent': '174 47% 46%',
    '--color-accent-foreground': '44 87% 96%',
    '--color-destructive': '18 71% 45%',
    '--color-destructive-foreground': '44 87% 96%',
    '--color-border': '186 13% 60%',
    '--color-input': '186 13% 60%',
    '--color-ring': '205 71% 52%',
    '--color-panel-left': '45 45% 86%',
    '--color-panel-right': '186 38% 84%',
    '--color-panel-inset': '186 24% 68%',
    '--color-panel-border': '186 23% 46%',
  },
  'solarized-dark': {
    '--color-background': '193 92% 12%',
    '--color-foreground': '195 12% 57%',
    '--color-card': '193 86% 14%',
    '--color-card-foreground': '195 12% 57%',
    '--color-popover': '193 86% 14%',
    '--color-popover-foreground': '195 12% 57%',
    '--color-primary': '205 71% 52%',
    '--color-primary-foreground': '193 97% 10%',
    '--color-secondary': '193 70% 18%',
    '--color-secondary-foreground': '195 12% 57%',
    '--color-muted': '194 26% 40%',
    '--color-muted-foreground': '195 12% 65%',
    '--color-accent': '174 47% 46%',
    '--color-accent-foreground': '193 97% 10%',
    '--color-destructive': '18 71% 45%',
    '--color-destructive-foreground': '193 97% 10%',
    '--color-border': '194 26% 35%',
    '--color-input': '194 26% 35%',
    '--color-ring': '205 71% 52%',
    '--color-panel-left': '193 55% 22%',
    '--color-panel-right': '193 70% 18%',
    '--color-panel-inset': '194 32% 26%',
    '--color-panel-border': '194 26% 32%',
  },
} as const satisfies Record<ThemeId, ThemeTokens>;

export const themePresets = {
  light: {
    id: 'light',
    label: 'Default Light',
    appearance: 'light',
    radix: {
      accentColor: 'iris' satisfies RadixAccent,
      grayColor: 'sand' satisfies RadixGray,
      panelBackground: 'solid' satisfies RadixPanelBackground,
    },
    tokens: themeTokens.light,
  },
  dark: {
    id: 'dark',
    label: 'Default Dark',
    appearance: 'dark',
    radix: {
      accentColor: 'violet' satisfies RadixAccent,
      grayColor: 'slate' satisfies RadixGray,
      panelBackground: 'solid' satisfies RadixPanelBackground,
    },
    tokens: themeTokens.dark,
  },
  'solarized-light': {
    id: 'solarized-light',
    label: 'Solarized Light',
    appearance: 'light',
    radix: {
      accentColor: 'cyan' satisfies RadixAccent,
      grayColor: 'sage' satisfies RadixGray,
      panelBackground: 'solid' satisfies RadixPanelBackground,
    },
    tokens: themeTokens['solarized-light'],
  },
  'solarized-dark': {
    id: 'solarized-dark',
    label: 'Solarized Dark',
    appearance: 'dark',
    radix: {
      accentColor: 'cyan' satisfies RadixAccent,
      grayColor: 'olive' satisfies RadixGray,
      panelBackground: 'solid' satisfies RadixPanelBackground,
    },
    tokens: themeTokens['solarized-dark'],
  },
} as const satisfies Record<ThemeId, ThemePreset>;

export const THEME_IDS = Object.keys(themePresets) as ThemeId[];

export const DEFAULT_LIGHT_THEME_ID: ThemeId = 'light';
export const DEFAULT_DARK_THEME_ID: ThemeId = 'dark';

export const LIGHT_THEME_OPTIONS: ReadonlyArray<{ id: ThemeId; label: string }> = THEME_IDS.filter(
  (themeId) => themePresets[themeId].appearance === 'light'
).map((themeId) => ({
  id: themeId,
  label: themePresets[themeId].label,
}));

export const DARK_THEME_OPTIONS: ReadonlyArray<{ id: ThemeId; label: string }> = THEME_IDS.filter(
  (themeId) => themePresets[themeId].appearance === 'dark'
).map((themeId) => ({
  id: themeId,
  label: themePresets[themeId].label,
}));

export const themeClassNames: ThemeId[] = [...THEME_IDS];
