import { useMemo, useState } from 'react';
import type { PatchMetadata, PatchType } from '@/types';
import { CompactButton } from './ui/compact-button';
import { getPatchFoldRange } from '@/utils/patch';

interface EmailBodyProps {
  body: string | null;
  patchType?: PatchType;
  patchMetadata?: PatchMetadata | null;
}

interface BodySegments {
  hasFold: boolean;
  prefix: string;
  folded: string;
  suffix: string;
  foldedLineCount: number;
}

const baseContainerClass =
  "border-l-2 border-accent-primary/50 bg-surface-muted px-5 sm:px-8 py-4 font-mono text-sm leading-relaxed";
const blockClass = "whitespace-pre-wrap break-words";

export function EmailBody({ body, patchType = 'none', patchMetadata }: EmailBodyProps) {
  if (!body || body.length === 0) {
    return (
      <div className={baseContainerClass}>
        <div className={`${blockClass} text-muted-foreground`}>(No message body)</div>
      </div>
    );
  }

  const segments = useMemo<BodySegments>(() => {
    const lines = body.split('\n');
    const fold = getPatchFoldRange(patchMetadata);

    if (!fold || lines.length === 0) {
      return {
        hasFold: false,
        prefix: body,
        folded: '',
        suffix: '',
        foldedLineCount: 0,
      };
    }

    const maxIndex = lines.length - 1;
    const start = Math.max(0, Math.min(fold.start, maxIndex));
    const end = Math.max(start, Math.min(fold.end, maxIndex));

    if (start === 0 && end === maxIndex) {
      return {
        hasFold: true,
        prefix: '',
        folded: lines.join('\n'),
        suffix: '',
        foldedLineCount: lines.length,
      };
    }

    const prefixLines = lines.slice(0, start);
    const foldedLines = lines.slice(start, end + 1);
    const suffixLines = lines.slice(end + 1);

    if (foldedLines.length === 0) {
      return {
        hasFold: false,
        prefix: body,
        folded: '',
        suffix: '',
        foldedLineCount: 0,
      };
    }

    return {
      hasFold: true,
      prefix: prefixLines.join('\n'),
      folded: foldedLines.join('\n'),
      suffix: suffixLines.join('\n'),
      foldedLineCount: foldedLines.length,
    };
  }, [body, patchMetadata]);

  const [open, setOpen] = useState(false);
  const toggle = () => setOpen((prev) => !prev);

  if (!segments.hasFold) {
    return (
      <div className={baseContainerClass}>
        <pre className={blockClass}>{segments.prefix}</pre>
      </div>
    );
  }

  const summaryLabel =
    patchType === 'attachment' ? 'Patch attachment detected' : 'Patch content detected';

  return (
    <div className={`${baseContainerClass} space-y-4`}>
      {segments.prefix && <pre className={blockClass}>{segments.prefix}</pre>}

      <div className="rounded border border-border/60 bg-surface-base/90">
        <div className="flex flex-wrap items-center justify-between gap-2 px-3 py-2 text-[11px] uppercase tracking-[0.08em] text-muted-foreground">
          <span>{summaryLabel}</span>
          <CompactButton onClick={toggle}>
            {open ? 'Hide patch' : `Show patch (${segments.foldedLineCount} lines)`}
          </CompactButton>
        </div>
        {open && (
          <div className="border-t border-border/60 px-3 py-3">
            <pre className={blockClass}>{segments.folded}</pre>
          </div>
        )}
      </div>

      {segments.suffix && <pre className={blockClass}>{segments.suffix}</pre>}
    </div>
  );
}
