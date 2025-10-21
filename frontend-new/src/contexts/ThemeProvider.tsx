import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from 'react';
import { Theme as RadixTheme } from '@radix-ui/themes';
import {
  DEFAULT_DARK_THEME_ID,
  DEFAULT_LIGHT_THEME_ID,
  DARK_THEME_OPTIONS,
  LIGHT_THEME_OPTIONS,
  themeClassNames,
  themePresets,
} from '../theme/presets';
import {
  ThemeSettingsContext,
  type ThemeSettingsContextValue,
  type ThemeModePreference,
} from './theme-settings-context';
import type { ThemeAppearance, ThemeId } from '../theme/presets';

const MODE_STORAGE_KEY = 'nexus.theme.modePreference';
const LIGHT_SCHEME_KEY = 'nexus.theme.lightScheme';
const DARK_SCHEME_KEY = 'nexus.theme.darkScheme';

const themeClassSet = new Set<ThemeId>(themeClassNames);
const useIsomorphicLayoutEffect = typeof window !== 'undefined' ? useLayoutEffect : useEffect;

const isThemeId = (value: string | null | undefined): value is ThemeId => {
  if (!value) return false;
  return themeClassSet.has(value as ThemeId);
};

const getStoredModePreference = (): ThemeModePreference => {
  if (typeof window === 'undefined') return 'system';
  const stored = window.localStorage.getItem(MODE_STORAGE_KEY);
  return stored === 'light' || stored === 'dark' || stored === 'system' ? stored : 'system';
};

const getStoredScheme = (storageKey: string, fallback: ThemeId, appearance: ThemeAppearance) => {
  if (typeof window === 'undefined') return fallback;
  const stored = window.localStorage.getItem(storageKey);
  if (!isThemeId(stored)) return fallback;
  return themePresets[stored].appearance === appearance ? stored : fallback;
};

const getSystemAppearance = (): ThemeAppearance => {
  if (typeof window === 'undefined') return 'light';
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
};

const applyThemeToDocument = (themeId: ThemeId, appearance: ThemeAppearance) => {
  if (typeof document === 'undefined') return;

  const root = document.documentElement;
  root.classList.remove(...themeClassNames);
  root.classList.remove('light', 'dark');
  root.classList.add(themeId);
  root.classList.add(appearance);
  root.dataset.theme = themeId;
  root.dataset.appearance = appearance;
  root.style.colorScheme = appearance;

  const preset = themePresets[themeId];
  if (!preset) return;

  const { tokens } = preset;
  Object.entries(tokens).forEach(([token, value]) => {
    root.style.setProperty(token, value);
  });
};

export function ThemeProvider({ children }: { children: ReactNode }) {
  return <ThemeSettingsProvider>{children}</ThemeSettingsProvider>;
}

function ThemeSettingsProvider({ children }: { children: ReactNode }) {
  const [modePreference, setModePreferenceState] = useState<ThemeModePreference>(
    getStoredModePreference
  );

  const [lightSchemeId, setLightSchemeIdState] = useState<ThemeId>(() =>
    getStoredScheme(LIGHT_SCHEME_KEY, DEFAULT_LIGHT_THEME_ID, 'light')
  );

  const [darkSchemeId, setDarkSchemeIdState] = useState<ThemeId>(() =>
    getStoredScheme(DARK_SCHEME_KEY, DEFAULT_DARK_THEME_ID, 'dark')
  );

  const [systemAppearance, setSystemAppearance] = useState<ThemeAppearance>(getSystemAppearance);

  useEffect(() => {
    if (typeof window === 'undefined') return;
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const listener = (event: MediaQueryListEvent) => {
      setSystemAppearance(event.matches ? 'dark' : 'light');
    };

    setSystemAppearance(mediaQuery.matches ? 'dark' : 'light');
    mediaQuery.addEventListener('change', listener);
    return () => mediaQuery.removeEventListener('change', listener);
  }, []);

  const resolvedMode: ThemeAppearance = useMemo(() => {
    return modePreference === 'system' ? systemAppearance : modePreference;
  }, [modePreference, systemAppearance]);

  const resolvedThemeId = resolvedMode === 'dark' ? darkSchemeId : lightSchemeId;
  const activeTheme = useMemo(() => themePresets[resolvedThemeId], [resolvedThemeId]);
  const previousThemeRef = useRef<{ themeId: ThemeId; appearance: ThemeAppearance } | null>(null);

  useIsomorphicLayoutEffect(() => {
    const previous = previousThemeRef.current;
    if (
      previous &&
      previous.themeId === resolvedThemeId &&
      previous.appearance === resolvedMode
    ) {
      return;
    }

    applyThemeToDocument(resolvedThemeId, resolvedMode);
    previousThemeRef.current = { themeId: resolvedThemeId, appearance: resolvedMode };
  }, [resolvedMode, resolvedThemeId]);

  const setModePreference = useCallback((mode: ThemeModePreference) => {
    setModePreferenceState((current) => {
      if (current === mode) {
        return current;
      }
      if (typeof window !== 'undefined') {
        window.localStorage.setItem(MODE_STORAGE_KEY, mode);
      }
      return mode;
    });
  }, []);

  const setLightScheme = useCallback(
    (schemeId: string) => {
      if (!isThemeId(schemeId)) return;
      const preset = themePresets[schemeId];
      if (preset.appearance !== 'light') return;
      setLightSchemeIdState((current) => {
        if (current === preset.id) {
          return current;
        }
        if (typeof window !== 'undefined') {
          window.localStorage.setItem(LIGHT_SCHEME_KEY, preset.id);
        }
        return preset.id;
      });
    },
    [setLightSchemeIdState]
  );

  const setDarkScheme = useCallback(
    (schemeId: string) => {
      if (!isThemeId(schemeId)) return;
      const preset = themePresets[schemeId];
      if (preset.appearance !== 'dark') return;
      setDarkSchemeIdState((current) => {
        if (current === preset.id) {
          return current;
        }
        if (typeof window !== 'undefined') {
          window.localStorage.setItem(DARK_SCHEME_KEY, preset.id);
        }
        return preset.id;
      });
    },
    [setDarkSchemeIdState]
  );

  const resetDefaults = useCallback(() => {
    setLightScheme(DEFAULT_LIGHT_THEME_ID);
    setDarkScheme(DEFAULT_DARK_THEME_ID);
    setModePreference('system');
  }, [setDarkScheme, setLightScheme, setModePreference]);

  const contextValue = useMemo<ThemeSettingsContextValue>(
    () => ({
      modePreference,
      resolvedMode,
      resolvedThemeId,
      lightSchemeId,
      darkSchemeId,
      availableLightThemes: LIGHT_THEME_OPTIONS,
      availableDarkThemes: DARK_THEME_OPTIONS,
      setModePreference,
      setLightScheme,
      setDarkScheme,
      resetDefaults,
    }),
    [
      darkSchemeId,
      lightSchemeId,
      modePreference,
      resetDefaults,
      resolvedMode,
      resolvedThemeId,
      setDarkScheme,
      setLightScheme,
      setModePreference,
    ]
  );

  return (
    <ThemeSettingsContext.Provider value={contextValue}>
      <RadixTheme
        appearance={activeTheme.appearance}
        accentColor={activeTheme.radix.accentColor}
        grayColor={activeTheme.radix.grayColor}
        panelBackground={activeTheme.radix.panelBackground}
        radius="large"
        scaling="100%"
      >
        {children}
      </RadixTheme>
    </ThemeSettingsContext.Provider>
  );
}
