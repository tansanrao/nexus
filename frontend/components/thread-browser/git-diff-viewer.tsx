"use client"

import parseGitDiff, {
  type AnyChunk,
  type AnyFileChange,
  type AnyLineChange,
} from "parse-git-diff"
import {
  IconArrowsExchange,
  IconCheck,
  IconChevronDown,
  IconChevronRight,
  IconCircleDot,
  IconCircleMinus,
  IconCirclePlus,
  IconCopy,
  IconFileDiff,
  IconFileMinus,
  IconFilePlus,
  IconFileText,
  IconGitBranch,
} from "@tabler/icons-react"
import {
  useCallback,
  useEffect,
  useMemo,
  useState,
  useSyncExternalStore,
} from "react"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Separator } from "@/components/ui/separator"
import { highlightAgent } from "@/lib/shiki"
import { cn } from "@/lib/utils"

import { useSyncShikiTheme } from "./use-sync-shiki-theme"

type GitDiffViewerProps = {
  diff: string
  defaultExpanded?: boolean
  gitCommitHash?: string | null
}

type AggregatedStats = {
  filesChanged: number
  additions: number
  deletions: number
}

type FileSummary = {
  key: string
  file: AnyFileChange
  title: string
  additions: number
  deletions: number
  language: string
}

type LineKind =
  | "context"
  | "added"
  | "deleted"
  | "unchanged"
  | "message"
  | "separator"
  | "binary"

type DisplayLine = {
  key: string
  kind: LineKind
  text: string
  lineLabel?: string
}

type RenderedLine = DisplayLine & {
  html?: string
}

const LANGUAGE_MAP: Record<string, string> = {
  tsx: "tsx",
  ts: "typescript",
  jsx: "jsx",
  js: "javascript",
  mjs: "javascript",
  cjs: "javascript",
  json: "json",
  json5: "json",
  py: "python",
  rs: "rust",
  go: "go",
  rb: "ruby",
  java: "java",
  kt: "kotlin",
  swift: "swift",
  cs: "csharp",
  c: "c",
  h: "c",
  cpp: "cpp",
  cc: "cpp",
  cxx: "cpp",
  hpp: "cpp",
  mm: "objective-c",
  php: "php",
  html: "html",
  htm: "html",
  css: "css",
  scss: "scss",
  less: "less",
  md: "markdown",
  mdx: "mdx",
  yaml: "yaml",
  yml: "yaml",
  toml: "toml",
  sql: "sql",
  sh: "bash",
  bash: "bash",
  zsh: "bash",
  fish: "bash",
  dockerfile: "docker",
  makefile: "makefile",
}

