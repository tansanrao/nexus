# Frontend Theming Refactor Plan

## Objectives

- Treat color, spacing, and typography as data so any terminal scheme (Catppuccin, Solarized, Gruvbox, Dracula, Rose Pine, etc.) can be applied without touching component code.
- Keep the interface information-dense, text-first, and IDE-like: neutral surfaces, vivid accents for hierarchy, consistent typographic scale.
- Allow users to independently pick a light-theme palette and a dark-theme palette, or lock the UI to a single mode.
- Provide a clear contribution path for adding schemes and extending semantic tokens.

## Target Architecture

### 1. Theme Tokens

Create a `ThemeTokens` TypeScript interface that captures everything the UI needs:

```ts
interface ThemeTokens {
  id: string;                // e.g. "catppuccin-latte"
  label: string;             // human-friendly name
  mode: 'light' | 'dark';
  colors: {
    surface: {
      base: string;          // base background
      raised: string;        // card / panel
      muted: string;         // subtle accent
      border: string;
      overlay: string;       // popovers, menus
    };
    text: {
      primary: string;
      secondary: string;
      muted: string;
      accent: string;        // e.g. for links
      negative: string;
    };
    accent: {
      primary: string;       // brand highlight
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
    chart: string[];         // small palette for visualizations
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
```

Store palettes as JSON in `src/theme/palettes/<scheme>.json`. If a theme has both light and dark variants (e.g. Catppuccin Latte/Mocha) each file describes one variant.

### 2. Theme Registry & Loader

- `src/theme/registry.ts` exports a `ThemeRegistry` that imports all palette JSON, validates them against `ThemeTokens`, and exposes:
  - `getThemesByMode(mode: 'light' | 'dark')`
  - `getTokens(id: string)`
  - `listSchemes(): ThemeSummary[]` for picker UI.
- Validation uses `zod` (or native) to ensure required fields exist and colors are valid hex/HSL.
- New palettes can be added by dropping a JSON file and registering it in the registry index.

### 3. CSS Variable Generator

- `src/theme/css.ts` exposes `generateCssVariables(tokens: ThemeTokens)` which returns a string of custom properties (e.g. `--surface-base`, `--text-muted`, `--accent-primary`, `--syntax-keyword`).
- Attach variables to `.theme-${tokens.id}` class on `document.documentElement` and to `body`. All components read from these variables.
- Maintain semantic aliases for utility classes (`.surface`, `.surface-muted`, `.text-meta`, etc.) in `src/styles/theme.css`, but **they only map variables**, never literal color values.

### 4. Theme Context & Hooks

- Extend `ThemeContext` to hold:
  ```ts
  interface ThemeState {
    modePreference: 'system' | 'light' | 'dark';
    lightSchemeId: string;
    darkSchemeId: string;
    effectiveTokens: ThemeTokens;
  }
  ```
- Expose hooks:
  - `useThemeTokens()` → returns `effectiveTokens` for components needing raw values.
  - `useThemeActions()` → setters for mode, light scheme, dark scheme.
- The provider listens to system preference changes only when `modePreference === 'system'`.

### 5. Component Styling Guidelines

1. **No hard-coded colors.** Use either CSS utilities (`className="text-accent"`) or `useThemeTokens()` for inline styles (e.g. canvas rendering).
2. **Typographic scale utilities**: `text-meta`, `text-body`, `text-heading`, `text-display` map to `tokens.typography.scale`. Update `Section`, navigation, and headings to use these classes.
3. **Surface utilities**: `.surface`, `.surface-emphasis`, `.surface-muted`, `.surface-overlay`, `.surface-border`.
4. **State badges**: `.badge-info`, `.badge-success`, etc, map to `tokens.colors.states`.
5. **Syntax Highlighting**: Create a shared `CodeBlock` component that uses the syntax tokens for inline/blocks.
6. **Charts & Metrics**: Use `tokens.colors.chart` (cyclical) for graphs.
7. **Focus Rings**: derive from `tokens.colors.accent.primary` for consistency.

### 6. Layout & Information Density

- Introduce stack utilities (`stack-sm`, `stack-md`) that apply consistent vertical rhythm using `tokens.spacing`.
- Default baseline: `line-height` from tokens; ensure base font size 14–15px for dense text.
- Navigation, lists, and detail panes adopt `.text-meta` for meta info, `.text-body` for primary content, `.text-heading` for row titles to mimic IDE panels.

### 7. Theme Picker UX

- Provide two dropdowns in settings:
  - **Light Mode Theme** (options: tokens where `mode === 'light'`)
  - **Dark Mode Theme** (options: tokens where `mode === 'dark'`)
  - **Mode Preference**: Light / Dark / System (system toggles automatically between the two selected schemes).
- Show miniature swatch preview using accent & surface colors to convey character of each scheme.

### 8. Implementation Phases

1. **Foundation**
   - Implement `ThemeTokens`, registry, CSS generator, and context updates.
   - Migrate existing Catppuccin & Solarized palettes into the new JSON format.
2. **Styling Primitives**
   - Refactor `index.css` into `theme.css` with utilities referencing tokens.
   - Update primitive components (`CompactButton`, `Section`, inputs, select, badges, navigation links).
3. **Application Sweep**
   - Replace lingering hard-coded colors in pages/components.
   - Verify dropdowns, modals, and Radix portals inherit variables (update `SelectContent`, `Dialog`, etc).
4. **Typography & Layout**
   - Apply new class names for headings/meta text across settings, nav, thread view.
5. **Extended Palettes**
   - Add sample palettes selectively; start with Catppuccin and Solarized as built-ins.
6. **Cleanup**
   - Remove legacy theme constants from `index.css`.
   - Update tests/linters to enforce semantic class usage.

### 9. Adding a New Theme

1. Create `src/theme/palettes/<scheme>.ts` that exports a `ThemeTokens[]`. You can copy `catppuccin.ts` or `solarized.ts` as a starting point.
2. Export it in `src/theme/registry.ts`.
3. Optional: add a curated chart palette if the theme has limited colors.
4. Run `npm run build` to confirm validation passes.

### 10. Migration Checklist

- [ ] Registry + context updated and wired into `main.tsx`.
- [ ] `index.css` replaced by token-driven `theme.css`.
- [ ] All components updated to use semantic classes or `useThemeTokens`.
- [ ] Settings theme picker uses new registry APIs.
- [ ] Documentation (this file) kept alongside theme JSON for contributors.
- [ ] Manual verification with at least Catppuccin (Latte/Mocha) and Solarized (Light/Dark).

## Summary

This refactor moves the frontend from ad-hoc theme variables to a robust, data-driven engine. By normalizing tokens and centralizing CSS generation, we can plug in any terminal color scheme and instantly get a colorful, IDE-like, information-dense interface with minimal effort. Adding new palettes becomes a matter of shipping a JSON file—components simply follow the semantic language provided by `ThemeTokens`.
