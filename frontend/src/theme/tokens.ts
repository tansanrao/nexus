export type ThemeMode = 'light' | 'dark';

export interface ThemeTokens {
  id: string;
  label: string;
  mode: ThemeMode;
  colors: {
    surface: {
      base: string;
      raised: string;
      muted: string;
      border: string;
      overlay: string;
    };
    text: {
      primary: string;
      secondary: string;
      muted: string;
      accent: string;
      negative: string;
    };
    accent: {
      primary: string;
      secondary: string;
      tertiary: string;
    };
    states: {
      info: string;
      success: string;
      warning: string;
      danger: string;
    };
    syntax: {
      keyword: string;
      string: string;
      number: string;
      comment: string;
      background: string;
    };
    chart: string[];
  };
  typography: {
    fontFamily: string;
    fontMono: string;
    lineHeight: number;
    scale: {
      xs: string;
      sm: string;
      base: string;
      lg: string;
      xl: string;
      display: string;
    };
  };
  spacing: {
    gapXs: string;
    gapSm: string;
    gapMd: string;
    gapLg: string;
  };
}

export interface ThemeSummary {
  id: string;
  label: string;
  mode: ThemeMode;
  accent: string;
  surface: string;
}
