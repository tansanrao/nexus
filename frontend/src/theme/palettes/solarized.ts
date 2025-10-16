import { baseSpacing, baseTypography } from '../common';
import type { ThemeTokens } from '../tokens';

const solarized: ThemeTokens[] = [
  {
    id: 'solarized-light',
    label: 'Solarized Light',
    mode: 'light',
    colors: {
      surface: {
        base: '#fdf6e3',
        raised: '#eee8d5',
        muted: '#e4ddc8',
        border: '#d5cdbc',
        overlay: '#f5efdc',
      },
      text: {
        primary: '#586e75',
        secondary: '#657b83',
        muted: '#93a1a1',
        accent: '#268bd2',
        negative: '#dc322f',
      },
      accent: {
        primary: '#268bd2',
        secondary: '#2aa198',
        tertiary: '#859900',
      },
      states: {
        info: '#2aa198',
        success: '#859900',
        warning: '#b58900',
        danger: '#dc322f',
      },
      syntax: {
        keyword: '#6c71c4',
        string: '#859900',
        number: '#cb4b16',
        comment: '#93a1a1',
        background: '#f5efdc',
      },
      chart: ['#268bd2', '#2aa198', '#859900', '#b58900', '#cb4b16', '#dc322f'],
    },
    typography: baseTypography,
    spacing: baseSpacing,
  },
  {
    id: 'solarized-dark',
    label: 'Solarized Dark',
    mode: 'dark',
    colors: {
      surface: {
        base: '#002b36',
        raised: '#073642',
        muted: '#0a3a4a',
        border: '#0f4a57',
        overlay: '#001f27',
      },
      text: {
        primary: '#eee8d5',
        secondary: '#93a1a1',
        muted: '#839496',
        accent: '#268bd2',
        negative: '#dc322f',
      },
      accent: {
        primary: '#268bd2',
        secondary: '#2aa198',
        tertiary: '#b58900',
      },
      states: {
        info: '#2aa198',
        success: '#859900',
        warning: '#b58900',
        danger: '#dc322f',
      },
      syntax: {
        keyword: '#6c71c4',
        string: '#859900',
        number: '#cb4b16',
        comment: '#586e75',
        background: '#001f27',
      },
      chart: ['#268bd2', '#2aa198', '#859900', '#b58900', '#cb4b16', '#dc322f'],
    },
    typography: baseTypography,
    spacing: baseSpacing,
  },
];

export default solarized;
