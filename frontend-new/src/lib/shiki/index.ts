import { HighlightAgent } from './highlight-agent';

export const highlightAgent = new HighlightAgent({
  initialTheme: 'github-light',
  preloadThemes: ['github-dark'],
});

export const getHighlightThemeVersion = () => highlightAgent.getThemeVersion();
