import { useMemo } from 'react';
import { ScrollArea } from './ui/scroll-area';
import type { ThreadDetail, EmailHierarchy } from '../types';
import { GitDiffViewer } from './GitDiffViewer';
import { extractDiffContent } from '../utils/diff';
import { useTimezone } from '../contexts/timezone-context';
import { formatDateInTimezone } from '../utils/timezone';

interface ThreadDiffViewProps {
  threadId: number | null;
  threadDetail: ThreadDetail | null;
  loading: boolean;
  error: string | null;
}

export function ThreadDiffView({ threadId, threadDetail, loading, error }: ThreadDiffViewProps) {
  const { timezone } = useTimezone();
  const { aggregatedDiff, includedEmails } = useMemo(() => {
    if (!threadDetail) {
      return { aggregatedDiff: '', includedEmails: [] as EmailHierarchy[] };
    }

    const emailDiffs = threadDetail.emails
      .map((email) => ({
        email,
        diff: extractDiffContent(email.body, email.patch_metadata),
      }))
      .filter(({ diff }) => diff && diff.trim().length > 0);

    if (emailDiffs.length === 0) {
      return { aggregatedDiff: '', includedEmails: [] as EmailHierarchy[] };
    }

    const cleanedDiffs = emailDiffs.map(({ diff }) => diff.trimEnd());

    return {
      aggregatedDiff: cleanedDiffs.join('\n\n'),
      includedEmails: emailDiffs.map(({ email }) => email),
    };
  }, [threadDetail]);

  if (!threadId) {
    return (
      <div className="flex items-center justify-center h-full p-8 bg-surface-inset">
        <p className="text-muted-foreground text-sm text-center">
          Select a thread to view its aggregated diffs
        </p>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="p-6 space-y-4 bg-surface-inset h-full">
        <div className="border-b border-surface-border/60 pb-3 animate-pulse">
          <div className="h-6 bg-muted rounded w-3/4 mb-2"></div>
          <div className="h-4 bg-muted/60 rounded w-1/2"></div>
        </div>
        {Array.from({ length: 4 }).map((_, i) => (
          <div key={i} className="border border-dashed border-surface-border/40 rounded-md p-4 animate-pulse bg-surface">
            <div className="h-4 bg-muted rounded w-1/3 mb-2"></div>
            <div className="h-3 bg-muted/70 rounded w-full"></div>
          </div>
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-full p-8 bg-surface-inset">
        <p className="text-destructive text-sm">{error}</p>
      </div>
    );
  }

  if (!threadDetail) {
    return (
      <div className="flex items-center justify-center h-full p-8 bg-surface-inset">
        <p className="text-muted-foreground text-sm">Thread details unavailable</p>
      </div>
    );
  }

  const { thread } = threadDetail;
  const formatDate = (value: string) =>
    formatDateInTimezone(value, timezone, { month: 'short', day: 'numeric', year: 'numeric' });

  return (
    <ScrollArea
      className="h-full bg-surface-inset min-w-0"
      style={{ backgroundColor: 'hsl(var(--color-panel-right))' }}
    >
      <div className="p-6 space-y-4">
        <div className="border-b border-surface-border/60 pb-3">
          <h1
            className="text-lg font-semibold text-foreground mb-2 leading-tight break-words max-w-[min(48rem,100%)]"
            title={thread.subject}
          >
            {thread.subject}
          </h1>
          <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
            <span>{thread.message_count || 0} {thread.message_count === 1 ? 'message' : 'messages'}</span>
            <span>•</span>
            <span>Started {formatDate(thread.start_date)}</span>
            {thread.last_date !== thread.start_date && (
              <>
                <span>•</span>
                <span>Last activity {formatDate(thread.last_date)}</span>
              </>
            )}
          </div>
          <p className="text-xs text-muted-foreground mt-2">
            Showing all patches from this thread combined into a single diff view grouped by file.
          </p>
          {includedEmails.length > 0 && (
            <div className="mt-2 space-y-1 text-xs text-muted-foreground">
              <p className="font-medium text-foreground/80">Included patches</p>
              <ul className="list-disc list-inside space-y-0.5">
                {includedEmails.map((email) => (
                  <li key={email.id}>
                    {email.subject || email.message_id}
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>

        {aggregatedDiff.trim().length === 0 ? (
          <div className="border border-dashed border-surface-border/50 rounded-md bg-surface p-6 text-center">
            <p className="text-sm text-muted-foreground">
              No diff content detected across emails in this thread.
            </p>
          </div>
        ) : (
          <GitDiffViewer
            emailBody={`${aggregatedDiff}\n`}
            patchMetadata={null}
            defaultExpanded
          />
        )}
      </div>
    </ScrollArea>
  );
}
