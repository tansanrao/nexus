import { createHighlighter, createJavaScriptRegexEngine } from 'shiki';
import type { Highlighter } from 'shiki';

type HighlightOptions = {
  code: string;
  lang: string;
};

type HighlightAgentOptions = {
  initialTheme?: string;
  preloadThemes?: string[];
};

type ThemeChangeListener = () => void;

const DEFAULT_LANGS = [
  'diff',
  'bash',
  'shell',
  'typescript',
  'tsx',
  'javascript',
  'jsx',
  'json',
  'css',
  'scss',
  'less',
  'html',
  'xml',
  'markdown',
  'mdx',
  'yaml',
  'toml',
  'python',
  'go',
  'rust',
  'java',
  'c',
  'cpp',
  'csharp',
  'php',
  'ruby',
];

const LANGUAGE_ALIASES: Record<string, string> = {
  shell: 'bash',
  sh: 'bash',
  zsh: 'bash',
  console: 'bash',
  plaintext: 'plaintext',
  text: 'plaintext',
  txt: 'plaintext',
  cjs: 'javascript',
  mjs: 'javascript',
  js: 'javascript',
  jsx: 'jsx',
  ts: 'typescript',
  tsx: 'tsx',
  yml: 'yaml',
};

function normalizeLanguage(lang: string | null | undefined): string {
  const lower = (lang || '').toLowerCase();
  if (!lower) return 'plaintext';
  return LANGUAGE_ALIASES[lower] ?? lower;
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

function dedent(value: string): string {
  const lines = value.replace(/^\n+|\n+$/g, '').split('\n');
  const indents = lines
    .filter((line) => line.trim().length > 0)
    .map((line) => line.match(/^\s*/)?.[0].length ?? 0);
  const minIndent = indents.length > 0 ? Math.min(...indents) : 0;
  if (minIndent === 0) {
    return lines.join('\n');
  }
  return lines.map((line) => line.slice(minIndent)).join('\n');
}

export class HighlightAgent {
  readonly ready: Promise<void>;

  private highlighter!: Highlighter;
  private currentTheme!: string;
  private readonly cache = new Map<string, string>();
  private readonly listeners = new Set<ThemeChangeListener>();
  private version = 0;

  constructor(opts: HighlightAgentOptions = {}) {
    const initialTheme = opts.initialTheme ?? 'github-light';
    this.currentTheme = initialTheme;

    this.ready = (async () => {
      const engine = await createJavaScriptRegexEngine();
      const themeSet = new Set<string>([
        initialTheme,
        ...(opts.preloadThemes ?? []),
      ]);

      this.highlighter = await createHighlighter({
        themes: Array.from(themeSet),
        langs: DEFAULT_LANGS,
        engine,
      });

      await this.highlighter.setTheme(initialTheme as never);
      await this.applyThemeCSS();
    })();
  }

  subscribe(listener: ThemeChangeListener): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  getThemeVersion(): number {
    return this.version;
  }

  getCurrentTheme(): string {
    return this.currentTheme;
  }

  async setTheme(themeName: string): Promise<void> {
    await this.ready;
    if (!themeName || themeName === this.currentTheme) {
      return;
    }

    try {
      await this.highlighter.loadTheme(themeName as never);
    } catch (error) {
      console.warn(`Unable to load Shiki theme "${themeName}"`, error);
      return;
    }

    await this.highlighter.setTheme(themeName as never);
    this.currentTheme = themeName;
    this.cache.clear();
    this.version += 1;
    await this.applyThemeCSS();
    this.emitThemeChange();
  }

  async addLanguage(lang: string): Promise<void> {
    await this.ready;
    const normalized = normalizeLanguage(lang);
    const loaded = this.highlighter.getLoadedLanguages();
    if (!loaded.includes(normalized)) {
      try {
        await this.highlighter.loadLanguage(normalized as never);
      } catch {
        // Ignore failures; Shiki will fallback gracefully.
      }
    }
  }

  async highlight({ code, lang }: HighlightOptions): Promise<string> {
    await this.ready;
    const normalized = normalizeLanguage(lang);
    const cacheKey = this.makeCacheKey('block', normalized, code);
    const cached = this.cache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const safeCode = code ?? '';
    if (!this.highlighter.getLoadedLanguages().includes(normalized)) {
      await this.addLanguage(normalized);
    }

    let html: string;
    try {
      html = await this.highlighter.codeToHtml(safeCode, {
        lang: normalized as never,
        theme: this.currentTheme as never,
      });
    } catch {
      html = `<pre class="shiki"><code>${escapeHtml(safeCode)}</code></pre>`;
    }

    this.cache.set(cacheKey, html);
    return html;
  }

  async highlightInline({ code, lang }: HighlightOptions): Promise<string> {
    const cacheKey = this.makeCacheKey('inline', normalizeLanguage(lang), code);
    const cached = this.cache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const blockHtml = await this.highlight({ code, lang });
    const inlineHtml = this.extractInlineHtml(blockHtml, code);
    this.cache.set(cacheKey, inlineHtml);
    return inlineHtml;
  }

  async highlightAll(selector = 'pre > code'): Promise<void> {
    if (typeof document === 'undefined') {
      return;
    }

    await this.ready;
    const nodes = Array.from(document.querySelectorAll<HTMLElement>(selector));
    if (!nodes.length) {
      return;
    }

    for (const node of nodes) {
      const classMatch = node.className.match(/language-([\w-]+)/i);
      const lang = classMatch?.[1] ?? 'plaintext';
      const code = dedent(node.textContent ?? '');
      const html = await this.highlight({ code, lang });
      const pre = node.closest('pre');

      if (!pre) {
        node.outerHTML = html;
        continue;
      }

      const wrapper = document.createElement('div');
      wrapper.innerHTML = html;
      const newPre = wrapper.firstElementChild as HTMLElement | null;

      if (newPre) {
        pre.replaceWith(newPre);
        this.injectCopyButton(newPre, code);
      }
    }
  }

  private makeCacheKey(kind: 'block' | 'inline', lang: string, code: string): string {
    return `${kind}::${this.currentTheme}::${lang}::${code}`;
  }

  private extractInlineHtml(blockHtml: string, originalCode: string): string {
    const match = blockHtml.match(/<code[^>]*>([\s\S]*?)<\/code>/i);
    if (!match) {
      return escapeHtml(originalCode ?? '');
    }

    const inner = match[1];
    // Re-wrap lines so they can flow inline when needed.
    return `<span class="shiki-inline-root">${inner}</span>`;
  }

  private async applyThemeCSS(): Promise<void> {
    if (typeof document === 'undefined' || !this.highlighter) {
      return;
    }

    const root = document.documentElement;
    const getTheme = this.highlighter.getTheme?.bind(this.highlighter);
    if (!getTheme) {
      return;
    }

    try {
      const theme = await getTheme(this.currentTheme);
      const colors = theme?.colors ?? {};
      Object.entries(colors).forEach(([key, value]) => {
        if (typeof value === 'string') {
          root.style.setProperty(`--shiki-${key.replace(/\./g, '-')}`, value);
        }
      });
    } catch {
      // Ignore theme resolution failures; highlighting will still work.
    }
  }

  private emitThemeChange(): void {
    for (const listener of this.listeners) {
      listener();
    }
  }

  private injectCopyButton(pre: HTMLElement, code: string): void {
    if (typeof navigator === 'undefined' || typeof navigator.clipboard === 'undefined') {
      return;
    }

    const existing = pre.querySelector(':scope > button.copy');
    if (existing) {
      return;
    }

    const button = document.createElement('button');
    button.className = 'copy';
    button.type = 'button';
    button.textContent = 'Copy';
    button.addEventListener('click', async () => {
      try {
        await navigator.clipboard.writeText(code);
        button.textContent = 'Copied!';
      } catch {
        button.textContent = 'Error';
      }
      setTimeout(() => {
        button.textContent = 'Copy';
      }, 1200);
    });

    pre.appendChild(button);
  }
}
