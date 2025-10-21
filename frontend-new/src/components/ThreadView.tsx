import { useCallback, useEffect, useMemo, useState } from 'react';
import { ChevronDown, ChevronUp, List, ListTree } from 'lucide-react';
import { ScrollArea } from './ui/scroll-area';
import { EmailItem } from './EmailItem';
import type { EmailHierarchy, ThreadDetail } from '../types';
import { Button } from './ui/button';
import { useTimezone } from '../contexts/TimezoneContext';
import { formatDateInTimezone } from '../utils/timezone';

const EMPTY_EMAIL_LIST: EmailHierarchy[] = [];

interface ThreadViewProps {
  threadId: number | null;
  threadDetail: ThreadDetail | null;
  loading: boolean;
  error: string | null;
}

export function ThreadView({ threadId, threadDetail, loading, error }: ThreadViewProps) {
  const { timezone } = useTimezone();
  const [collapsedEmailIds, setCollapsedEmailIds] = useState<Set<number>>(new Set());
  const [hideDeepCollapsedReplies, setHideDeepCollapsedReplies] = useState(true);

  useEffect(() => {
    setCollapsedEmailIds(new Set());
  }, [threadId]);

  // Always derive emails to keep hook order stable across renders
  const emails = threadDetail?.emails ?? EMPTY_EMAIL_LIST;

  // Precompute descendant reply counts for each email based on depth
  const hiddenRepliesByEmailId = useMemo(() => {
    const counts = new Map<number, number>();
    for (let i = 0; i < emails.length; i++) {
      const current = emails[i];
      const currentDepth = current.depth;
      let descendantCount = 0;
      for (let j = i + 1; j < emails.length; j++) {
        const next = emails[j];
        if (next.depth <= currentDepth) break;
        descendantCount += 1;
      }
      counts.set(current.id, descendantCount);
    }
    return counts;
  }, [emails]);

  const handleCollapsedChange = useCallback((emailId: number, collapsed: boolean) => {
    setCollapsedEmailIds((prev) => {
      const next = new Set(prev);
      if (collapsed) {
        next.add(emailId);
      } else {
        next.delete(emailId);
      }
      return next;
    });
  }, []);

  const collapseAll = useCallback(() => {
    setCollapsedEmailIds(new Set(emails.map((email) => email.id)));
  }, [emails]);

  const expandAll = useCallback(() => {
    setCollapsedEmailIds(new Set());
  }, []);

  const emailsWithState = useMemo(() => {
    if (!hideDeepCollapsedReplies) {
      return emails.map((email) => ({
        email,
        isCollapsed: collapsedEmailIds.has(email.id),
        isHidden: false,
      }));
    }

    const result: Array<{
      email: EmailHierarchy;
      isCollapsed: boolean;
      isHidden: boolean;
    }> = [];
    const collapsedStack: number[] = [];

    for (const email of emails) {
      while (
        collapsedStack.length > 0 &&
        email.depth <= collapsedStack[collapsedStack.length - 1]
      ) {
        collapsedStack.pop();
      }

      const isCollapsed = collapsedEmailIds.has(email.id);
      const hasCollapsedAncestor = collapsedStack.length > 0;
      const isHidden = hasCollapsedAncestor && email.depth > 1;

      result.push({ email, isCollapsed, isHidden });

      if (isCollapsed && email.depth >= 1) {
        collapsedStack.push(email.depth);
      }
    }

    return result;
  }, [emails, collapsedEmailIds, hideDeepCollapsedReplies]);

  const toggleCollapseMode = useCallback(() => {
    setHideDeepCollapsedReplies((prev) => !prev);
  }, []);

  if (!threadId) {
    return (
      <div className="flex items-center justify-center h-full p-8 bg-surface-inset">
        <p className="text-muted-foreground text-sm text-center">
          Select a thread to view its details
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
        {Array.from({ length: 5 }).map((_, i) => (
          <div key={i} className="py-3 border-l-2 border-transparent pl-3 animate-pulse">
            <div className="h-4 bg-muted rounded w-1/4 mb-2"></div>
            <div className="h-20 bg-muted/60 rounded w-full"></div>
          </div>
        ))}
      </div>
    );
  }

  if (error || !threadDetail) {
    return (
      <div className="flex items-center justify-center h-full p-8 bg-surface-inset">
        <p className="text-destructive text-sm">{error || 'Thread not found'}</p>
      </div>
    );
  }

  const { thread } = threadDetail;
  const formatDate = (value: string) =>
    formatDateInTimezone(value, timezone, { month: 'short', day: 'numeric', year: 'numeric' });

  return (
    <ScrollArea className="h-full bg-surface-inset min-w-0"
    style={{ backgroundColor: 'hsl(var(--color-panel-right))' }}>
      <div className="p-6">
        {/* Thread header */}
        <div className="border-b border-surface-border/60 pb-3 mb-4">
          <h1
            className="text-lg font-semibold text-foreground mb-2 leading-tight break-words max-w-[min(48rem,100%)]"
            title={thread.subject}
          >
            {thread.subject}
          </h1>
          <div className="flex items-center gap-2 text-xs text-muted-foreground flex-wrap">
            <span>{thread.message_count || 0} {thread.message_count === 1 ? 'message' : 'messages'}</span>
            <span>•</span>
            <span>Started {formatDate(thread.start_date)}</span>
            {thread.last_date !== thread.start_date && (
              <>
                <span>•</span>
                <span>Last activity {formatDate(thread.last_date)}</span>
              </>
            )}
            <span>•</span>
            <div className="flex items-center gap-0.5">
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6"
                onClick={expandAll}
                title="Expand all"
              >
                <ChevronDown className="h-3 w-3" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6"
                onClick={collapseAll}
                title="Collapse all"
              >
                <ChevronUp className="h-3 w-3" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                className={`h-6 w-6 ${hideDeepCollapsedReplies ? 'bg-muted text-foreground' : ''}`}
                onClick={toggleCollapseMode}
                title={
                  hideDeepCollapsedReplies
                    ? 'Collapsed replies hide deeper messages'
                    : 'Collapsed replies show message headers'
                }
                aria-pressed={hideDeepCollapsedReplies}
              >
                {hideDeepCollapsedReplies ? (
                  <ListTree className="h-3 w-3" />
                ) : (
                  <List className="h-3 w-3" />
                )}
              </Button>
            </div>
          </div>
        </div>

        {/* Emails */}
        <div className="space-y-0">
          {emailsWithState.map(({ email, isCollapsed, isHidden }) => (
            <EmailItem
              key={email.id}
              email={email}
              isCollapsed={isCollapsed}
              onCollapsedChange={(next) => handleCollapsedChange(email.id, next)}
              hiddenReplyCount={hiddenRepliesByEmailId.get(email.id) || 0}
              isHidden={isHidden}
            />
          ))}
        </div>
      </div>
    </ScrollArea>
  );
}
