import { useEffect, useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import { Search, Mail } from 'lucide-react';
import { api } from '../api/client';
import type { Thread, ThreadSortBy, SortOrder, SearchType } from '../types';
import { useTimezone } from '../contexts/TimezoneContext';
import { useMailingList } from '../contexts/MailingListContext';
import { formatDateInTimezone } from '../utils/timezone';
import { formatDistanceToNow } from 'date-fns';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Card } from './ui/card';
import { Badge } from './ui/badge';
import { ScrollArea } from './ui/scroll-area';
import { FilterPanel } from './mailinglist/FilterPanel';
import { cn } from '@/lib/utils';

export function ThreadListSidebar() {
  const [threads, setThreads] = useState<Thread[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const [sortBy, setSortBy] = useState<ThreadSortBy>('last_date');
  const [order, setOrder] = useState<SortOrder>('desc');
  const [searchInput, setSearchInput] = useState('');
  const [activeSearch, setActiveSearch] = useState('');
  const [searchType, setSearchType] = useState<SearchType>('subject');
  const [filtersExpanded, setFiltersExpanded] = useState(false);
  const { threadId } = useParams<{ threadId: string }>();
  const { selectedMailingList } = useMailingList();
  const { timezone } = useTimezone();
  const limit = 50;

  useEffect(() => {
    const loadThreads = async () => {
      if (!selectedMailingList) return;

      try {
        setLoading(true);
        let data: Thread[];

        if (activeSearch.trim()) {
          data = await api.threads.search(selectedMailingList, {
            search: activeSearch,
            search_type: searchType,
            page,
            limit,
            sort_by: sortBy,
            order
          });
        } else {
          data = await api.threads.list(selectedMailingList, { page, limit, sort_by: sortBy, order });
        }

        setThreads(data);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load threads');
      } finally {
        setLoading(false);
      }
    };

    loadThreads();
  }, [page, sortBy, order, activeSearch, searchType, selectedMailingList]);

  const handleSearch = () => {
    setActiveSearch(searchInput);
    setPage(1);
  };

  const handleClearSearch = () => {
    setSearchInput('');
    setActiveSearch('');
    setPage(1);
  };

  const handleKeyPress = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
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

  const formatStartDate = (dateStr: string) => {
    return formatDateInTimezone(dateStr, timezone, 'MMM d, yyyy');
  };

  const formatRelativeTime = (dateStr: string) => {
    try {
      const date = new Date(dateStr);
      return formatDistanceToNow(date, { addSuffix: true });
    } catch (error) {
      console.error('Error formatting relative time:', error);
      return dateStr;
    }
  };

  if (loading && threads.length === 0) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-sm text-muted-foreground">Loading threads...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="h-full flex items-center justify-center p-4">
        <Card className="p-6">
          <div className="text-sm text-destructive">Error: {error}</div>
        </Card>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Search and filters */}
      <div className="border-b p-3 space-y-3">
        {/* Search bar */}
        <div className="flex gap-2">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              type="text"
              value={searchInput}
              onChange={(e) => setSearchInput(e.target.value)}
              onKeyPress={handleKeyPress}
              placeholder="Search..."
              className="pl-9 h-9"
            />
          </div>
          <Button size="sm" onClick={handleSearch}>Search</Button>
        </div>

        {activeSearch && (
          <div className="flex items-center justify-between gap-2">
            <div className="text-xs text-muted-foreground">
              Searching in {searchType === 'subject' ? 'subject' : 'full text'}
            </div>
            <Button variant="ghost" size="sm" onClick={handleClearSearch}>
              Clear
            </Button>
          </div>
        )}

        {/* Filter panel */}
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

      {/* Thread list */}
      <ScrollArea className="flex-1">
        <div className="py-1">
          {threads.length === 0 ? (
            <div className="p-8 text-center">
              <Mail className="h-8 w-8 mx-auto text-muted-foreground mb-2" />
              <p className="text-xs text-muted-foreground">No threads found</p>
            </div>
          ) : (
            <div className="space-y-0">
              {threads.map((thread) => {
                const isSelected = threadId === String(thread.id);
                return (
                  <Link
                    key={thread.id}
                    to={`/threads/${thread.id}`}
                    className="block"
                  >
                    <div
                      className={cn(
                        "px-3 py-3 border-l-2 border-transparent hover:bg-accent/50 cursor-pointer transition-all duration-200",
                        isSelected && "border-l-primary bg-accent"
                      )}
                    >
                      {/* Row 1: Subject + Message count */}
                      <div className="flex items-start justify-between gap-2 mb-1">
                        <h3 className="text-sm font-semibold line-clamp-1 flex-1">
                          {thread.subject}
                        </h3>
                        <Badge variant="outline" className="text-xs h-5 px-1.5 shrink-0">
                          {thread.message_count || 0}
                        </Badge>
                      </div>

                      {/* Row 2: Metadata */}
                      <div className="flex items-center gap-2 text-xs text-muted-foreground">
                        <span className="truncate">{formatRelativeTime(thread.last_date)}</span>
                        <span>•</span>
                        <span className="truncate">Started {formatStartDate(thread.start_date)}</span>
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
            disabled={threads.length < limit}
          >
            Next
          </Button>
        </div>
      </div>
    </div>
  );
}
