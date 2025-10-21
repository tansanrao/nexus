import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ComponentProps,
  type ReactNode,
} from 'react';
import { Theme as RadixTheme } from '@radix-ui/themes';
import { ThemeProvider as NextThemesProvider, useTheme } from 'next-themes';
import type { ThemeProviderProps } from 'next-themes';

type ThemeAppearance = 'light' | 'dark';
type RadixAccent = NonNullable<ComponentProps<typeof RadixTheme>['accentColor']>;
type RadixGray = NonNullable<ComponentProps<typeof RadixTheme>['grayColor']>;

type ThemeSettings = {
  appearance: ThemeAppearance;
  accentColor: RadixAccent;
  grayColor: RadixGray;
};

const themeSettings = {
  light: { appearance: 'light', accentColor: 'iris', grayColor: 'sand' },
  dark: { appearance: 'dark', accentColor: 'violet', grayColor: 'slate' },
  'solarized-light': { appearance: 'light', accentColor: 'cyan', grayColor: 'sage' },
  'solarized-dark': { appearance: 'dark', accentColor: 'cyan', grayColor: 'olive' },
} as const satisfies Record<string, ThemeSettings>;

type ThemeKey = keyof typeof themeSettings;
const DEFAULT_THEME_KEY: ThemeKey = 'light';

const isThemeKey = (value: string | null | undefined): value is ThemeKey => {
  if (!value) return false;
  return Object.prototype.hasOwnProperty.call(themeSettings, value);
};

type ThemeModePreference = 'light' | 'dark' | 'system';

interface ThemeSettingsContextValue {
  modePreference: ThemeModePreference;
  resolvedMode: 'light' | 'dark';
  lightSchemeId: ThemeKey;
  darkSchemeId: ThemeKey;
  availableLightThemes: Array<{ id: ThemeKey; label: string }>;
  availableDarkThemes: Array<{ id: ThemeKey; label: string }>;
  setModePreference: (mode: ThemeModePreference) => void;
  setLightScheme: (schemeId: string) => void;
  setDarkScheme: (schemeId: string) => void;
  resetDefaults: () => void;
}

const MODE_STORAGE_KEY = 'nexus.theme.modePreference';
const LIGHT_SCHEME_KEY = 'nexus.theme.lightScheme';
const DARK_SCHEME_KEY = 'nexus.theme.darkScheme';

const LIGHT_THEME_OPTIONS: Array<{ id: ThemeKey; label: string }> = [
  { id: 'light', label: 'Default Light' },
  { id: 'solarized-light', label: 'Solarized Light' },
];

const DARK_THEME_OPTIONS: Array<{ id: ThemeKey; label: string }> = [
  { id: 'dark', label: 'Default Dark' },
  { id: 'solarized-dark', label: 'Solarized Dark' },
];

const ThemeSettingsContext = createContext<ThemeSettingsContextValue | undefined>(undefined);

function RadixThemeBridge({ children }: { children: ReactNode }) {
  const { resolvedTheme, theme } = useTheme();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  const activeThemeName = mounted ? resolvedTheme ?? theme : undefined;
  const activeThemeKey = isThemeKey(activeThemeName) ? activeThemeName : DEFAULT_THEME_KEY;
  const activeTheme = themeSettings[activeThemeKey];

  return (
    <RadixTheme
      appearance={activeTheme.appearance}
      accentColor={activeTheme.accentColor}
      grayColor={activeTheme.grayColor}
      panelBackground="solid"
      radius="large"
      scaling="100%"
    >
      {children}
    </RadixTheme>
  );
}

export function ThemeProvider({ children, ...props }: ThemeProviderProps) {
  return (
    <NextThemesProvider {...props}>
      <ThemeSettingsProvider>
        <RadixThemeBridge>{children}</RadixThemeBridge>
      </ThemeSettingsProvider>
    </NextThemesProvider>
  );
}

function ThemeSettingsProvider({ children }: { children: ReactNode }) {
  const { setTheme, resolvedTheme, theme: currentTheme } = useTheme();

  const [modePreference, setModePreferenceState] = useState<ThemeModePreference>(() => {
    if (typeof window === 'undefined') return 'system';
    const stored = window.localStorage.getItem(MODE_STORAGE_KEY);
    return stored === 'light' || stored === 'dark' || stored === 'system' ? stored : 'system';
  });

  const [lightSchemeId, setLightSchemeIdState] = useState<ThemeKey>(() => {
    if (typeof window === 'undefined') return 'light';
    const stored = window.localStorage.getItem(LIGHT_SCHEME_KEY);
    return isThemeKey(stored) && themeSettings[stored].appearance === 'light' ? stored : 'light';
  });

  const [darkSchemeId, setDarkSchemeIdState] = useState<ThemeKey>(() => {
    if (typeof window === 'undefined') return 'dark';
    const stored = window.localStorage.getItem(DARK_SCHEME_KEY);
    return isThemeKey(stored) && themeSettings[stored].appearance === 'dark' ? stored : 'dark';
  });

  useEffect(() => {
    if (modePreference === 'system') {
      setTheme('system');
      return;
    }

    const targetTheme = modePreference === 'light' ? lightSchemeId : darkSchemeId;
    setTheme(targetTheme);
  }, [modePreference, lightSchemeId, darkSchemeId, setTheme]);

  const effectiveThemeKey = useMemo(() => {
    if (isThemeKey(resolvedTheme)) return resolvedTheme;
    if (isThemeKey(currentTheme)) return currentTheme;
    return modePreference === 'dark' ? darkSchemeId : lightSchemeId;
  }, [resolvedTheme, currentTheme, modePreference, darkSchemeId, lightSchemeId]);

  const resolvedMode = useMemo<'light' | 'dark'>(() => {
    return themeSettings[effectiveThemeKey].appearance;
  }, [effectiveThemeKey]);

  const setModePreference = (mode: ThemeModePreference) => {
    setModePreferenceState(mode);
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(MODE_STORAGE_KEY, mode);
    }
  };

  const setLightScheme = (schemeId: string) => {
    if (!isThemeKey(schemeId) || themeSettings[schemeId]?.appearance !== 'light') return;
    setLightSchemeIdState(schemeId);
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(LIGHT_SCHEME_KEY, schemeId);
    }
  };

  const setDarkScheme = (schemeId: string) => {
    if (!isThemeKey(schemeId) || themeSettings[schemeId]?.appearance !== 'dark') return;
    setDarkSchemeIdState(schemeId);
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(DARK_SCHEME_KEY, schemeId);
    }
  };

  useEffect(() => {
    if (themeSettings[lightSchemeId]?.appearance !== 'light') {
      setLightScheme('light');
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (themeSettings[darkSchemeId]?.appearance !== 'dark') {
      setDarkScheme('dark');
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const resetDefaults = () => {
    setLightScheme('light');
    setDarkScheme('dark');
    setModePreference('system');
  };

  const value: ThemeSettingsContextValue = {
    modePreference,
    resolvedMode,
    lightSchemeId,
    darkSchemeId,
    availableLightThemes: LIGHT_THEME_OPTIONS,
    availableDarkThemes: DARK_THEME_OPTIONS,
    setModePreference,
    setLightScheme,
    setDarkScheme,
    resetDefaults,
  };

  return <ThemeSettingsContext.Provider value={value}>{children}</ThemeSettingsContext.Provider>;
}

export function useThemeSettings() {
  const context = useContext(ThemeSettingsContext);
  if (!context) {
    throw new Error('useThemeSettings must be used within a ThemeProvider');
  }
  return context;
}
