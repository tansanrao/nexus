import { createContext, useContext } from 'react';
import type { ThemeAppearance, ThemeId } from '../theme/presets';

export type ThemeModePreference = 'light' | 'dark' | 'system';

export interface ThemeSettingsContextValue {
  modePreference: ThemeModePreference;
  resolvedMode: ThemeAppearance;
  resolvedThemeId: ThemeId;
  lightSchemeId: ThemeId;
  darkSchemeId: ThemeId;
  availableLightThemes: ReadonlyArray<{ id: ThemeId; label: string }>;
  availableDarkThemes: ReadonlyArray<{ id: ThemeId; label: string }>;
  setModePreference: (mode: ThemeModePreference) => void;
  setLightScheme: (schemeId: string) => void;
  setDarkScheme: (schemeId: string) => void;
  resetDefaults: () => void;
}

export const ThemeSettingsContext =
  createContext<ThemeSettingsContextValue | undefined>(undefined);

export function useThemeSettings(): ThemeSettingsContextValue {
  const context = useContext(ThemeSettingsContext);
  if (!context) {
    throw new Error('useThemeSettings must be used within a ThemeProvider');
  }
  return context;
}