export function GitDiffViewer({
  diff,
  defaultExpanded = true,
  gitCommitHash,
}: GitDiffViewerProps) {
  useSyncShikiTheme()

  const normalizedDiff = useMemo(() => diff.replace(/\r\n/g, "\n"), [diff])

  const gitDiff = useMemo(() => {
    if (!normalizedDiff.trim()) {
      return { files: [] as AnyFileChange[] }
    }

    try {
      return parseGitDiff(normalizedDiff)
    } catch {
      return { files: [] as AnyFileChange[] }
    }
  }, [normalizedDiff])

  const fileSummaries = useMemo(() => {
    return gitDiff.files.map((file, index) => summarizeFile(file, index))
  }, [gitDiff.files])

  const stats = useMemo(() => summarizeTotals(fileSummaries), [fileSummaries])

  const [isExpanded, setIsExpanded] = useState(defaultExpanded)
  const [showRaw, setShowRaw] = useState(false)
  const [copiedDiff, setCopiedDiff] = useState(false)
  const [copiedHash, setCopiedHash] = useState(false)
  const [expandedFiles, setExpandedFiles] = useState<Record<string, boolean>>({})

  useEffect(() => {
    const initial: Record<string, boolean> = {}
    for (const summary of fileSummaries) {
      initial[summary.key] = true
    }
    setExpandedFiles(initial)
  }, [fileSummaries])

  const handleToggleFile = useCallback((key: string) => {
    setExpandedFiles((prev) => ({
      ...prev,
      [key]: !(prev[key] ?? true),
    }))
  }, [])

  const handleExpandAll = useCallback(() => {
    const next: Record<string, boolean> = {}
    for (const summary of fileSummaries) {
      next[summary.key] = true
    }
    setExpandedFiles(next)
  }, [fileSummaries])

  const handleCollapseAll = useCallback(() => {
    const next: Record<string, boolean> = {}
    for (const summary of fileSummaries) {
      next[summary.key] = false
    }
    setExpandedFiles(next)
  }, [fileSummaries])

  const handleCopyDiff = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(diff)
      setCopiedDiff(true)
      setTimeout(() => setCopiedDiff(false), 1500)
    } catch {
      setCopiedDiff(false)
    }
  }, [diff])

  const handleCopyHash = useCallback(async () => {
    if (!gitCommitHash) {
      return
    }

    try {
      await navigator.clipboard.writeText(gitCommitHash)
      setCopiedHash(true)
      setTimeout(() => setCopiedHash(false), 1500)
    } catch {
      setCopiedHash(false)
    }
  }, [gitCommitHash])

  if (!diff.trim()) {
    return (
      <div className="rounded-md border border-border bg-muted/20 px-4 py-6 text-center text-sm text-muted-foreground">
        No diff content detected.
      </div>
    )
  }

  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-wrap items-center gap-3 text-sm text-muted-foreground">
          <button
            type="button"
            className="inline-flex items-center gap-2 font-medium text-foreground hover:underline"
            onClick={() => setIsExpanded((value) => !value)}
          >
            {isExpanded ? (
              <IconChevronDown className="size-4" />
            ) : (
              <IconChevronRight className="size-4" />
            )}
            <span className="inline-flex items-center gap-2">
              <IconGitBranch className="size-4 text-muted-foreground" />
              Git diff
            </span>
          </button>
          <Separator orientation="vertical" className="hidden h-4 md:block" />
          <span className="hidden items-center gap-2 text-xs md:inline-flex">
            <IconFileDiff className="size-4" />
            {stats.filesChanged} files
          </span>
          <span className="hidden items-center gap-2 text-xs text-emerald-500 md:inline-flex">
            <IconFilePlus className="size-4" />
            +{stats.additions}
          </span>
          <span className="hidden items-center gap-2 text-xs text-rose-500 md:inline-flex">
            <IconFileMinus className="size-4" />
            -{stats.deletions}
          </span>
          {gitCommitHash ? (
            <Button
              variant={copiedHash ? "default" : "outline"}
              size="sm"
              onClick={handleCopyHash}
            >
              {copiedHash ? (
                <IconCheck className="mr-1 size-4" />
              ) : (
                <IconCopy className="mr-1 size-4" />
              )}
              {copiedHash ? "Hash copied" : `Commit ${gitCommitHash.slice(0, 12)}`}
            </Button>
          ) : null}
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowRaw((value) => !value)}
          >
            <IconFileText className="mr-1 size-4" />
            {showRaw ? "Rendered view" : "Raw diff"}
          </Button>
          <Button
            variant={copiedDiff ? "default" : "outline"}
            size="sm"
            onClick={handleCopyDiff}
          >
            {copiedDiff ? (
              <IconCheck className="mr-1 size-4" />
            ) : (
              <IconCopy className="mr-1 size-4" />
            )}
            {copiedDiff ? "Copied" : "Copy"}
          </Button>
        </div>
      </div>

      {isExpanded ? (
        <div className="overflow-hidden rounded-md border border-border bg-background">
          <div className="flex flex-wrap items-center justify-between gap-2 border-b border-border/60 bg-muted/20 px-4 py-2 text-xs text-muted-foreground">
            <div className="flex items-center gap-3">
              <span>{stats.filesChanged} files changed</span>
              <span className="text-emerald-500 font-medium">+{stats.additions}</span>
              <span className="text-rose-500 font-medium">-{stats.deletions}</span>
            </div>
            <div className="flex items-center gap-2">
              <Button variant="ghost" size="sm" onClick={handleExpandAll}>
                Expand all
              </Button>
              <Button variant="ghost" size="sm" onClick={handleCollapseAll}>
                Collapse all
              </Button>
            </div>
          </div>
          {showRaw ? (
            <ScrollArea className="max-h-[70vh]">
              <pre className="whitespace-pre overflow-auto px-4 py-4 text-sm leading-relaxed">
                {diff}
              </pre>
            </ScrollArea>
          ) : (
            <div className="flex flex-col divide-y divide-border">
              {fileSummaries.map((summary) => (
                <FileDiffSection
                  key={summary.key}
                  summary={summary}
                  isExpanded={expandedFiles[summary.key] ?? true}
                  onToggle={() => handleToggleFile(summary.key)}
                />
              ))}
            </div>
          )}
        </div>
      ) : null}
    </div>
  )
}

