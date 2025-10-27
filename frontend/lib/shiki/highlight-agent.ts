"use client"

import {
  createHighlighter,
  createJavaScriptRegexEngine,
  type Highlighter,
} from "shiki"

type HighlightOptions = {
  code: string
  lang: string
}

const DEFAULT_LANGS = [
  "diff",
  "bash",
  "shell",
  "typescript",
  "tsx",
  "javascript",
  "jsx",
  "json",
  "css",
  "scss",
  "less",
  "html",
  "xml",
  "markdown",
  "mdx",
  "yaml",
  "toml",
  "python",
  "go",
  "rust",
  "java",
  "c",
  "cpp",
  "csharp",
  "php",
  "ruby",
]

const LANGUAGE_ALIASES: Record<string, string> = {
  shell: "bash",
  sh: "bash",
  zsh: "bash",
  console: "bash",
  plaintext: "plaintext",
  text: "plaintext",
  txt: "plaintext",
  cjs: "javascript",
  mjs: "javascript",
  js: "javascript",
  jsx: "jsx",
  ts: "typescript",
  tsx: "tsx",
  yml: "yaml",
}

function normalizeLanguage(lang: string | null | undefined): string {
  const lower = (lang || "").toLowerCase()
  if (!lower) return "plaintext"
  return LANGUAGE_ALIASES[lower] ?? lower
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;")
}

type Theme = "github-light" | "github-dark"

export class HighlightAgent {
  readonly ready: Promise<void>

  private highlighter!: Highlighter
  private currentTheme: Theme
  private readonly cache = new Map<string, string>()
  private readonly listeners = new Set<() => void>()
  private version = 0

  constructor(initialTheme: Theme = "github-light") {
    this.currentTheme = initialTheme
    this.ready = this.initialize(initialTheme)
  }

  private async initialize(theme: Theme) {
    const engine = await createJavaScriptRegexEngine()
    this.highlighter = await createHighlighter({
      themes: [theme, theme === "github-light" ? "github-dark" : "github-light"],
      langs: DEFAULT_LANGS,
      engine,
    })
    await this.highlighter.setTheme(theme as never)
  }

  async setTheme(theme: Theme) {
    await this.ready
    if (theme === this.currentTheme) {
      return
    }

    await this.highlighter.loadTheme(theme as never)
    await this.highlighter.setTheme(theme as never)
    this.currentTheme = theme
    this.cache.clear()
    this.version += 1
    this.emitThemeChange()
  }

  async highlight({ code, lang }: HighlightOptions): Promise<string> {
    await this.ready
    const normalized = normalizeLanguage(lang)
    const cacheKey = `${this.currentTheme}:${normalized}:block:${code}`
    const cached = this.cache.get(cacheKey)
    if (cached) {
      return cached
    }

    const safeCode = code ?? ""
    if (!this.highlighter.getLoadedLanguages().includes(normalized)) {
      try {
        await this.highlighter.loadLanguage(normalized as never)
      } catch {
        // ignore, fallback to escaped pre
      }
    }

    let html: string
    try {
      html = await this.highlighter.codeToHtml(safeCode, {
        lang: normalized as never,
        theme: this.currentTheme as never,
      })
    } catch {
      html = `<pre class="shiki"><code>${escapeHtml(safeCode)}</code></pre>`
    }

    this.cache.set(cacheKey, html)
    return html
  }

  async highlightInline({ code, lang }: HighlightOptions): Promise<string> {
    await this.ready
    const normalized = normalizeLanguage(lang)
    const cacheKey = `${this.currentTheme}:${normalized}:inline:${code}`
    const cached = this.cache.get(cacheKey)
    if (cached) {
      return cached
    }

    const blockHtml = await this.highlight({ code, lang })
    const inlineHtml = this.extractInlineHtml(blockHtml, code)
    this.cache.set(cacheKey, inlineHtml)
    return inlineHtml
  }

  subscribe(listener: () => void): () => void {
    this.listeners.add(listener)
    return () => {
      this.listeners.delete(listener)
    }
  }

  getThemeVersion(): number {
    return this.version
  }

  private extractInlineHtml(blockHtml: string, originalCode: string): string {
    const match = blockHtml.match(/<code[^>]*>([\s\S]*?)<\/code>/i)
    if (!match) {
      return `<span class="shiki-inline">${escapeHtml(originalCode ?? "")}</span>`
    }

    const inner = match[1]
    return `<span class="shiki-inline">${inner}</span>`
  }

  private emitThemeChange() {
    for (const listener of this.listeners) {
      listener()
    }
  }
}
