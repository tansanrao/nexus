import { baseSpacing, baseTypography } from '../common';
import type { ThemeTokens } from '../tokens';

const catppuccin: ThemeTokens[] = [
  {
    id: 'catppuccin-latte',
    label: 'Catppuccin Latte',
    mode: 'light',
    colors: {
      surface: {
        base: '#eff1f5',
        raised: '#eef2f9',
        muted: '#e4e8f1',
        border: '#d2d6e3',
        overlay: '#f6f8fc',
      },
      text: {
        primary: '#4c4f69',
        secondary: '#5c5f77',
        muted: '#7c7f93',
        accent: '#1e66f5',
        negative: '#d20f39',
      },
      accent: {
        primary: '#1e66f5',
        secondary: '#8839ef',
        tertiary: '#40a02b',
      },
      states: {
        info: '#209fb5',
        success: '#40a02b',
        warning: '#df8e1d',
        danger: '#d20f39',
      },
      syntax: {
        keyword: '#5c5f77',
        string: '#40a02b',
        number: '#fe640b',
        comment: '#9ca0b0',
        background: '#eef2f9',
      },
      chart: ['#1e66f5', '#209fb5', '#40a02b', '#df8e1d', '#d20f39', '#8839ef'],
    },
    typography: baseTypography,
    spacing: baseSpacing,
  },
  {
    id: 'catppuccin-mocha',
    label: 'Catppuccin Mocha',
    mode: 'dark',
    colors: {
      surface: {
        base: '#1e1e2e',
        raised: '#232437',
        muted: '#2d2f43',
        border: '#3a3d55',
        overlay: '#2a2b3e',
      },
      text: {
        primary: '#cdd6f4',
        secondary: '#bac2de',
        muted: '#a6adc8',
        accent: '#89b4fa',
        negative: '#f38ba8',
      },
      accent: {
        primary: '#89b4fa',
        secondary: '#cba6f7',
        tertiary: '#94e2d5',
      },
      states: {
        info: '#89dceb',
        success: '#a6e3a1',
        warning: '#f9e2af',
        danger: '#f38ba8',
      },
      syntax: {
        keyword: '#cba6f7',
        string: '#a6e3a1',
        number: '#f38ba8',
        comment: '#6c7086',
        background: '#232437',
      },
      chart: ['#89b4fa', '#cba6f7', '#94e2d5', '#f9e2af', '#f38ba8', '#fab387'],
    },
    typography: baseTypography,
    spacing: baseSpacing,
  },
];

export default catppuccin;