function FileDiffSection({
  summary,
  isExpanded,
  onToggle,
}: {
  summary: FileSummary
  isExpanded: boolean
  onToggle: () => void
}) {
  const lines = useMemo(() => buildDisplayLines(summary.file), [summary.file])
  const renderedLines = useRenderedDiffLines(lines, summary.language)

  return (
    <div className="flex flex-col">
      <button
        type="button"
        className="flex w-full items-center justify-between gap-3 bg-muted/20 px-4 py-2 text-left text-sm font-medium text-foreground transition hover:bg-muted/40"
        onClick={onToggle}
      >
        <span className="inline-flex items-center gap-2">
          {isExpanded ? (
            <IconChevronDown className="size-4 text-muted-foreground" />
          ) : (
            <IconChevronRight className="size-4 text-muted-foreground" />
          )}
          <span className="truncate">{summary.title}</span>
          <ChangeTypeBadge file={summary.file} />
        </span>
        <span className="inline-flex items-center gap-2 text-xs">
          <span className="text-emerald-500 font-medium">+{summary.additions}</span>
          <span className="text-rose-500 font-medium">-{summary.deletions}</span>
        </span>
      </button>
      {isExpanded ? (
        <div className="border-t border-border/60 bg-background px-4 py-4">
          {renderedLines.length === 0 ? (
            <div className="text-xs text-muted-foreground">
              No textual changes available for this file.
            </div>
          ) : (
            <div className="flex flex-col gap-1">
              {renderedLines.map((line) =>
                renderDiffLine(line)
              )}
            </div>
          )}
        </div>
      ) : null}
    </div>
  )
}

function renderDiffLine(line: RenderedLine) {
  if (line.kind === "separator") {
    return (
      <div
        key={line.key}
        className="my-2 border-t border-dashed border-border/50"
        aria-hidden="true"
      />
    )
  }

  const presentation = LINE_PRESENTATION[line.kind] ?? LINE_PRESENTATION.default
  const showLineNumber = Boolean(line.lineLabel)
  const lineNumberContent = line.lineLabel ?? (line.kind === "context" ? "@@" : "")

  return (
    <div
      key={line.key}
      className={cn(
        "flex items-stretch gap-2 px-3 text-[10pt] leading-[1.35] transition-colors",
        presentation.container
      )}
    >
      <span
        className={cn(
          "w-14 shrink-0 pr-1 text-right font-mono tabular-nums",
          showLineNumber ? "opacity-60" : line.kind === "context" ? "opacity-45" : "opacity-0",
          presentation.lineNumber
        )}
      >
        {lineNumberContent}
      </span>
      <span
        className={cn(
          "flex-1 min-w-0 font-mono text-[10pt] leading-[1.35] whitespace-pre-wrap break-words",
          presentation.text
        )}
      >
        {renderLineContent(line)}
      </span>
    </div>
  )
}

function renderLineContent(line: RenderedLine) {
  if (line.kind === "binary") {
    return (
      <span className="text-xs text-muted-foreground">
        Binary file contents not shown
      </span>
    )
  }

  if (line.kind === "message") {
    return <span>{line.text || "\u00A0"}</span>
  }

  if (line.kind === "context") {
    if (line.html) {
      return (
        <span
          className="shiki-inline text-muted-foreground"
          dangerouslySetInnerHTML={{ __html: line.html }}
        />
      )
    }

    return <span className="text-muted-foreground">{line.text}</span>
  }

  if (line.html) {
    return (
      <span dangerouslySetInnerHTML={{ __html: line.html }} />
    )
  }

  return <span>{line.text || ""}</span>
}

