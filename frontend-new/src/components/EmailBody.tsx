import {
  useEffect,
  useMemo,
  useState,
  type KeyboardEvent as ReactKeyboardEvent,
  type MouseEvent as ReactMouseEvent,
} from 'react';
import { cn } from '../lib/utils';

interface EmailBodyProps {
  body: string;
}

interface FormattedLine {
  depth: number;
  text: string;
}

type QuoteSegment =
  | { type: 'text'; lines: string[] }
  | { type: 'quote'; node: QuoteNode };

interface QuoteNode {
  depth: number;
  segments: QuoteSegment[];
}

const QUOTE_STYLES = [
  { marker: 'bg-sky-500/80', text: 'text-sky-500 dark:text-sky-300' },
  { marker: 'bg-emerald-500/80', text: 'text-emerald-500 dark:text-emerald-300' },
  { marker: 'bg-amber-500/80', text: 'text-amber-600 dark:text-amber-300' },
  { marker: 'bg-fuchsia-500/80', text: 'text-fuchsia-500 dark:text-fuchsia-300' },
  { marker: 'bg-purple-500/80', text: 'text-purple-500 dark:text-purple-300' },
];

export function EmailBody({ body }: EmailBodyProps) {
  const parsed = useMemo(() => parseQuotedBody(body), [body]);

  if (!body.trim()) {
    return null;
  }

  return (
    <div className="text-sm font-mono text-foreground leading-relaxed overflow-x-auto bg-surface-inset/70 p-3 max-w-full min-w-0">
      <div className="flex flex-col gap-0">
        <QuoteNodeRenderer node={parsed} />
      </div>
    </div>
  );
}

function QuoteNodeRenderer({ node }: { node: QuoteNode }) {
  let hasRenderedContent = false;

  return (
    <>
      {node.segments.map((segment, index) => {
        if (segment.type === 'text') {
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
          return <QuoteText key={index} lines={displayLines} depth={node.depth} />;
        }

        hasRenderedContent = true;
        return <QuoteBlock key={index} node={segment.node} />;
      })}
    </>
  );
}

function QuoteText({ lines, depth }: { lines: string[]; depth: number }) {
  return (
    <>
      {lines.map((text, index) => (
        <EmailLine key={index} line={{ depth, text }} />
      ))}
    </>
  );
}

function QuoteBlock({ node }: { node: QuoteNode }) {
  const [collapsed, setCollapsed] = useState(false);
  const lineCount = useMemo(() => countQuoteLines(node), [node]);

  useEffect(() => {
    setCollapsed(false);
  }, [node]);

  const toggleCollapsed = (
    event: ReactMouseEvent<HTMLDivElement> | ReactKeyboardEvent<HTMLDivElement>
  ) => {
    const target = event.target as HTMLElement;

    if (target.closest('a')) {
      event.stopPropagation();
      return;
    }

    const selection =
      typeof window !== 'undefined' ? window.getSelection() : null;

    if (selection && selection.toString()) {
      event.stopPropagation();
      return;
    }

    event.stopPropagation();
    setCollapsed((value) => !value);
  };

  const handleKeyDown = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      toggleCollapsed(event);
    }
  };

  return (
    <div
      data-quote-block
      className="flex flex-col gap-0 cursor-pointer rounded-sm focus:outline-none focus-visible:ring-1 focus-visible:ring-offset-0 focus-visible:ring-primary/60"
      onClick={toggleCollapsed}
      role="button"
      tabIndex={0}
      onKeyDown={handleKeyDown}
      aria-expanded={!collapsed}
      aria-label={collapsed ? 'Expand quoted text' : 'Collapse quoted text'}
    >
      {collapsed ? (
        <EmailLine
          line={{
            depth: node.depth,
            text: `[${lineCount} ${lineCount === 1 ? 'line' : 'lines'} hidden]`,
          }}
        />
      ) : (
        <div className="flex flex-col gap-0">
          <QuoteNodeRenderer node={node} />
        </div>
      )}
    </div>
  );
}

function EmailLine({ line }: { line: FormattedLine }) {
  const content = line.text === '' ? '\u00A0' : line.text;
  const depthStyle =
    line.depth > 0 ? QUOTE_STYLES[(line.depth - 1) % QUOTE_STYLES.length] : null;

  return (
    <div className="flex items-stretch gap-2">
      {line.depth > 0 && (
        <div className="flex gap-[2px] pr-1">
          {Array.from({ length: line.depth }).map((_, index) => {
            const style = QUOTE_STYLES[index % QUOTE_STYLES.length];
            return (
              <span
                key={index}
                className={cn('w-[3px] self-stretch rounded-none', style.marker)}
              />
            );
          })}
        </div>
      )}
      <div
        className={cn(
          'flex-1 whitespace-pre-wrap break-words leading-relaxed',
          depthStyle ? ['pl-1', depthStyle.text] : 'text-foreground'
        )}
      >
        {content}
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
