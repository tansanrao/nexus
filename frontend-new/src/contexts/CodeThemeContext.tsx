import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react';
import { highlightAgent } from '../lib/shiki';

type CodeThemeContextValue = {
  availableThemes: string[];
  codeTheme: string;
  setCodeTheme: (theme: string) => void;
};

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

const CodeThemeContext = createContext<CodeThemeContextValue | undefined>(undefined);

export function CodeThemeProvider({ children }: { children: ReactNode }) {
  const availableThemes = useMemo(() => [...POPULAR_THEMES], []);
  const [codeTheme, setCodeThemeState] = useState<string>(() => {
    return localStorage.getItem(STORAGE_KEY) || DEFAULT_THEME;
  });

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

  const setCodeTheme = (theme: string) => {
    setCodeThemeState(theme);
  };

  return (
    <CodeThemeContext.Provider value={{ availableThemes, codeTheme, setCodeTheme }}>
      {children}
    </CodeThemeContext.Provider>
  );
}

export function useCodeTheme(): CodeThemeContextValue {
  const context = useContext(CodeThemeContext);
  if (!context) {
    throw new Error('useCodeTheme must be used within a CodeThemeProvider');
  }

  return context;
}
