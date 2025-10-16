import catppuccin from './palettes/catppuccin';
import solarized from './palettes/solarized';
import type { ThemeMode, ThemeSummary, ThemeTokens } from './tokens';

const themes: ThemeTokens[] = [
  ...catppuccin,
  ...solarized,
];

const themeMap = new Map<string, ThemeTokens>();

for (const theme of themes) {
  if (themeMap.has(theme.id)) {
    throw new Error(`Duplicate theme id detected: ${theme.id}`);
  }
  themeMap.set(theme.id, theme);
}

export const DEFAULT_LIGHT_THEME_ID = 'catppuccin-latte';
export const DEFAULT_DARK_THEME_ID = 'catppuccin-mocha';

export function getThemesByMode(mode: ThemeMode): ThemeTokens[] {
  return themes.filter((theme) => theme.mode === mode);
}

export function getTheme(id: string): ThemeTokens | undefined {
  return themeMap.get(id);
}

export function listThemeSummaries(): ThemeSummary[] {
  return themes.map((theme) => ({
    id: theme.id,
    label: theme.label,
    mode: theme.mode,
    accent: theme.colors.accent.primary,
    surface: theme.colors.surface.base,
  }));
}

export function resolveTheme(id: string, fallbackMode: ThemeMode): ThemeTokens {
  const found = getTheme(id);
  if (found) return found;
  const fallbackList = getThemesByMode(fallbackMode);
  const fallback = fallbackList.find((theme) => theme.id === (fallbackMode === 'dark' ? DEFAULT_DARK_THEME_ID : DEFAULT_LIGHT_THEME_ID));
  return fallback ?? fallbackList[0];
}

export const allThemes = themes;