const LINE_PRESENTATION: Record<LineKind | "default", {
  container: string
  lineNumber: string
  text?: string
}> = {
  added: {
    container: "bg-emerald-500/10 border-l border-emerald-500/40",
    lineNumber: "text-emerald-500",
  },
  deleted: {
    container: "bg-rose-500/10 border-l border-rose-500/40",
    lineNumber: "text-rose-500",
  },
  unchanged: {
    container: "border-l border-border/40",
    lineNumber: "text-muted-foreground/60",
  },
  context: {
    container: "bg-muted/10 border-l border-dashed border-border/40",
    lineNumber: "text-muted-foreground/60",
    text: "text-muted-foreground",
  },
  message: {
    container: "bg-amber-500/15 border-l-2 border-amber-500/60",
    lineNumber: "text-amber-600",
    text: "text-amber-900 dark:text-amber-200",
  },
  binary: {
    container: "bg-muted/20 border-l border-border/50",
    lineNumber: "text-muted-foreground/60",
    text: "text-muted-foreground",
  },
  separator: {
    container: "",
    lineNumber: "",
  },
  default: {
    container: "border-l border-border/40",
    lineNumber: "text-muted-foreground/60",
  },
}

function useRenderedDiffLines(lines: DisplayLine[], language: string): RenderedLine[] {
  const themeVersion = useShikiThemeVersion()
  const [rendered, setRendered] = useState<RenderedLine[]>([])

  useEffect(() => {
    let cancelled = false

    const run = async () => {
      const results: RenderedLine[] = []

      for (const line of lines) {
        if (line.kind === "added" || line.kind === "deleted" || line.kind === "unchanged") {
          try {
            const html = await highlightAgent.highlightInline({
              code: line.text ?? " ",
              lang: language || "diff",
            })
            results.push({ ...line, html })
         } catch {
            const safe = escapeHtml(line.text ?? "")
            results.push({
              ...line,
              html: safe,
            })
          }
          continue
        }

        if (line.kind === "context") {
          try {
            const html = await highlightAgent.highlightInline({
              code: line.text ?? " ",
              lang: "diff",
            })
            results.push({ ...line, html })
          } catch {
            const safe = escapeHtml(line.text ?? "")
            results.push({
              ...line,
              html: safe,
            })
          }
          continue
        }

        results.push(line)
      }

      if (!cancelled) {
        setRendered(results)
      }
    }

    void run()

    return () => {
      cancelled = true
    }
  }, [lines, language, themeVersion])

  return rendered
}

function buildDisplayLines(file: AnyFileChange): DisplayLine[] {
  const lines: DisplayLine[] = []

  file.chunks.forEach((chunk, chunkIndex) => {
    if (chunk.type === "BinaryFilesChunk") {
      lines.push({
        key: `binary-${chunkIndex}`,
        kind: "binary",
        text: "Binary file contents not shown",
      })
      return
    }

    if (chunkIndex > 0) {
      lines.push({
        key: `separator-${chunkIndex}`,
        kind: "separator",
        text: "",
      })
    }

    const rangeLabel = getChunkRangeLabel(chunk)
    lines.push({
      key: `context-${chunkIndex}`,
      kind: "context",
      text: rangeLabel,
    })

    chunk.changes.forEach((change, changeIndex) => {
      const keyBase = `${chunkIndex}-${changeIndex}`
      switch (change.type) {
        case "AddedLine":
          lines.push({
            key: `added-${keyBase}`,
            kind: "added",
            text: change.content ?? "",
            lineLabel:
              typeof change.lineAfter === "number" ? `+${change.lineAfter}` : undefined,
          })
          break
        case "DeletedLine":
          lines.push({
            key: `deleted-${keyBase}`,
            kind: "deleted",
            text: change.content ?? "",
            lineLabel:
              typeof change.lineBefore === "number" ? `-${change.lineBefore}` : undefined,
          })
          break
        case "UnchangedLine":
          lines.push({
            key: `context-line-${keyBase}`,
            kind: "unchanged",
            text: change.content ?? "",
            lineLabel:
              typeof change.lineAfter === "number"
                ? `${change.lineAfter}`
                : typeof change.lineBefore === "number"
                  ? `${change.lineBefore}`
                  : undefined,
          })
          break
        case "MessageLine":
          lines.push({
            key: `message-${keyBase}`,
            kind: "message",
            text: change.content ?? "",
          })
          break
        default:
          break
      }
    })
  })

  return lines
}

