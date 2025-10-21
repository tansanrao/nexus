import { useEffect, useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import { formatDistanceToNow } from 'date-fns';
import { Mail, Search } from 'lucide-react';
import { api } from '../api/client';
import type {
  ThreadWithStarter,
  ThreadSortBy,
  SortOrder,
  SearchType,
  PageMetadata,
  PaginatedResponse,
} from '../types';
import { useTimezone } from '../contexts/TimezoneContext';
import { useMailingList } from '../contexts/MailingListContext';
import { formatDateInTimezone } from '../utils/timezone';
import { Input } from './ui/input';
import { ScrollArea } from './ui/scroll-area';
import { FilterPanel } from './mailinglist/FilterPanel';
import { CompactButton } from './ui/compact-button';
import { cn } from '@/lib/utils';

export function ThreadListSidebar() {
  const [threads, setThreads] = useState<ThreadWithStarter[]>([]);
  const [pageInfo, setPageInfo] = useState<PageMetadata | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const [sortBy, setSortBy] = useState<ThreadSortBy>('lastDate');
  const [order, setOrder] = useState<SortOrder>('desc');
  const [searchInput, setSearchInput] = useState('');
  const [activeSearch, setActiveSearch] = useState('');
  const [searchType, setSearchType] = useState<SearchType>('subject');
  const [filtersExpanded, setFiltersExpanded] = useState(false);
  const { threadId } = useParams<{ threadId: string }>();
  const { selectedMailingList } = useMailingList();
  const { timezone } = useTimezone();
  const size = 50;

  useEffect(() => {
    const loadThreads = async () => {
      if (!selectedMailingList) return;
      try {
        setLoading(true);
        let response: PaginatedResponse<ThreadWithStarter>;
        if (activeSearch.trim()) {
          response = await api.threads.search(selectedMailingList, {
            q: activeSearch,
            searchType: searchType,
            page,
            size,
            sortBy: sortBy,
            order,
          });
        } else {
          response = await api.threads.list(selectedMailingList, { page, size, sortBy: sortBy, order });
        }
        setThreads(response.data);
        setPageInfo(response.page);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load threads');
        setPageInfo(null);
      } finally {
        setLoading(false);
      }
    };

    loadThreads();
  }, [page, sortBy, order, activeSearch, searchType, selectedMailingList]);

  useEffect(() => {
    setPage(1);
  }, [selectedMailingList]);

  const handleSearch = () => {
    setActiveSearch(searchInput);
    setPage(1);
  };

  const handleClearSearch = () => {
    setSearchInput('');
    setActiveSearch('');
    setPage(1);
  };

  const handleKeyPress = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter') {
      handleSearch();
    }
  };

  const handleSortChange = (newSortBy: ThreadSortBy) => {
    if (newSortBy === sortBy) {
      setOrder(order === 'desc' ? 'asc' : 'desc');
    } else {
      setSortBy(newSortBy);
      setOrder('desc');
    }
    setPage(1);
  };

  const formatStartDate = (dateStr: string) =>
    formatDateInTimezone(dateStr, timezone, 'MMM d, yyyy');

  const formatRelativeTime = (dateStr: string) => {
    try {
      return formatDistanceToNow(new Date(dateStr), { addSuffix: true });
    } catch {
      return dateStr;
    }
  };

  if (loading && threads.length === 0) {
    return (
      <div className="h-full flex items-center justify-center">
        <span className="text-label">Loading threads…</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="h-full flex items-center justify-center px-4 text-center">
        <span className="text-label text-danger">Error: {error}</span>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col bg-surface-muted">
      <div className="surface-overlay mx-3 mt-3 rounded-md px-3 py-3 space-y-3">
        <form
          className="flex gap-2"
          onSubmit={(event) => {
            event.preventDefault();
            handleSearch();
          }}
        >
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              type="text"
              value={searchInput}
              onChange={(event) => setSearchInput(event.target.value)}
              onKeyPress={handleKeyPress}
              placeholder="Search subject or message text"
              className="pl-9 h-9 text-sm bg-surface-base border border-surface-border/80"
            />
          </div>
          <CompactButton type="submit" className="px-3">
            Go
          </CompactButton>
        </form>

        <div className="flex items-center justify-between">
          <span className="text-label">
            {activeSearch ? `Scope: ${searchType === 'subject' ? 'Subject' : 'Full text'}` : 'Recent threads'}
          </span>
          {activeSearch && (
            <CompactButton onClick={handleClearSearch} className="px-2">
              Clear
            </CompactButton>
          )}
        </div>

        <FilterPanel
          searchType={searchType}
          setSearchType={setSearchType}
          sortBy={sortBy}
          order={order}
          onSortChange={handleSortChange}
          isExpanded={filtersExpanded}
          onToggle={() => setFiltersExpanded(!filtersExpanded)}
        />
      </div>

      <ScrollArea className="flex-1 mt-3">
        <div className="space-y-2 px-3 pb-3">
          {threads.length === 0 ? (
            <div className="surface-muted px-4 py-6 text-center">
              <Mail className="h-7 w-7 mx-auto text-muted-foreground mb-2" />
              <p className="text-label">No threads found</p>
            </div>
          ) : (
            threads.map((thread) => {
              const isSelected = threadId === String(thread.id);
              return (
                <Link
                  key={thread.id}
                  to={`/threads/${thread.id}`}
                  className={cn(
                    "block rounded-md border border-transparent bg-surface-base/80 px-3 py-2 transition-all hover:border-surface-border/80 hover:bg-surface-overlay focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/40",
                    isSelected && "border-accent-primary bg-surface-overlay shadow-sm"
                  )}
                >
                  <div className="flex items-start gap-2">
                    <div className="flex-1 space-y-1">
                      <h3 className="text-sm font-semibold leading-snug text-foreground line-clamp-2">
                        {thread.subject}
                      </h3>
                      <div className="flex flex-wrap items-center gap-2 text-[11px] uppercase tracking-[0.08em] text-muted-foreground">
                        <span>{formatRelativeTime(thread.last_date)}</span>
                        <span aria-hidden="true">•</span>
                        <span>Started {formatStartDate(thread.start_date)}</span>
                      </div>
                    </div>
                    <span className="pill">{thread.message_count || 0}</span>
                  </div>
                </Link>
              );
            })
          )}
        </div>
      </ScrollArea>

      <div className="mt-auto border-t border-border/60 px-3 py-3">
        <div className="flex items-center justify-between text-label">
          <CompactButton onClick={() => setPage((p) => Math.max(1, p - 1))} disabled={page === 1}>
            Prev
          </CompactButton>
          <span>Page {page}</span>
          <CompactButton
            onClick={() => setPage((p) => p + 1)}
            disabled={
              !pageInfo || pageInfo.totalPages === 0 || pageInfo.page >= pageInfo.totalPages
            }
          >
            Next
          </CompactButton>
        </div>
      </div>
    </div>
  );
}
