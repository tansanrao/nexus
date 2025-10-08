import { useEffect, useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import { api } from '../api/client';
import type { AuthorWithStats, Thread, ThreadWithStarter } from '../types';
import { useTimezone } from '../contexts/TimezoneContext';
import { formatDateInTimezone, formatDateCompact } from '../utils/timezone';
import { Button } from './ui/button';
import { Card } from './ui/card';
import { Badge } from './ui/badge';
import { ScrollArea } from './ui/scroll-area';
import { Avatar, AvatarFallback } from './ui/avatar';
import { cn } from '@/lib/utils';

type ThreadTabType = 'started' | 'participated';

export function AuthorDetailMiddle() {
  const { authorId, mailingList, threadId } = useParams<{ authorId: string; mailingList: string; threadId: string }>();
  const { timezone } = useTimezone();
  const [author, setAuthor] = useState<AuthorWithStats | null>(null);
  const [activeTab, setActiveTab] = useState<ThreadTabType>('started');
  const [threadsStarted, setThreadsStarted] = useState<ThreadWithStarter[]>([]);
  const [threadsParticipated, setThreadsParticipated] = useState<Thread[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const limit = 20;

  useEffect(() => {
    const loadAuthorData = async () => {
      if (!authorId || !mailingList) return;

      try {
        setLoading(true);
        const authorData = await api.authors.get(mailingList, parseInt(authorId));
        setAuthor(authorData);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load author');
      } finally {
        setLoading(false);
      }
    };

    loadAuthorData();
  }, [authorId, mailingList]);

  useEffect(() => {
    const loadTabData = async () => {
      if (!authorId || !mailingList) return;

      try {
        setLoading(true);
        if (activeTab === 'started') {
          const threadsData = await api.authors.getThreadsStarted(mailingList, parseInt(authorId), page, limit);
          setThreadsStarted(threadsData);
        } else if (activeTab === 'participated') {
          const threadsData = await api.authors.getThreadsParticipated(mailingList, parseInt(authorId), page, limit);
          setThreadsParticipated(threadsData);
        }
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load data');
      } finally {
        setLoading(false);
      }
    };

    loadTabData();
  }, [authorId, page, activeTab, mailingList]);

  const formatDate = (dateStr: string | null) => {
    if (!dateStr) return 'N/A';
    return formatDateInTimezone(dateStr, timezone, 'MMM d, yyyy');
  };

  const getInitials = (name: string | null | undefined, email: string) => {
    if (name) {
      return name.split(' ').map(n => n[0]).slice(0, 2).join('').toUpperCase();
    }
    return email.substring(0, 2).toUpperCase();
  };

  const getDisplayName = (author: AuthorWithStats) => {
    return author.canonical_name || author.name_variations[0] || author.email;
  };

  const getCurrentThreads = () => {
    return activeTab === 'started' ? threadsStarted : threadsParticipated;
  };

  if (loading && !author) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-sm text-muted-foreground">Loading author...</div>
      </div>
    );
  }

  if (error || !author) {
    return (
      <div className="h-full flex items-center justify-center p-4">
        <Card className="p-6">
          <div className="text-sm text-destructive">Error: {error || 'Author not found'}</div>
        </Card>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Author metadata card */}
      <div className="border-b p-4">
        <Card className="p-4">
          <div className="flex items-start gap-3 mb-3">
            <Avatar className="h-12 w-12">
              <AvatarFallback className="text-sm font-medium">
                {getInitials(author.canonical_name, author.email)}
              </AvatarFallback>
            </Avatar>
            <div className="flex-1 min-w-0">
              <h2 className="text-base font-semibold truncate">
                {getDisplayName(author)}
              </h2>
              <div className="text-xs text-muted-foreground truncate">{author.email}</div>
            </div>
          </div>
          <div className="flex gap-2 flex-wrap mb-2">
            <Badge variant="secondary" className="text-xs">
              {author.email_count} emails
            </Badge>
            <Badge variant="outline" className="text-xs">
              {author.thread_count} threads
            </Badge>
          </div>
          {/* Show mailing lists */}
          {author.mailing_lists && author.mailing_lists.length > 0 && (
            <div className="mb-2">
              <div className="text-xs text-muted-foreground mb-1">Active in:</div>
              <div className="flex gap-1 flex-wrap">
                {author.mailing_lists.map((ml) => (
                  <Badge key={ml} variant="outline" className="text-xs">
                    {ml}
                  </Badge>
                ))}
              </div>
            </div>
          )}
          {/* Show name variations */}
          {author.name_variations && author.name_variations.length > 1 && (
            <div className="mb-2">
              <div className="text-xs text-muted-foreground mb-1">Also known as:</div>
              <div className="text-xs text-muted-foreground">
                {author.name_variations.slice(1, 4).join(', ')}
                {author.name_variations.length > 4 && ` +${author.name_variations.length - 4} more`}
              </div>
            </div>
          )}
          <div className="text-xs text-muted-foreground">
            Active: {formatDate(author.first_email_date)} – {formatDate(author.last_email_date)}
          </div>
        </Card>
      </div>

      {/* Tab selector */}
      <div className="border-b p-2 flex gap-1">
        <Button
          variant={activeTab === 'started' ? 'secondary' : 'ghost'}
          size="sm"
          onClick={() => { setActiveTab('started'); setPage(1); }}
          className="flex-1"
        >
          Threads Started
        </Button>
        <Button
          variant={activeTab === 'participated' ? 'secondary' : 'ghost'}
          size="sm"
          onClick={() => { setActiveTab('participated'); setPage(1); }}
          className="flex-1"
        >
          Participated
        </Button>
      </div>

      {/* Thread list */}
      <ScrollArea className="flex-1">
        <div className="p-2">
          {loading && getCurrentThreads().length === 0 ? (
            <div className="p-8 text-center">
              <div className="text-xs text-muted-foreground">Loading threads...</div>
            </div>
          ) : getCurrentThreads().length === 0 ? (
            <div className="p-8 text-center">
              <div className="text-xs text-muted-foreground">No threads found</div>
            </div>
          ) : (
            <div className="space-y-1">
              {getCurrentThreads().map((thread) => {
                const isSelected = threadId === String(thread.id);
                return (
                  <Link
                    key={thread.id}
                    to={`/${mailingList}/authors/${authorId}/threads/${thread.id}`}
                    className="block"
                  >
                    <div
                      className={cn(
                        "p-3 rounded-md transition-colors hover:bg-accent",
                        isSelected && "bg-accent"
                      )}
                    >
                      <div className="text-sm font-medium line-clamp-2 mb-1">
                        {thread.subject}
                      </div>
                      <div className="flex items-center gap-2 text-xs text-muted-foreground">
                        <Badge variant="secondary" className="text-xs px-1 py-0">
                          {thread.message_count || 0}
                        </Badge>
                        <span className="truncate">{formatDateCompact(thread.last_date, timezone)}</span>
                      </div>
                    </div>
                  </Link>
                );
              })}
            </div>
          )}
        </div>
      </ScrollArea>

      {/* Pagination */}
      <div className="border-t p-2">
        <div className="flex justify-between items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setPage((p) => Math.max(1, p - 1))}
            disabled={page === 1}
          >
            Prev
          </Button>
          <span className="text-xs text-muted-foreground">Page {page}</span>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setPage((p) => p + 1)}
            disabled={getCurrentThreads().length < limit}
          >
            Next
          </Button>
        </div>
      </div>
    </div>
  );
}
