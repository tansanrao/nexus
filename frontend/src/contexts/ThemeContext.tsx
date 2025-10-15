import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react';
import { applyThemeTokens } from '../theme/css';
import {
  DEFAULT_DARK_THEME_ID,
  DEFAULT_LIGHT_THEME_ID,
  getTheme,
  getThemesByMode,
  resolveTheme,
} from '../theme/registry';
import type { ThemeTokens } from '../theme/tokens';

export type ThemeModePreference = 'light' | 'dark' | 'system';

interface ThemeContextValue {
  modePreference: ThemeModePreference;
  resolvedMode: 'light' | 'dark';
  lightSchemeId: string;
  darkSchemeId: string;
  availableLightThemes: ThemeTokens[];
  availableDarkThemes: ThemeTokens[];
  effectiveTokens: ThemeTokens;
  setModePreference: (mode: ThemeModePreference) => void;
  setLightScheme: (schemeId: string) => void;
  setDarkScheme: (schemeId: string) => void;
  resetDefaults: () => void;
}

const MODE_KEY = 'theme.modePreference';
const LIGHT_SCHEME_KEY = 'theme.lightScheme';
const DARK_SCHEME_KEY = 'theme.darkScheme';

const LIGHT_THEME_OPTIONS = getThemesByMode('light');
const DARK_THEME_OPTIONS = getThemesByMode('dark');

const ThemeContext = createContext<ThemeContextValue | undefined>(undefined);

function getStoredValue<T extends string>(key: string, fallback: T): T {
  if (typeof window === 'undefined') return fallback;
  const stored = localStorage.getItem(key);
  if (!stored) return fallback;
  return stored as T;
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [modePreference, setModePreferenceState] = useState<ThemeModePreference>(() =>
    getStoredValue<ThemeModePreference>(MODE_KEY, 'system'),
  );

  const [lightSchemeId, setLightSchemeIdState] = useState<string>(() =>
    getStoredValue<string>(LIGHT_SCHEME_KEY, DEFAULT_LIGHT_THEME_ID),
  );

  const [darkSchemeId, setDarkSchemeIdState] = useState<string>(() =>
    getStoredValue<string>(DARK_SCHEME_KEY, DEFAULT_DARK_THEME_ID),
  );

  const [systemPrefersDark, setSystemPrefersDark] = useState<boolean>(() => {
    if (typeof window === 'undefined') return false;
    return window.matchMedia('(prefers-color-scheme: dark)').matches;
  });

  useEffect(() => {
    if (typeof window === 'undefined') return;
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = (event: MediaQueryListEvent) => setSystemPrefersDark(event.matches);
    mediaQuery.addEventListener('change', handler);
    return () => mediaQuery.removeEventListener('change', handler);
  }, []);

  const resolvedMode: 'light' | 'dark' = useMemo(() => {
    if (modePreference === 'system') {
      return systemPrefersDark ? 'dark' : 'light';
    }
    return modePreference;
  }, [modePreference, systemPrefersDark]);

  const activeTokens = useMemo(() => {
    const activeId = resolvedMode === 'dark' ? darkSchemeId : lightSchemeId;
    return resolveTheme(activeId, resolvedMode);
  }, [darkSchemeId, lightSchemeId, resolvedMode]);

  useEffect(() => {
    applyThemeTokens(activeTokens);
  }, [activeTokens]);

  const setModePreference = (mode: ThemeModePreference) => {
    setModePreferenceState(mode);
    if (typeof window !== 'undefined') {
      localStorage.setItem(MODE_KEY, mode);
    }
  };

  const setLightScheme = (schemeId: string) => {
    const theme = getTheme(schemeId);
    if (!theme || theme.mode !== 'light') return;
    setLightSchemeIdState(theme.id);
    if (typeof window !== 'undefined') {
      localStorage.setItem(LIGHT_SCHEME_KEY, theme.id);
    }
  };

  const setDarkScheme = (schemeId: string) => {
    const theme = getTheme(schemeId);
    if (!theme || theme.mode !== 'dark') return;
    setDarkSchemeIdState(theme.id);
    if (typeof window !== 'undefined') {
      localStorage.setItem(DARK_SCHEME_KEY, theme.id);
    }
  };

  useEffect(() => {
    const theme = getTheme(lightSchemeId);
    if (!theme || theme.mode !== 'light') {
      setLightScheme(DEFAULT_LIGHT_THEME_ID);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    const theme = getTheme(darkSchemeId);
    if (!theme || theme.mode !== 'dark') {
      setDarkScheme(DEFAULT_DARK_THEME_ID);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const resetDefaults = () => {
    setLightScheme(DEFAULT_LIGHT_THEME_ID);
    setDarkScheme(DEFAULT_DARK_THEME_ID);
    setModePreference('system');
  };

  const value: ThemeContextValue = {
    modePreference,
    resolvedMode,
    lightSchemeId,
    darkSchemeId,
    availableLightThemes: LIGHT_THEME_OPTIONS,
    availableDarkThemes: DARK_THEME_OPTIONS,
    effectiveTokens: activeTokens,
    setModePreference,
    setLightScheme,
    setDarkScheme,
    resetDefaults,
  };

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

export function useTheme() {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
}
