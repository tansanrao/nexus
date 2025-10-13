import { createContext, useContext, useState, useEffect, type ReactNode } from 'react';

type ThemeMode = 'light' | 'dark' | 'system';
type LightTheme = 'catppuccin-latte' | 'solarized-light';
type DarkTheme = 'catppuccin-mocha' | 'solarized-dark';

interface ThemeContextType {
  themeMode: ThemeMode;
  lightTheme: LightTheme;
  darkTheme: DarkTheme;
  setThemeMode: (mode: ThemeMode) => void;
  setLightTheme: (theme: LightTheme) => void;
  setDarkTheme: (theme: DarkTheme) => void;
  effectiveTheme: string; // The actual theme class being applied
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

const THEME_MODE_KEY = 'themeMode';
const LIGHT_THEME_KEY = 'lightTheme';
const DARK_THEME_KEY = 'darkTheme';

export function ThemeProvider({ children }: { children: ReactNode }) {
  // Initialize from localStorage or defaults
  const [themeMode, setThemeModeState] = useState<ThemeMode>(() => {
    const saved = localStorage.getItem(THEME_MODE_KEY);
    return (saved as ThemeMode) || 'system';
  });

  const [lightTheme, setLightThemeState] = useState<LightTheme>(() => {
    const saved = localStorage.getItem(LIGHT_THEME_KEY);
    return (saved as LightTheme) || 'catppuccin-latte';
  });

  const [darkTheme, setDarkThemeState] = useState<DarkTheme>(() => {
    const saved = localStorage.getItem(DARK_THEME_KEY);
    return (saved as DarkTheme) || 'catppuccin-mocha';
  });

  const [systemPrefersDark, setSystemPrefersDark] = useState(() => {
    return window.matchMedia('(prefers-color-scheme: dark)').matches;
  });

  // Listen for system theme changes
  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = (e: MediaQueryListEvent) => {
      setSystemPrefersDark(e.matches);
    };

    mediaQuery.addEventListener('change', handler);
    return () => mediaQuery.removeEventListener('change', handler);
  }, []);

  // Calculate effective theme
  const effectiveTheme = (() => {
    const isDark = themeMode === 'dark' || (themeMode === 'system' && systemPrefersDark);
    return isDark ? `theme-${darkTheme}` : `theme-${lightTheme}`;
  })();

  // Apply theme to document
  useEffect(() => {
    const root = document.documentElement;

    // Remove all theme classes
    root.classList.remove(
      'theme-catppuccin-latte',
      'theme-catppuccin-mocha',
      'theme-solarized-light',
      'theme-solarized-dark'
    );

    // Add the effective theme
    root.classList.add(effectiveTheme);
  }, [effectiveTheme]);

  const setThemeMode = (mode: ThemeMode) => {
    setThemeModeState(mode);
    localStorage.setItem(THEME_MODE_KEY, mode);
  };

  const setLightTheme = (theme: LightTheme) => {
    setLightThemeState(theme);
    localStorage.setItem(LIGHT_THEME_KEY, theme);
  };

  const setDarkTheme = (theme: DarkTheme) => {
    setDarkThemeState(theme);
    localStorage.setItem(DARK_THEME_KEY, theme);
  };

  return (
    <ThemeContext.Provider
      value={{
        themeMode,
        lightTheme,
        darkTheme,
        setThemeMode,
        setLightTheme,
        setDarkTheme,
        effectiveTheme,
      }}
    >
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  const context = useContext(ThemeContext);
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
}
