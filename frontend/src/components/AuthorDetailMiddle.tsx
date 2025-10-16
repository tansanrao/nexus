import { useEffect, useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import { Calendar } from 'lucide-react';
import { api } from '../api/client';
import type {
  AuthorWithStats,
  Thread,
  ThreadWithStarter,
  PaginatedResponse,
  PageMetadata,
} from '../types';
import { useTimezone } from '../contexts/TimezoneContext';
import { useMailingList } from '../contexts/MailingListContext';
import { formatDateInTimezone, formatDateCompact } from '../utils/timezone';
import { Button } from './ui/button';
import { Card } from './ui/card';
import { Badge } from './ui/badge';
import { ScrollArea } from './ui/scroll-area';
import { Avatar, AvatarFallback } from './ui/avatar';
import { cn } from '@/lib/utils';

type ThreadTabType = 'started' | 'participated';

export function AuthorDetailMiddle() {
  const { authorId, threadId } = useParams<{ authorId: string; threadId: string }>();
  const { selectedMailingList } = useMailingList();
  const { timezone } = useTimezone();
  const [author, setAuthor] = useState<AuthorWithStats | null>(null);
  const [activeTab, setActiveTab] = useState<ThreadTabType>('started');
  const [threadsStarted, setThreadsStarted] = useState<ThreadWithStarter[]>([]);
  const [threadsParticipated, setThreadsParticipated] = useState<Thread[]>([]);
  const [threadsStartedPageInfo, setThreadsStartedPageInfo] = useState<PageMetadata | null>(null);
  const [threadsParticipatedPageInfo, setThreadsParticipatedPageInfo] = useState<PageMetadata | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const size = 20;

  useEffect(() => {
    const loadAuthorData = async () => {
      if (!authorId || !selectedMailingList) return;

      try {
        setLoading(true);
        const authorData = await api.authors.get(selectedMailingList, parseInt(authorId));
        setAuthor(authorData);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load author');
      } finally {
        setLoading(false);
      }
    };

    loadAuthorData();
  }, [authorId, selectedMailingList]);

  useEffect(() => {
    const loadTabData = async () => {
      if (!authorId || !selectedMailingList) return;

      try {
        setLoading(true);
        if (activeTab === 'started') {
          const threadsData: PaginatedResponse<ThreadWithStarter> = await api.authors.getThreadsStarted(
            selectedMailingList,
            parseInt(authorId),
            page,
            size,
          );
          setThreadsStarted(threadsData.data);
          setThreadsStartedPageInfo(threadsData.page);
        } else if (activeTab === 'participated') {
          const threadsData: PaginatedResponse<Thread> = await api.authors.getThreadsParticipated(
            selectedMailingList,
            parseInt(authorId),
            page,
            size,
          );
          setThreadsParticipated(threadsData.data);
          setThreadsParticipatedPageInfo(threadsData.page);
        }
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load data');
        if (activeTab === 'started') {
          setThreadsStartedPageInfo(null);
        } else if (activeTab === 'participated') {
          setThreadsParticipatedPageInfo(null);
        }
      } finally {
        setLoading(false);
      }
    };

    loadTabData();
  }, [authorId, page, activeTab, selectedMailingList]);

  useEffect(() => {
    setPage(1);
  }, [authorId, selectedMailingList, activeTab]);

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

  const getCurrentPageInfo = () => {
    return activeTab === 'started' ? threadsStartedPageInfo : threadsParticipatedPageInfo;
  };

  const currentThreads = getCurrentThreads();
  const currentPageInfo = getCurrentPageInfo();

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
      {/* Author metadata card - Enhanced */}
      <div className="border-b p-4 bg-gradient-to-b from-card to-card/50">
        <Card className="p-5 shadow-lg">
          {/* Avatar & Header */}
          <div className="flex items-start gap-4 mb-4">
            <Avatar className="h-16 w-16 ring-4 ring-primary/10 flex-shrink-0">
              <AvatarFallback className="text-lg font-bold bg-primary/20 text-primary">
                {getInitials(author.canonical_name, author.email)}
              </AvatarFallback>
            </Avatar>

            <div className="flex-1 min-w-0">
              <h2 className="text-xl font-bold mb-1">
                {getDisplayName(author)}
              </h2>
              <p className="text-sm text-muted-foreground truncate">
                {author.email}
              </p>
            </div>
          </div>

          {/* Stats Grid */}
          <div className="grid grid-cols-2 gap-3 mb-4">
            <div className="text-center p-3 rounded-lg bg-primary/5">
              <div className="text-2xl font-bold text-primary">
                {author.email_count}
              </div>
              <div className="text-xs text-muted-foreground">Messages</div>
            </div>
            <div className="text-center p-3 rounded-lg bg-secondary/5">
              <div className="text-2xl font-bold text-secondary">
                {author.thread_count}
              </div>
              <div className="text-xs text-muted-foreground">Threads</div>
            </div>
          </div>

          {/* Activity Period */}
          <div className="flex items-center gap-2 text-xs text-muted-foreground mb-3">
            <Calendar className="h-3.5 w-3.5 flex-shrink-0" />
            <span>
              Active: {formatDate(author.first_email_date)} – {formatDate(author.last_email_date)}
            </span>
          </div>

          {/* Mailing Lists */}
          {author.mailing_lists && author.mailing_lists.length > 0 && (
            <div className="mb-3">
              <div className="text-xs font-medium text-muted-foreground mb-2">
                Active in {author.mailing_lists.length} list{author.mailing_lists.length > 1 ? 's' : ''}
              </div>
              <div className="flex flex-wrap gap-1.5">
                {author.mailing_lists.map((ml) => (
                  <Badge key={ml} variant="outline" className="text-xs">
                    {ml}
                  </Badge>
                ))}
              </div>
            </div>
          )}

          {/* Name Variations */}
          {author.name_variations && author.name_variations.length > 1 && (
            <div>
              <div className="text-xs font-medium text-muted-foreground mb-2">
                Also known as:
              </div>
              <div className="text-xs text-muted-foreground">
                {author.name_variations.slice(1, 4).join(', ')}
                {author.name_variations.length > 4 && ` +${author.name_variations.length - 4} more`}
              </div>
            </div>
          )}
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
          {loading && currentThreads.length === 0 ? (
            <div className="p-8 text-center">
              <div className="text-xs text-muted-foreground">Loading threads...</div>
            </div>
          ) : currentThreads.length === 0 ? (
            <div className="p-8 text-center">
              <div className="text-xs text-muted-foreground">No threads found</div>
            </div>
          ) : (
            <div className="space-y-1">
              {currentThreads.map((thread) => {
                const isSelected = threadId === String(thread.id);
                return (
                  <Link
                    key={thread.id}
                    to={`/authors/${authorId}/threads/${thread.id}`}
                    className="block"
                  >
                    <div
                      className={cn(
                        "px-3 py-2.5 rounded-md hover:bg-accent/50 transition-all",
                        isSelected && "bg-accent border-l-2 border-primary pl-[10px]"
                      )}
                    >
                      <div className="text-sm font-semibold line-clamp-2 mb-1.5">
                        {thread.subject}
                      </div>
                      <div className="flex items-center gap-2 text-xs text-muted-foreground">
                        <Badge variant="outline" className="h-4 px-1 text-xs">
                          {thread.message_count || 0}
                        </Badge>
                        <span>{formatDateCompact(thread.last_date, timezone)}</span>
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
            disabled={
              !currentPageInfo || currentPageInfo.totalPages === 0 || currentPageInfo.page >= currentPageInfo.totalPages
            }
          >
            Next
          </Button>
        </div>
      </div>
    </div>
  );
}
