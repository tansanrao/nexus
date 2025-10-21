import { useCallback, useEffect, useMemo, useState, type ReactNode } from 'react';
import { highlightAgent } from '../lib/shiki';
import {
  CodeThemeContext,
  type CodeThemeContextValue,
} from './code-theme-context';

const DEFAULT_THEME = 'github-light';
const STORAGE_KEY = 'nexus_code_theme';

const POPULAR_THEMES = [
  'github-light',
  'github-dark',
  'github-dark-dimmed',
  'dark-plus',
  'light-plus',
  'dracula',
  'nord',
  'one-dark-pro',
  'one-light',
  'night-owl',
  'rose-pine',
  'rose-pine-moon',
  'kanagawa-wave',
  'everforest-dark',
  'everforest-light',
  'catppuccin-mocha',
  'catppuccin-latte',
  'gruvbox-dark-medium',
  'gruvbox-light-soft',
  'vitesse-dark',
  'vitesse-light',
  'material-theme',
  'material-theme-darker',
  'material-theme-ocean',
  'material-theme-palenight',
  'monokai',
  'monokai-light',
  'tokyo-night',
  'tokyo-night-light',
  'tokyo-night-storm',
  'min-dark',
  'min-light',
  'solarized-dark',
  'solarized-light',
] as const;

export function CodeThemeProvider({ children }: { children: ReactNode }) {
  const availableThemes = useMemo(() => [...POPULAR_THEMES], []);
  const [codeTheme, setCodeThemeState] = useState<string>(() => {
    return localStorage.getItem(STORAGE_KEY) || DEFAULT_THEME;
  });

  const setCodeTheme = useCallback((theme: string) => {
    setCodeThemeState(theme);
  }, []);

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, codeTheme);
    const applyTheme = async () => {
      try {
        await highlightAgent.setTheme(codeTheme);
      } catch (error) {
        console.error('Failed to apply Shiki theme', error);
      }
    };
    void applyTheme();
  }, [codeTheme]);

  const contextValue = useMemo<CodeThemeContextValue>(
    () => ({
      availableThemes,
      codeTheme,
      setCodeTheme,
    }),
    [availableThemes, codeTheme, setCodeTheme]
  );

  return (
    <CodeThemeContext.Provider value={contextValue}>
      {children}
    </CodeThemeContext.Provider>
  );
}
