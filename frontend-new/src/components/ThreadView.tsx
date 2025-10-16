import { useEffect, useMemo, useState } from 'react';
import { ChevronDown, ChevronUp } from 'lucide-react';
import { ScrollArea } from './ui/scroll-area';
import { EmailItem } from './EmailItem';
import { apiClient } from '../lib/api';
import type { ThreadDetail } from '../types';
import { formatDate } from '../utils/date';
import { useApiConfig } from '../contexts/ApiConfigContext';
import { Button } from './ui/button';

interface ThreadViewProps {
  threadId: number | null;
}

export function ThreadView({ threadId }: ThreadViewProps) {
  const { selectedMailingList } = useApiConfig();
  const [threadDetail, setThreadDetail] = useState<ThreadDetail | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [allCollapsed, setAllCollapsed] = useState(false);

  useEffect(() => {
    if (threadId && selectedMailingList) {
      loadThreadDetail(threadId);
    }
  }, [threadId, selectedMailingList]);


  const loadThreadDetail = async (id: number) => {
    if (!selectedMailingList) return;
    
    setLoading(true);
    setError(null);
    try {
      const detail = await apiClient.getThread(selectedMailingList, id);
      setThreadDetail(detail);
    } catch (err) {
      setError('Failed to load thread details');
      console.error('Error loading thread:', err);
    } finally {
      setLoading(false);
    }
  };

  // Always derive emails to keep hook order stable across renders
  const emails = threadDetail?.emails ?? [];

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

  if (!threadId) {
    return (
      <div className="flex items-center justify-center h-full p-8">
        <p className="text-muted-foreground text-sm">
          Select a thread to view its details
        </p>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="p-4 space-y-4">
        <div className="border-b pb-3 animate-pulse">
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
      <div className="flex items-center justify-center h-full p-8">
        <p className="text-destructive text-sm">{error || 'Thread not found'}</p>
      </div>
    );
  }

  const { thread } = threadDetail;

  return (
    <ScrollArea className="h-full">
      <div className="p-4">
        {/* Thread header */}
        <div className="border-b pb-3 mb-4">
          <h1 className="text-xl font-bold text-foreground truncate mb-2">{thread.subject}</h1>
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
                onClick={() => setAllCollapsed(false)}
                title="Expand all"
              >
                <ChevronDown className="h-3 w-3" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6"
                onClick={() => setAllCollapsed(true)}
                title="Collapse all"
              >
                <ChevronUp className="h-3 w-3" />
              </Button>
            </div>
          </div>
        </div>

        {/* Emails */}
        <div className="space-y-0">
          {emails.map((email) => (
            <EmailItem
              key={email.id}
              email={email}
              forceCollapsed={allCollapsed}
              hiddenReplyCount={hiddenRepliesByEmailId.get(email.id) || 0}
            />
          ))}
        </div>
      </div>
    </ScrollArea>
  );
}

