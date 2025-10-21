import { useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore } from 'react';
import parseGitDiff, {
  type AddedFile,
  type AnyFileChange,
  type AnyChunk,
  type ChangedFile,
  type DeletedFile,
  type RenamedFile,
} from 'parse-git-diff';
import {
  ChevronDown,
  ChevronRight,
  ChevronUp,
  Check,
  Copy,
  FileCode2,
  GitBranch,
  Scroll,
  Code,
  Maximize2,
  Minimize2,
} from 'lucide-react';
import type { PatchMetadata } from '../types';
import { Button } from './ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  DropdownMenuLabel,
  DropdownMenuSeparator,
} from './ui/dropdown-menu';
import { cn } from '../lib/utils';
import { highlightAgent } from '../lib/shiki';
import { useCodeTheme } from '../contexts/CodeThemeContext';
import { extractDiffContent } from '../utils/diff';

interface GitDiffViewerProps {
  emailBody: string | null;
  patchMetadata: PatchMetadata | null;
  gitCommitHash?: string;
  defaultExpanded?: boolean;
}

export function GitDiffViewer({
  emailBody,
  patchMetadata,
  gitCommitHash,
  defaultExpanded = false,
}: GitDiffViewerProps) {
  const [isExpanded, setIsExpanded] = useState(defaultExpanded);
  const [showRaw, setShowRaw] = useState(false);
  const [expandedFiles, setExpandedFiles] = useState<Record<string, boolean>>({});
  const [copiedRawDiff, setCopiedRawDiff] = useState(false);
  const [copiedHash, setCopiedHash] = useState(false);
  const [fullScreenFileKey, setFullScreenFileKey] = useState<string | null>(null);
  const copyRawTimeoutRef = useRef<number | null>(null);
  const copyHashTimeoutRef = useRef<number | null>(null);
  const { availableThemes, codeTheme, setCodeTheme } = useCodeTheme();

  if (!emailBody) {
    return null;
  }

  const diffContent = extractDiffContent(emailBody, patchMetadata);

  if (!diffContent.trim()) {
    return null;
  }

  const { parsedDiff, stats, fileSummaries, parseError } = useMemo(() => {
    try {
      const parsed = parseDiffContent(diffContent);
      const summaries = parsed.files.map((file, index) => {
        const baseKey = getFileKey(file);
        const uniqueKey = `${baseKey}#${index}`;

        return {
          file,
          key: uniqueKey,
          displayPath: getDisplayPath(file),
          additions: countLineChanges(file, 'AddedLine'),
          deletions: countLineChanges(file, 'DeletedLine'),
        };
      });

      const aggregate = summaries.reduce(
        (acc, summary) => {
          acc.additions += summary.additions;
          acc.deletions += summary.deletions;
          return acc;
        },
        { additions: 0, deletions: 0 }
      );

      return {
        parsedDiff: parsed,
        fileSummaries: summaries,
        stats: {
          filesChanged: parsed.files.length,
          additions: aggregate.additions,
          deletions: aggregate.deletions,
        },
        parseError: null as string | null,
      };
    } catch (err) {
      console.error('Failed to parse git diff', err);
      return {
        parsedDiff: null,
        fileSummaries: [],
        stats: null,
        parseError: 'Unable to parse git diff content.',
      };
    }
  }, [diffContent, setCopiedRawDiff]);

  useEffect(() => {
    if (!fullScreenFileKey) {
      return;
    }

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setFullScreenFileKey(null);
      }
    };

    window.addEventListener('keydown', handleKeyDown);

    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = 'hidden';

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      document.body.style.overflow = previousOverflow;
    };
  }, [fullScreenFileKey]);

  useEffect(() => {
    if (!fullScreenFileKey) {
      return;
    }

    const exists = fileSummaries.some((summary) => summary.key === fullScreenFileKey);
    if (!exists) {
      setFullScreenFileKey(null);
    }
  }, [fileSummaries, fullScreenFileKey]);

  const fullScreenSummary = useMemo(() => {
    if (!fullScreenFileKey) {
      return null;
    }
    return fileSummaries.find((summary) => summary.key === fullScreenFileKey) ?? null;
  }, [fileSummaries, fullScreenFileKey]);

  const fullScreenLanguage = fullScreenSummary
    ? inferLanguage(fullScreenSummary.displayPath)
    : null;

  const handleCopyRawDiff = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(diffContent);
      setCopiedRawDiff(true);
      if (copyRawTimeoutRef.current) {
        window.clearTimeout(copyRawTimeoutRef.current);
      }
      copyRawTimeoutRef.current = window.setTimeout(() => {
        setCopiedRawDiff(false);
        copyRawTimeoutRef.current = null;
      }, 1600);
    } catch (err) {
      console.error('Failed to copy diff to clipboard', err);
    }
  }, [diffContent]);

  useEffect(() => {
    return () => {
      if (copyRawTimeoutRef.current) {
        window.clearTimeout(copyRawTimeoutRef.current);
        copyRawTimeoutRef.current = null;
      }
      if (copyHashTimeoutRef.current) {
        window.clearTimeout(copyHashTimeoutRef.current);
        copyHashTimeoutRef.current = null;
      }
    };
  }, []);

  const handleCopyHash = useCallback(async () => {
    if (!gitCommitHash) {
      return;
    }
    try {
      await navigator.clipboard.writeText(gitCommitHash);
      setCopiedHash(true);
      if (copyHashTimeoutRef.current) {
        window.clearTimeout(copyHashTimeoutRef.current);
      }
      copyHashTimeoutRef.current = window.setTimeout(() => {
        setCopiedHash(false);
        copyHashTimeoutRef.current = null;
      }, 1600);
    } catch (err) {
      console.error('Failed to copy commit hash', err);
    }
  }, [gitCommitHash]);

  const toggleFile = useCallback((fileKey: string) => {
    setExpandedFiles((prev) => ({
      ...prev,
      [fileKey]: !(prev[fileKey] ?? true),
    }));
  }, []);

  return (
    <div
      className="mt-3 border border-surface-border/60 rounded-md overflow-hidden max-w-full w-full min-w-0"
      style={{ textRendering: 'optimizeLegibility' }}
    >
      <div className="px-3 py-1 bg-surface-inset/50 flex flex-wrap items-center justify-between gap-2">
        <button
          type="button"
          onClick={() => setIsExpanded((value) => !value)}
          className="flex items-center gap-3 py-0.5 text-left hover:text-foreground transition-colors flex-1 min-w-0"
        >
          <div className="flex items-center gap-2">
            {isExpanded ? (
              <ChevronUp className="h-4 w-4 text-muted-foreground" />
            ) : (
              <ChevronDown className="h-4 w-4 text-muted-foreground" />
            )}
            <GitBranch className="h-4 w-4 text-muted-foreground" />
            <span className="text-sm font-medium">Git Diff</span>
          </div>
          {gitCommitHash && (
            <button
              type="button"
              className={cn(
                "text-xs text-muted-foreground cursor-pointer hover:text-foreground transition-colors px-1 py-0.5 rounded",
                copiedHash && "text-emerald-500 bg-emerald-500/10"
              )}
              title={copiedHash ? "Copied!" : "Click to copy commit hash"}
              onClick={(event) => {
                event.stopPropagation();
                void handleCopyHash();
              }}
            >
              {copiedHash ? (
                <span className="flex items-center gap-1">
                  <Check className="h-3 w-3" />
                  ({gitCommitHash.substring(0, 12)})
                </span>
              ) : (
                `(${gitCommitHash.substring(0, 12)})`
              )}
            </button>
          )}
          {stats && (
            <span className="flex items-center gap-2 text-xs text-muted-foreground">
              <span>{stats.filesChanged} file{stats.filesChanged === 1 ? '' : 's'}</span>
              <span className="text-emerald-500 font-medium">+{stats.additions}</span>
              <span className="text-rose-500 font-medium">-{stats.deletions}</span>
            </span>
          )}
        </button>

        <div className="flex items-center gap-2">
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="h-8 w-8"
                title="Code Theme"
              >
                <Code className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-48 max-h-64 overflow-y-auto">
              <DropdownMenuLabel className="text-xs text-muted-foreground px-2 py-1.5">
                Code Theme
              </DropdownMenuLabel>
              <DropdownMenuSeparator />
              {availableThemes.map((theme) => (
                <DropdownMenuItem
                  key={theme}
                  onClick={() => setCodeTheme(theme)}
                  className={cn(
                    "text-xs capitalize px-2 py-1.5",
                    codeTheme === theme && "bg-muted"
                  )}
                >
                  {theme.replace(/[-_]/g, ' ')}
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>
          <Button
            variant="ghost"
            size="icon"
            className={cn(
              'h-8 w-8 transition-colors',
              copiedRawDiff && 'text-emerald-500'
            )}
            type="button"
            onClick={handleCopyRawDiff}
            title={copiedRawDiff ? 'Copied!' : 'Copy raw diff'}
          >
            {copiedRawDiff ? (
              <Check className="h-4 w-4" />
            ) : (
              <Copy className="h-4 w-4" />
            )}
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className={cn('h-8 w-8', showRaw && 'bg-surface-inset/60')}
            type="button"
            onClick={() => setShowRaw((value) => !value)}
            title="Toggle raw diff"
          >
            <Scroll className="h-4 w-4" />
          </Button>
        </div>
      </div>

      {/* Expandable Content */}
      {isExpanded && (
        <div className="border-t border-surface-border/60 overflow-hidden bg-surface">
          {parseError && (
            <div className="px-4 py-3 text-xs text-destructive bg-destructive/10 border-b border-destructive/30">
              {parseError}
            </div>
          )}
          {showRaw ? (
            <div className="raw-diff-wrapper">
              <RawDiffView diff={diffContent} />
            </div>
          ) : (
            <div className="space-y-2 p-3">
              {parsedDiff && parsedDiff.files.length === 0 && (
                <div className="text-sm text-muted-foreground">
                  No file changes detected in this diff.
                </div>
              )}

              {fileSummaries.map((summary, fileIndex) => {
                const file = summary.file;
                const fileKey = summary.key || `file-${fileIndex}`;
                const isFileExpanded = expandedFiles[fileKey] ?? true;
                const language = inferLanguage(summary.displayPath);
                const isFullScreen = fullScreenFileKey === fileKey;

                return (
                  <div
                    key={fileKey}
                    className="overflow-hidden border border-surface-border/60 rounded-md bg-surface min-w-0"
                  >
                    <div className="flex items-center bg-surface">
                      <button
                        type="button"
                        className="flex-1 px-3 py-1.5 hover:bg-surface-inset/70 transition-colors flex items-center justify-between text-left gap-3 min-w-0"
                        onClick={() => toggleFile(fileKey)}
                      >
                        <div className="flex flex-wrap items-center gap-2 min-w-0">
                          <ChevronRight
                            className={cn(
                              'h-4 w-4 text-muted-foreground transition-transform',
                              isFileExpanded && 'rotate-90'
                            )}
                          />
                          <FileCode2 className="h-4 w-4 text-muted-foreground" />
                          <span className="text-sm font-medium text-foreground break-words min-w-0">
                            {summary.displayPath}
                          </span>
                          <FileBadge file={file} />
                          <span className="text-xs text-emerald-500 font-medium">+{summary.additions}</span>
                          <span className="text-xs text-rose-500 font-medium">-{summary.deletions}</span>
                        </div>
                        <span className="text-xs text-muted-foreground uppercase tracking-wide">
                          {language.toUpperCase()}
                        </span>
                      </button>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 mr-2 shrink-0"
                        type="button"
                        title={isFullScreen ? 'Exit full screen' : 'Expand to full screen'}
                        onClick={(event) => {
                          event.stopPropagation();
                          setExpandedFiles((prev) => ({
                            ...prev,
                            [fileKey]: true,
                          }));
                          setFullScreenFileKey((current) => (current === fileKey ? null : fileKey));
                        }}
                      >
                        {isFullScreen ? (
                          <Minimize2 className="h-4 w-4" />
                        ) : (
                          <Maximize2 className="h-4 w-4" />
                        )}
                      </Button>
                    </div>

                    {isFileExpanded && (
                      <div
                        className={cn(
                          'border-t border-surface-border/60 pb-1.5 overflow-x-auto',
                          !isFullScreen && 'max-h-[32rem] overflow-y-auto'
                        )}
                      >
                        <FileDiffContent file={file} language={language} />
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </div>
      )}
      {fullScreenSummary && fullScreenLanguage && (
        <div
          className="fixed inset-0 z-50 bg-background/95 backdrop-blur-sm flex flex-col"
          role="dialog"
          aria-modal="true"
        >
          <div className="flex items-center justify-between gap-3 px-5 py-3 border-b border-surface-border/80 bg-surface">
            <div className="flex flex-wrap items-center gap-2 min-w-0">
              <FileCode2 className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm font-medium text-foreground break-words min-w-0">
                {fullScreenSummary.displayPath}
              </span>
              <FileBadge file={fullScreenSummary.file} />
              <span className="text-xs text-emerald-500 font-medium">
                +{fullScreenSummary.additions}
              </span>
              <span className="text-xs text-rose-500 font-medium">
                -{fullScreenSummary.deletions}
              </span>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-xs text-muted-foreground uppercase tracking-wide">
                {fullScreenLanguage.toUpperCase()}
              </span>
              <Button
                variant="ghost"
                size="icon"
                className="h-8 w-8"
                type="button"
                title="Exit full screen"
                onClick={() => setFullScreenFileKey(null)}
              >
                <Minimize2 className="h-4 w-4" />
              </Button>
            </div>
          </div>
          <div className="flex-1 overflow-auto bg-surface">
            <div className="max-w-5xl mx-auto w-full px-4 py-4">
              <FileDiffContent file={fullScreenSummary.file} language={fullScreenLanguage} />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function parseDiffContent(diffContent: string): ReturnType<typeof parseGitDiff> {
  const normalizedContent = diffContent.replace(/\r\n/g, '\n').replace(/^\s+/, '');

  if (!normalizedContent.trim()) {
    return {
      type: 'GitDiff',
      files: [],
    };
  }

  const segments = splitDiffIntoSegments(normalizedContent);

  if (segments.length <= 1) {
    return parseGitDiff(normalizedContent);
  }

  const aggregatedRecords = aggregateSegmentsByFile(segments);

  if (aggregatedRecords.length === 0) {
    return parseGitDiff(normalizedContent);
  }

  return {
    type: 'GitDiff',
    files: aggregatedRecords,
  };
}

function splitDiffIntoSegments(diffContent: string): string[] {
  return diffContent
    .split(/(?=^diff --git)/gm)
    .map((segment) => segment.replace(/^\s+/, ''))
    .filter((segment) => segment.startsWith('diff --git'));
}

function aggregateSegmentsByFile(segments: string[]): AnyFileChange[] {
  interface AggregatedRecord {
    key: string;
    template: AnyFileChange;
    chunks: AnyChunk[];
  }

  const records: AggregatedRecord[] = [];
  const recordMap = new Map<string, AggregatedRecord>();

  segments.forEach((segment) => {
    const parsed = parseGitDiff(segment);
    parsed.files.forEach((file) => {
      const key = getAggregationKey(file);
      if (!key) {
        return;
      }

      let record = recordMap.get(key);
      if (!record) {
        record = {
          key,
          template: createTemplateFromFile(file),
          chunks: [],
        };
        recordMap.set(key, record);
        records.push(record);
      }

      const clonedChunks = file.chunks.map(cloneChunk);
      record.chunks.push(...clonedChunks);
    });
  });

  return records.map((record) => buildAggregatedFile(record.template, record.chunks));
}

function getAggregationKey(file: AnyFileChange): string {
  if (isRenamedFile(file)) {
    return file.pathAfter || file.pathBefore;
  }

  if (isPathFile(file)) {
    return file.path;
  }

  return '';
}

function createTemplateFromFile(file: AnyFileChange): AnyFileChange {
  switch (file.type) {
    case 'RenamedFile':
      return {
        type: 'RenamedFile',
        pathBefore: file.pathBefore,
        pathAfter: file.pathAfter,
        chunks: [],
      };
    case 'AddedFile':
      return {
        type: 'AddedFile',
        path: file.path,
        chunks: [],
      };
    case 'DeletedFile':
      return {
        type: 'DeletedFile',
        path: file.path,
        chunks: [],
      };
    default:
      return {
        type: 'ChangedFile',
        path: file.path,
        chunks: [],
      };
  }
}

function buildAggregatedFile(template: AnyFileChange, chunks: AnyChunk[]): AnyFileChange {
  switch (template.type) {
    case 'RenamedFile':
      return {
        ...template,
        chunks,
      };
    case 'AddedFile':
      return {
        ...template,
        chunks,
      };
    case 'DeletedFile':
      return {
        ...template,
        chunks,
      };
    default:
      return {
        ...template,
        chunks,
      };
  }
}

function cloneChunk(chunk: AnyChunk): AnyChunk {
  if (chunk.type === 'BinaryFilesChunk') {
    return { ...chunk };
  }

  if (chunk.type === 'CombinedChunk') {
    return {
      ...chunk,
      fromFileRangeA: { ...chunk.fromFileRangeA },
      fromFileRangeB: { ...chunk.fromFileRangeB },
      toFileRange: { ...chunk.toFileRange },
      changes: chunk.changes.map((change) => ({ ...change })) as typeof chunk.changes,
    };
  }

  return {
    ...chunk,
    fromFileRange: { ...chunk.fromFileRange },
    toFileRange: { ...chunk.toFileRange },
    changes: chunk.changes.map((change) => ({ ...change })) as typeof chunk.changes,
  };
}

function getFileKey(file: AnyFileChange): string {
  if (isRenamedFile(file)) {
    return `${file.pathBefore}->${file.pathAfter}`;
  }

  if (isPathFile(file)) {
    return file.path;
  }

  return Math.random().toString(36).slice(2);
}

function getDisplayPath(file: AnyFileChange): string {
  if (isRenamedFile(file)) {
    return `${file.pathBefore} → ${file.pathAfter}`;
  }

  if (isPathFile(file)) {
    return file.path;
  }

  return 'Unknown file';
}

function inferLanguage(path: string): string {
  const extension = path.split('.').pop()?.toLowerCase() ?? '';
  const languageByExtension: Record<string, string> = {
    tsx: 'tsx',
    ts: 'typescript',
    jsx: 'jsx',
    js: 'javascript',
    mjs: 'javascript',
    cjs: 'javascript',
    json: 'json',
    md: 'markdown',
    mdx: 'markdown',
    css: 'css',
    scss: 'scss',
    less: 'less',
    html: 'html',
    htm: 'html',
    svg: 'markup',
    xml: 'markup',
    yaml: 'yaml',
    yml: 'yaml',
    sh: 'bash',
    bash: 'bash',
    zsh: 'bash',
    py: 'python',
    go: 'go',
    rs: 'rust',
    java: 'java',
    kt: 'kotlin',
    swift: 'swift',
    cs: 'csharp',
    cpp: 'cpp',
    cxx: 'cpp',
    cc: 'cpp',
    c: 'c',
    h: 'c',
    hpp: 'cpp',
    php: 'php',
    rb: 'ruby',
    sql: 'sql',
    txt: 'text',
  };

  return languageByExtension[extension] ?? 'diff';
}

type LineChangeType = 'AddedLine' | 'DeletedLine';

function countLineChanges(file: AnyFileChange, targetType: LineChangeType): number {
  return file.chunks.reduce((fileTotal, chunk) => {
    if (chunk.type === 'BinaryFilesChunk') {
      return fileTotal;
    }

    const chunkCount = chunk.changes.reduce((chunkTotal, change) => {
      return change.type === targetType ? chunkTotal + 1 : chunkTotal;
    }, 0);

    return fileTotal + chunkCount;
  }, 0);
}

function isRenamedFile(file: AnyFileChange): file is RenamedFile {
  return file.type === 'RenamedFile';
}

function isPathFile(file: AnyFileChange): file is AddedFile | ChangedFile | DeletedFile {
  return file.type === 'AddedFile' || file.type === 'ChangedFile' || file.type === 'DeletedFile';
}

interface DisplayLine {
  key: string;
  type: 'context' | 'divider' | 'spacer' | 'added' | 'deleted' | 'unchanged' | 'message' | 'binary';
  text: string;
  lineLabel?: string;
  chunkIndex?: number;
}

function FileDiffContent({ file, language }: { file: AnyFileChange; language: string }) {
  const lines = buildDisplayLines(file);
  const addedLinesByChunk = useMemo(() => {
    const map = new Map<number, string[]>();

    file.chunks.forEach((chunk, index) => {
      if (chunk.type === 'BinaryFilesChunk') {
        return;
      }

      const additions = chunk.changes
        .filter((change) => change.type === 'AddedLine')
        .map((change) => change.content ?? '');

      if (additions.length > 0) {
        map.set(index, additions);
      }
    });

    return map;
  }, [file]);

  if (lines.length === 0) {
    return <div className="text-xs text-muted-foreground">No textual changes available for this file.</div>;
  }

  const shikiBackgroundStyle = {
    backgroundColor: 'var(--shiki-editor-background, transparent)',
    color: 'var(--shiki-editor-foreground, inherit)',
  } as const;

  return (
    <div style={shikiBackgroundStyle} className="min-w-0">
      {lines.map((line) => {
        const { containerClass, textClass, lineNumberClass } = getLinePresentation(line.type);
        const showLineNumber =
          line.type !== 'context' &&
          line.type !== 'divider' &&
          line.type !== 'spacer' &&
          line.lineLabel;
        const lineNumberWidth =
          line.type === 'divider' || line.type === 'spacer'
            ? 'w-0 pr-0'
            : line.type === 'context'
            ? 'w-14 pr-1'
            : showLineNumber
            ? 'w-14 pr-1'
            : 'w-auto pr-2';
        const gapClass =
          line.type === 'context'
            ? 'gap-1'
            : line.type === 'divider' || line.type === 'spacer'
            ? 'gap-0'
            : 'gap-2';
        const paddingClass =
          line.type === 'divider'
            ? 'py-1.5'
            : line.type === 'spacer'
            ? 'py-1.5'
            : 'py-0.5';

        const chunkAdditions =
          line.chunkIndex !== undefined ? addedLinesByChunk.get(line.chunkIndex) ?? [] : [];
        const contextLabel = line.type === 'context' ? extractContextLabel(line.text) : '';

        return (
          <div
            key={line.key}
            className={cn(
              'flex items-stretch px-3 text-[10pt] leading-[1.35] transition-colors',
              paddingClass,
              gapClass,
              containerClass
            )}
          >
            <span
              className={cn(
                lineNumberWidth,
                'shrink-0 text-right font-mono text-[10pt] leading-[1.35] tabular-nums transition-opacity select-none',
                lineNumberClass,
                showLineNumber ? 'opacity-60' : line.type === 'context' ? 'opacity-45' : 'opacity-0'
              )}
            >
              {showLineNumber ? line.lineLabel : line.type === 'context' ? '@@' : ''}
            </span>
            {line.type === 'context' ? (
              <div className="flex flex-1 items-center gap-2 min-w-0 font-mono text-[10pt] leading-[1.35] whitespace-pre-wrap break-words">
                <span className={cn('flex-1 min-w-0 break-words', textClass)}>
                  <ContextLineRenderer text={line.text} language={language} />
                </span>
                {chunkAdditions.length > 0 && (
                  <FunctionCopyButton
                    addedLines={chunkAdditions}
                    label={contextLabel}
                  />
                )}
              </div>
            ) : (
              <span
                className={cn(
                  'flex-1 min-w-0 font-mono text-[10pt] leading-[1.35] whitespace-pre-wrap break-words',
                  textClass
                )}
              >
                {line.type === 'divider' ? (
                  <span
                    className="block w-full border-t border-dashed border-surface-border/70"
                    aria-hidden="true"
                  />
                ) : line.type === 'spacer' ? (
                  <span className="block w-full" aria-hidden="true" />
                ) : line.type === 'added' || line.type === 'deleted' || line.type === 'unchanged' ? (
                  <HighlightedCode
                    code={line.text || ' '}
                    language={language}
                    variant="inline"
                    className="text-[10pt] leading-[1.35]"
                  />
                ) : (
                  <span className="block whitespace-pre-wrap break-words text-[10pt] leading-[1.35] text-muted-foreground">
                    {line.text || ' '}
                  </span>
                )}
              </span>
            )}
          </div>
        );
      })}
    </div>
  );
}

function buildDisplayLines(file: AnyFileChange): DisplayLine[] {
  const allLines: DisplayLine[] = [];

  file.chunks.forEach((chunk, chunkIndex) => {
    if (chunk.type === 'BinaryFilesChunk') {
      allLines.push({
        key: `binary-${chunkIndex}`,
        type: 'binary',
        text: 'Binary file contents not shown',
        chunkIndex,
      });

      return;
    }

    let rangePrefix = '';
    if (chunk.type === 'CombinedChunk') {
      rangePrefix = `@@@ -${chunk.fromFileRangeA.start},${chunk.fromFileRangeA.lines} -${chunk.fromFileRangeB.start},${chunk.fromFileRangeB.lines} +${chunk.toFileRange.start},${chunk.toFileRange.lines} @@@`;
    } else {
      rangePrefix = `@@ -${chunk.fromFileRange.start},${chunk.fromFileRange.lines} +${chunk.toFileRange.start},${chunk.toFileRange.lines} @@`;
    }
    const contextLine = chunk.context?.trim()
      ? `${rangePrefix} ${chunk.context.trim()}`
      : rangePrefix;

    allLines.push({
      key: `divider-top-${chunkIndex}`,
      type: 'divider',
      text: '',
      chunkIndex,
    });

    allLines.push({
      key: `context-${chunkIndex}`,
      type: 'context',
      text: contextLine,
      chunkIndex,
    });

    allLines.push({
      key: `divider-bottom-${chunkIndex}`,
      type: 'spacer',
      text: '',
      chunkIndex,
    });

    chunk.changes.forEach((change, changeIndex) => {
      switch (change.type) {
        case 'AddedLine':
          allLines.push({
            key: `add-${chunkIndex}-${changeIndex}`,
            type: 'added',
            text: change.content,
            lineLabel: change.lineAfter !== undefined ? `+${change.lineAfter}` : undefined,
            chunkIndex,
          });
          break;
        case 'DeletedLine':
          allLines.push({
            key: `del-${chunkIndex}-${changeIndex}`,
            type: 'deleted',
            text: change.content,
            lineLabel: change.lineBefore !== undefined ? `-${change.lineBefore}` : undefined,
            chunkIndex,
          });
          break;
        case 'UnchangedLine':
          allLines.push({
            key: `same-${chunkIndex}-${changeIndex}`,
            type: 'unchanged',
            text: change.content,
            lineLabel:
              change.lineAfter !== undefined
                ? `${change.lineAfter}`
                : change.lineBefore !== undefined
                ? `${change.lineBefore}`
                : undefined,
            chunkIndex,
          });
          break;
        case 'MessageLine':
          allLines.push({
            key: `msg-${chunkIndex}-${changeIndex}`,
            type: 'message',
            text: change.content,
            chunkIndex,
          });
          break;
        default:
          break;
      }
    });
  });

  return allLines;
}

function RawDiffView({ diff }: { diff: string }) {
  const lines = diff.split('\n');
  
  return (
    <div className="raw-diff-container w-full">
      {lines.map((line, index) => {
        // Determine line type for proper spacing
        const isDivider = line.startsWith('@@') && line.includes('@@');
        const isEmpty = line.trim() === '';
        
        // Apply spacing based on line type to match default view
        const paddingClass = isDivider || isEmpty ? 'py-1.5' : 'py-0.5';
        
        return (
          <div
            key={index}
            className={cn(
              'flex items-stretch px-3 text-[10pt] leading-[1.35] transition-colors',
              paddingClass,
              'gap-2 border-l-0'
            )}
          >
            <span className="w-0 pr-0 shrink-0 text-right font-mono text-[10pt] leading-[1.35] tabular-nums transition-opacity select-none opacity-0"></span>
            <span className="flex-1 min-w-0 font-mono text-[10pt] leading-[1.35] whitespace-pre-wrap break-words">
              <HighlightedCode
                code={line || ' '}
                language="diff"
                variant="inline"
                className="text-[10pt] leading-[1.35]"
              />
            </span>
          </div>
        );
      })}
    </div>
  );
}

function extractContextLabel(text: string): string {
  const trimmed = text.trimEnd();
  const match = trimmed.match(/^(@{2,3}\s+[^@]+@@@?)\s*(.*)$/);
  return match && match[2] ? match[2].trim() : '';
}

function ContextLineRenderer({ text, language }: { text: string; language: string }) {
  const trailingCode = extractContextLabel(text);

  if (!trailingCode) {
    return (
      <span className="whitespace-pre-wrap break-words font-mono text-muted-foreground/70 leading-[1.35]">
        context
      </span>
    );
  }

  return (
    <HighlightedCode
      code={trailingCode}
      language={language}
      variant="inline"
      className="text-[10pt] leading-[1.35]"
    />
  );
}

function FunctionCopyButton({ addedLines, label }: { addedLines: string[]; label: string }) {
  const [copied, setCopied] = useState(false);
  const timeoutRef = useRef<number | null>(null);

  const copyText = useCallback(async (value: string): Promise<boolean> => {
    try {
      if (navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(value);
        return true;
      }
    } catch (err) {
      console.error('Primary clipboard write failed, falling back to legacy API', err);
    }

    const textarea = document.createElement('textarea');
    textarea.value = value;
    textarea.setAttribute('readonly', '');
    textarea.style.position = 'fixed';
    textarea.style.left = '-9999px';
    textarea.style.top = '0';
    textarea.style.opacity = '0';

    document.body.appendChild(textarea);
    const activeElement = document.activeElement as HTMLElement | null;

    textarea.select();
    textarea.setSelectionRange(0, textarea.value.length);

    let succeeded = false;
    try {
      succeeded = document.execCommand('copy');
    } catch (err) {
      console.error('Fallback clipboard copy failed', err);
    }

    if (activeElement && typeof activeElement.focus === 'function') {
      activeElement.focus();
    }
    const selection = document.getSelection();
    selection?.removeAllRanges();

    document.body.removeChild(textarea);
    return succeeded;
  }, []);

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        window.clearTimeout(timeoutRef.current);
        timeoutRef.current = null;
      }
    };
  }, []);

  const handleCopy = useCallback(async () => {
    if (addedLines.length === 0) {
      return;
    }

    const textToCopy = addedLines.join('\n');
    const success = await copyText(textToCopy);

    if (success) {
      setCopied(true);
      if (timeoutRef.current) {
        window.clearTimeout(timeoutRef.current);
      }
      timeoutRef.current = window.setTimeout(() => {
        setCopied(false);
        timeoutRef.current = null;
      }, 1600);
    } else {
      console.error('Failed to copy added lines for function context');
    }
  }, [addedLines, copyText]);

  const labelSuffix = label ? ` for ${label}` : '';
  const title = copied ? 'Copied!' : `Copy added lines${labelSuffix}`;

  return (
    <Button
      variant="ghost"
      size="icon"
      className={cn(
        'h-6 w-6 shrink-0 transition-colors p-0 [&_svg]:h-3.5 [&_svg]:w-3.5',
        copied && 'text-emerald-500'
      )}
      type="button"
      title={title}
      aria-label={title}
      onClick={(event) => {
        event.stopPropagation();
        void handleCopy();
      }}
    >
      {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
    </Button>
  );
}

interface HighlightedCodeProps {
  code: string;
  language: string;
  variant?: 'block' | 'inline';
  className?: string;
  showLineNumbers?: boolean;
}

function HighlightedCode({
  code,
  language,
  variant = 'block',
  className,
  showLineNumbers = true,
}: HighlightedCodeProps) {
  const inline = variant === 'inline';
  const html = inline ? useInlineHighlight(code, language) : useBlockHighlight(code, language);
  const displayClass = inline ? 'inline-block align-middle' : 'block w-full';

  if (!html) {
    if (inline) {
      return (
        <span
          className={cn(
            'font-mono whitespace-pre-wrap break-words',
            displayClass,
            className
          )}
        >
          {code || '\u00a0'}
        </span>
      );
    }

    return (
      <pre
        className={cn(
          'font-mono whitespace-pre-wrap break-words rounded-md bg-surface-inset/70 px-3 py-2 text-[10pt] leading-[1.35]',
          className
        )}
      >
        {code || '\u00a0'}
      </pre>
    );
  }

  if (!inline) {
    return (
      <div
        className={cn('shiki-block', displayClass, className)}
        dangerouslySetInnerHTML={{
          __html: showLineNumbers ? html : stripLineNumbers(html),
        }}
      />
    );
  }

  return (
    <span
      className={cn(
        'shiki-inline whitespace-pre-wrap break-words font-mono',
        displayClass,
        className
      )}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}

function useBlockHighlight(code: string, language: string): string | null {
  const themeVersion = useShikiThemeVersion();
  const [html, setHtml] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setHtml(null);

    const run = async () => {
      const safeCode = code ?? '';
      const safeLanguage = language || 'plaintext';
      try {
        const result = await highlightAgent.highlight({
          code: safeCode,
          lang: safeLanguage,
        });
        if (!cancelled) {
          setHtml(result);
        }
      } catch {
        if (!cancelled) {
          setHtml(null);
        }
      }
    };

    void run();

    return () => {
      cancelled = true;
    };
  }, [code, language, themeVersion]);

  return html;
}

function stripLineNumbers(html: string): string {
  return html
    .replace(/<span class="line-number[^"]*">[\s\S]*?<\/span>/g, '')
    .replace(/<span class="diff-line-number[^"]*">[\s\S]*?<\/span>/g, '')
    .replace(/<span data-line-number[^>]*>[\s\S]*?<\/span>/g, '')
    .replace(/(<span class="line"[^>]*?)\sdata-line-number="[^"]*"/g, '$1');
}

function useInlineHighlight(code: string, language: string): string | null {
  const themeVersion = useShikiThemeVersion();
  const [html, setHtml] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setHtml(null);

    const run = async () => {
      const safeCode = code.length > 0 ? code : ' ';
      const safeLanguage = language || 'plaintext';
      try {
        const result = await highlightAgent.highlightInline({
          code: safeCode,
          lang: safeLanguage,
        });
        if (!cancelled) {
          setHtml(result);
        }
      } catch {
        if (!cancelled) {
          setHtml(null);
        }
      }
    };

    void run();

    return () => {
      cancelled = true;
    };
  }, [code, language, themeVersion]);

  return html;
}

function useShikiThemeVersion(): number {
  return useSyncExternalStore(
    (listener) => highlightAgent.subscribe(listener),
    () => highlightAgent.getThemeVersion(),
    () => highlightAgent.getThemeVersion()
  );
}

function getLinePresentation(
  type: DisplayLine['type']
): { containerClass: string; textClass?: string; lineNumberClass: string } {
  switch (type) {
    case 'added':
      return {
        containerClass: 'bg-emerald-500/10 border-l border-emerald-500/40',
        lineNumberClass: 'text-emerald-500 dark:text-emerald-200',
      };
    case 'deleted':
      return {
        containerClass: 'bg-rose-500/10 border-l border-rose-500/40',
        lineNumberClass: 'text-rose-500 dark:text-rose-200',
      };
    case 'message':
      return {
        containerClass: 'bg-amber-500/15 border-l-2 border-amber-500',
        textClass: 'text-amber-900 dark:text-amber-200',
        lineNumberClass: 'text-amber-700 dark:text-amber-200',
      };
    case 'divider':
      return {
        containerClass: 'border-l-0',
        textClass: 'text-transparent',
        lineNumberClass: 'opacity-0',
      };
    case 'spacer':
      return {
        containerClass: 'border-l-0',
        textClass: 'text-transparent',
        lineNumberClass: 'opacity-0',
      };
    case 'context':
      return {
        containerClass: 'bg-transparent border-l-0',
        textClass: 'text-muted-foreground',
        lineNumberClass: 'text-muted-foreground/60',
      };
    case 'binary':
      return {
        containerClass: 'bg-surface-inset/50 border-l-2 border-surface-border/80',
        textClass: 'text-muted-foreground',
        lineNumberClass: 'text-muted-foreground/60',
      };
    default:
      return {
        containerClass: 'border-l-2 border-transparent',
        lineNumberClass: 'text-muted-foreground/60',
      };
  }
}

function FileBadge({ file }: { file: AnyFileChange }) {
  let label = '';
  let tone = '';

  switch (file.type) {
    case 'AddedFile':
      label = 'Added';
      tone = 'bg-emerald-500/10 text-emerald-500';
      break;
    case 'DeletedFile':
      label = 'Deleted';
      tone = 'bg-rose-500/10 text-rose-500';
      break;
    case 'RenamedFile':
      label = 'Renamed';
      tone = 'bg-blue-500/10 text-blue-500';
      break;
    default:
      label = 'Modified';
      tone = 'bg-muted text-muted-foreground';
  }

  return (
    <span className={cn('text-[11px] px-2 py-0.5 rounded-full font-medium uppercase tracking-wide', tone)}>
      {label}
    </span>
  );
}
