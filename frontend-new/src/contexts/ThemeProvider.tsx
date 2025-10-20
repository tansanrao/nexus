import { useEffect, useState, type ComponentProps, type ReactNode } from 'react';
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

const isThemeKey = (value: string | undefined): value is ThemeKey => {
  if (!value) return false;
  return Object.prototype.hasOwnProperty.call(themeSettings, value);
};

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
      <RadixThemeBridge>{children}</RadixThemeBridge>
    </NextThemesProvider>
  );
}
