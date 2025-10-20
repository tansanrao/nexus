import { useEffect, useMemo, useState } from 'react';
import { cn } from '../lib/utils';

interface EmailBodyProps {
  body: string;
}

type QuoteSegment =
  | { type: 'text'; lines: string[] }
  | { type: 'quote'; node: QuoteNode };

interface QuoteNode {
  depth: number;
  segments: QuoteSegment[];
}

const INDENT_PER_DEPTH = 12;

export function EmailBody({ body }: EmailBodyProps) {
  const parsed = useMemo(() => parseQuotedBody(body), [body]);

  if (!body.trim()) {
    return null;
  }

  return (
    <div className="text-sm whitespace-pre-wrap break-words font-mono text-foreground leading-relaxed overflow-x-auto bg-surface-inset/70 p-3 max-w-full min-w-0">
      <QuoteNodeRenderer node={parsed} />
    </div>
  );
}

function QuoteNodeRenderer({ node }: { node: QuoteNode }) {
  let hasRenderedContent = false;

  return (
    <>
      {node.segments.map((segment, index) => {
        if (segment.type === 'text') {
          const isLast = index === node.segments.length - 1;
          let displayLines = segment.lines.map((line) =>
            node.depth > 0 ? stripQuotePrefixForDepth(line, node.depth) : line
          );
          while (!hasRenderedContent && displayLines.length > 0 && displayLines[0].trim() === '') {
            displayLines.shift();
          }
          if (displayLines.length === 0) {
            return null;
          }
          hasRenderedContent = true;
          let text = displayLines.join('\n');
          if (!isLast) {
            text += '\n';
          }
          return (
            <span key={index} className="whitespace-pre-wrap">
              {text}
            </span>
          );
        }

        hasRenderedContent = true;
        return <QuoteBlock key={index} node={segment.node} />;
      })}
    </>
  );
}

function QuoteBlock({ node }: { node: QuoteNode }) {
  const defaultCollapsed = node.depth > 1;
  const [collapsed, setCollapsed] = useState(defaultCollapsed);

  useEffect(() => {
    setCollapsed(defaultCollapsed);
  }, [defaultCollapsed, node]);

  const preview = useMemo(() => getQuotePreview(node), [node]);
  const lineCount = useMemo(() => countQuoteLines(node), [node]);
  const previewText = preview || 'Quoted text';
  const collapsedLabel =
    lineCount > 0
      ? `${previewText} [${lineCount} ${lineCount === 1 ? 'line' : 'lines'} hidden]`
      : previewText;

  return (
    <div
      className="mb-1"
      style={{ marginLeft: `${Math.max(0, node.depth - 1) * INDENT_PER_DEPTH}px` }}
    >
      <div
        className={cn(
          'relative text-muted-foreground min-w-0',
          !collapsed && 'pl-3'
        )}
      >
        {!collapsed && (
          <span
            aria-hidden="true"
            className="pointer-events-none absolute left-0 top-5 bottom-0 w-px bg-muted-foreground/30"
          />
        )}
        <div className="flex items-start gap-2 min-w-0">
          <button
            type="button"
            className={cn(
              'relative inline-flex h-5 w-5 items-center justify-center text-sm font-mono leading-relaxed text-muted-foreground hover:text-foreground focus:outline-none focus-visible:ring-1 focus-visible:ring-offset-0 focus-visible:ring-primary transition-colors',
              !collapsed && '-ml-3 -translate-x-1/2'
            )}
            aria-expanded={!collapsed}
            aria-label={collapsed ? 'Expand quoted text' : 'Collapse quoted text'}
            title={collapsed ? 'Expand quoted text' : 'Collapse quoted text'}
            onClick={() => setCollapsed((value) => !value)}
          >
            [{collapsed ? '+' : '-'}]
          </button>
          <div
            className={cn(
              'flex-1 min-w-0 whitespace-pre-wrap text-sm leading-relaxed',
              collapsed ? 'text-muted-foreground/60 opacity-80' : 'pl-3'
            )}
          >
            {collapsed ? (
              <span>{collapsedLabel}</span>
            ) : (
              <QuoteNodeRenderer node={node} />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function countQuoteLines(node: QuoteNode): number {
  let count = 0;

  for (const segment of node.segments) {
    if (segment.type === 'text') {
      count += segment.lines.length;
    } else {
      count += countQuoteLines(segment.node);
    }
  }

  return count;
}

function getQuotePreview(node: QuoteNode): string {
  for (const segment of node.segments) {
    if (segment.type === 'text') {
      for (const line of segment.lines) {
        const stripped = stripQuotePrefixForDepth(line, node.depth).trim();
        if (stripped) {
          return truncatedPreview(stripped);
        }
      }
    } else {
      const nestedPreview = getQuotePreview(segment.node);
      if (nestedPreview) {
        return truncatedPreview(nestedPreview);
      }
    }
  }

  return '';
}

function truncatedPreview(text: string, maxLength = 120): string {
  if (text.length <= maxLength) {
    return text;
  }

  return `${text.slice(0, maxLength).trimEnd()}…`;
}

function stripQuotePrefixForDepth(line: string, depth: number): string {
  if (depth <= 0) {
    return line;
  }

  const pattern = new RegExp(`^(\\s*>\\s*){${depth}}`);
  return line.replace(pattern, '');
}

function parseQuotedBody(body: string): QuoteNode {
  const normalized = body.replace(/\r\n/g, '\n');
  const lines = normalized.split('\n');
  const root: QuoteNode = { depth: 0, segments: [] };
  const stack: QuoteNode[] = [root];

  for (const line of lines) {
    const depth = getQuoteDepth(line);

    while (stack.length > depth + 1) {
      stack.pop();
    }

    while (stack.length < depth + 1) {
      const parent = stack[stack.length - 1];
      const newNode: QuoteNode = { depth: parent.depth + 1, segments: [] };
      parent.segments.push({ type: 'quote', node: newNode });
      stack.push(newNode);
    }

    const current = stack[stack.length - 1];
    addLineToNode(current, line);
  }

  return root;
}

function addLineToNode(node: QuoteNode, line: string) {
  const last = node.segments[node.segments.length - 1];

  if (last && last.type === 'text') {
    last.lines.push(line);
  } else {
    node.segments.push({ type: 'text', lines: [line] });
  }
}

function getQuoteDepth(line: string): number {
  let depth = 0;

  for (let i = 0; i < line.length; i++) {
    const char = line[i];

    if (char === '>') {
      depth += 1;
      continue;
    }

    if (char === ' ') {
      continue;
    }

    break;
  }

  return depth;
}
