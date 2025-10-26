import { createContext, useContext } from 'react';

export interface CodeThemeContextValue {
  availableThemes: string[];
  codeTheme: string;
  setCodeTheme: (theme: string) => void;
}

export const CodeThemeContext = createContext<CodeThemeContextValue | undefined>(undefined);

export function useCodeTheme(): CodeThemeContextValue {
  const context = useContext(CodeThemeContext);
  if (!context) {
    throw new Error('useCodeTheme must be used within a CodeThemeProvider');
  }

  return context;
}