function getChunkRangeLabel(chunk: AnyChunk): string {
  if (chunk.type === "BinaryFilesChunk") {
    return "@@"
  }

  if (chunk.type === "CombinedChunk") {
    return `@@@ -${chunk.fromFileRangeA.start},${chunk.fromFileRangeA.lines} -${chunk.fromFileRangeB.start},${chunk.fromFileRangeB.lines} +${chunk.toFileRange.start},${chunk.toFileRange.lines} @@@`
  }

  return `@@ -${chunk.fromFileRange.start},${chunk.fromFileRange.lines} +${chunk.toFileRange.start},${chunk.toFileRange.lines} @@${chunk.context?.trim() ? ` ${chunk.context.trim()}` : ""}`
}

function summarizeFile(file: AnyFileChange, index: number): FileSummary {
  const title = getFileTitle(file)
  const { additions, deletions } = countChangesFromFile(file)
  const language = inferLanguageFromFile(file)

  return {
    key: `${index}-${title}`,
    file,
    title,
    additions,
    deletions,
    language,
  }
}

function summarizeTotals(files: FileSummary[]): AggregatedStats {
  return files.reduce<AggregatedStats>(
    (acc, summary) => {
      acc.filesChanged += 1
      acc.additions += summary.additions
      acc.deletions += summary.deletions
      return acc
    },
    { filesChanged: 0, additions: 0, deletions: 0 }
  )
}

function countChangesFromFile(file: AnyFileChange) {
  let additions = 0
  let deletions = 0

  file.chunks.forEach((chunk) => {
    if (chunk.type === "BinaryFilesChunk") {
      return
    }

    chunk.changes.forEach((change: AnyLineChange) => {
      if (change.type === "AddedLine") {
        additions += 1
      } else if (change.type === "DeletedLine") {
        deletions += 1
      }
    })
  })

  return { additions, deletions }
}

function getFileTitle(file: AnyFileChange): string {
  if (file.type === "RenamedFile") {
    if (file.pathBefore === file.pathAfter) {
      return file.pathAfter
    }
    return `${file.pathBefore} → ${file.pathAfter}`
  }

  if ("path" in file && file.path) {
    return file.path
  }

  return "(unknown path)"
}

function getFileTargetPath(file: AnyFileChange): string {
  if (file.type === "RenamedFile") {
    return file.pathAfter || file.pathBefore || ""
  }

  if ("path" in file) {
    return file.path || ""
  }

  return ""
}

function inferLanguageFromFile(file: AnyFileChange): string {
  const path = getFileTargetPath(file)
  const lower = path.toLowerCase()
  const parts = lower.split("/")
  const filename = parts[parts.length - 1] ?? ""

  if (LANGUAGE_MAP[filename]) {
    return LANGUAGE_MAP[filename]
  }

  const extension = filename.split(".").pop() ?? ""
  if (!extension) {
    return "diff"
  }

  return LANGUAGE_MAP[extension] ?? "diff"
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;")
}

function useShikiThemeVersion(): number {
  return useSyncExternalStore(
    (listener) => highlightAgent.subscribe(listener),
    () => highlightAgent.getThemeVersion(),
    () => highlightAgent.getThemeVersion()
  )
}

function ChangeTypeBadge({ file }: { file: AnyFileChange }) {
  let label = "Modified"
  let tone = "text-muted-foreground"
  let Icon = IconCircleDot

  switch (file.type) {
    case "AddedFile":
      label = "Added"
      tone = "text-emerald-500"
      Icon = IconCirclePlus
      break
    case "DeletedFile":
      label = "Removed"
      tone = "text-rose-500"
      Icon = IconCircleMinus
      break
    case "RenamedFile":
      label = "Renamed"
      tone = "text-blue-500"
      Icon = IconArrowsExchange
      break
    default:
      break
  }

  return (
    <Badge variant="outline" className={cn("inline-flex items-center gap-1 text-[10px] uppercase", tone)}>
      <Icon className="size-3" />
      {label}
    </Badge>
  )
}
