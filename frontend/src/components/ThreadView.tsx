import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { ChevronRight, Maximize2, Minimize2, Mail } from 'lucide-react';
import { api } from '../api/client';
import type { ThreadDetail } from '../types';
import { useTimezone } from '../contexts/TimezoneContext';
import { useMailingList } from '../contexts/MailingListContext';
import { formatDateInTimezone } from '../utils/timezone';
import { ScrollArea } from './ui/scroll-area';
import { CompactButton } from './ui/compact-button';
import { cn } from '@/lib/utils';
import { EmailBody } from './EmailBody';
import { buildPatchPreview } from '@/utils/patch';

type ThreadEmail = ThreadDetail['emails'][number];

export function ThreadView() {
  const { threadId } = useParams<{ threadId: string }>();
  const { selectedMailingList } = useMailingList();
  const { timezone } = useTimezone();
  const [threadData, setThreadData] = useState<ThreadDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [collapsedEmails, setCollapsedEmails] = useState<Set<number>>(new Set());

  useEffect(() => {
    const loadThread = async () => {
      if (!threadId || !selectedMailingList) return;
      try {
        setLoading(true);
        const data = await api.threads.get(selectedMailingList, parseInt(threadId, 10));
        setThreadData(data);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load thread');
      } finally {
        setLoading(false);
      }
    };

    loadThread();
  }, [threadId, selectedMailingList]);

  const formatDate = (dateStr: string) =>
    formatDateInTimezone(dateStr, timezone, 'MMM d, yyyy h:mm a');

  const toggleEmailCollapse = (emailId: number) => {
    setCollapsedEmails((prev) => {
      const next = new Set(prev);
      if (next.has(emailId)) {
        next.delete(emailId);
      } else {
        next.add(emailId);
      }
      return next;
    });
  };

  const collapseAll = () => {
    if (!threadData) return;
    setCollapsedEmails(new Set(threadData.emails.map((email) => email.id)));
  };

  const expandAll = () => setCollapsedEmails(new Set());

  if (loading) {
    return (
      <div className="h-full flex items-center justify-center">
        <span className="text-label">Loading thread…</span>
      </div>
    );
  }

  if (error || !threadData) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="surface-muted px-6 py-4 text-label text-danger">
          Error: {error || 'Thread not found'}
        </div>
      </div>
    );
  }

  const rootSubject = threadData.thread.subject;
  const messageCount = threadData.thread.message_count || threadData.emails.length;

  return (
    <div className="h-full flex flex-col bg-surface-base">
      <header className="sticky top-0 z-10 border-b border-border/60 bg-surface-overlay/80 backdrop-blur px-4 lg:px-6 py-4">
        <div className="flex flex-col gap-3">
          <div className="flex items-start justify-between gap-3">
            <h1 className="text-heading density-tight flex-1">{rootSubject}</h1>
            <span className="pill">{messageCount} msgs</span>
          </div>
          <div className="flex flex-wrap items-center gap-3 text-label">
            <span className="inline-flex items-center gap-1">
              <Mail className="h-3.5 w-3.5" />
              {threadData.thread.message_count || 0} messages
            </span>
            <span>Started {formatDate(threadData.thread.start_date)}</span>
            <span>Last {formatDate(threadData.thread.last_date)}</span>
            <span className="ml-auto flex gap-2">
              <CompactButton onClick={expandAll}>
                <Maximize2 className="h-3 w-3" />
                Expand
              </CompactButton>
              <CompactButton onClick={collapseAll}>
                <Minimize2 className="h-3 w-3" />
                Collapse
              </CompactButton>
            </span>
          </div>
        </div>
      </header>

      <ScrollArea className="flex-1">
        <div className="px-3 lg:px-6 py-5 space-y-3">
          {threadData.emails.length === 0 && (
            <div className="surface-muted py-12 text-center text-label">
              <Mail className="h-7 w-7 mx-auto mb-3 text-muted-foreground" />
              No messages in this thread
            </div>
          )}

          {threadData.emails.map((email) => (
            <MessagePanel
              key={email.id}
              email={email}
              rootSubject={rootSubject}
              collapsed={collapsedEmails.has(email.id)}
              onToggle={() => toggleEmailCollapse(email.id)}
              formatDate={formatDate}
            />
          ))}
        </div>
      </ScrollArea>
    </div>
  );
}

interface MessagePanelProps {
  email: ThreadEmail;
  rootSubject: string;
  collapsed: boolean;
  onToggle: () => void;
  formatDate: (date: string) => string;
}

function MessagePanel({ email, rootSubject, collapsed, onToggle, formatDate }: MessagePanelProps) {
  const depthOffset = Math.min(email.depth, 8) * 14;
  const previewText = email.body ? buildPatchPreview(email.body, email.patch_metadata) : '';

  return (
    <article style={{ marginLeft: `${depthOffset}px` }}>
      <button
        type="button"
        onClick={onToggle}
        className={cn(
          "surface w-full text-left px-4 py-3 transition-colors hover:border-accent-primary/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/40",
          collapsed ? "cursor-pointer" : "cursor-default"
        )}
        style={{ borderLeft: '3px solid var(--accent-primary)' }}
      >
        <div className="flex items-start gap-3">
          <ChevronRight
            className={cn(
              "mt-0.5 h-3.5 w-3.5 text-muted-foreground transition-transform duration-150",
              !collapsed && "rotate-90"
            )}
          />
          <div className="flex-1 min-w-0 space-y-1">
            <div className="flex flex-wrap items-center gap-2 text-sm font-semibold leading-tight text-foreground">
              <span className="truncate">
                {email.author_name || email.author_email}
              </span>
              {email.depth > 0 && (
                <span className="pill">Reply {email.depth}</span>
              )}
            </div>
            <div className="text-label">
              {email.author_email}
            </div>
            {email.subject !== rootSubject && (
              <div className="text-sm text-muted-foreground">
                {email.subject}
              </div>
            )}
            {collapsed && email.body && (
              <p className="text-sm text-muted-foreground line-clamp-2">
                {previewText || email.body}
              </p>
            )}
          </div>
          <time className="text-label whitespace-nowrap">
            {formatDate(email.date)}
          </time>
        </div>
      </button>

      {!collapsed && (
        <EmailBody
          body={email.body}
          patchType={email.patch_type}
          patchMetadata={email.patch_metadata}
        />
      )}
    </article>
  );
}
