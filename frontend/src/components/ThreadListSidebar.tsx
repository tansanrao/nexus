import { useEffect, useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import { Search, SortAsc, SortDesc, Mail } from 'lucide-react';
import { api } from '../api/client';
import type { Thread, ThreadSortBy, SortOrder, SearchType } from '../types';
import { useTimezone } from '../contexts/TimezoneContext';
import { formatDateInTimezone } from '../utils/timezone';
import { formatDistanceToNow } from 'date-fns';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Card } from './ui/card';
import { Badge } from './ui/badge';
import { ScrollArea } from './ui/scroll-area';
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
  const { threadId, mailingList } = useParams<{ threadId: string; mailingList: string }>();
  const { timezone } = useTimezone();
  const limit = 50;

  useEffect(() => {
    const loadThreads = async () => {
      if (!mailingList) return;

      try {
        setLoading(true);
        let data: Thread[];

        if (activeSearch.trim()) {
          data = await api.threads.search(mailingList, {
            search: activeSearch,
            search_type: searchType,
            page,
            limit,
            sort_by: sortBy,
            order
          });
        } else {
          data = await api.threads.list(mailingList, { page, limit, sort_by: sortBy, order });
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
  }, [page, sortBy, order, activeSearch, searchType, mailingList]);

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
    return formatDateInTimezone(dateStr, timezone, 'MMM d, yyyy h:mm a');
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
      <div className="border-b p-4 space-y-3">
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

        {/* Search type */}
        <div className="flex gap-1">
          <Button
            variant={searchType === 'subject' ? 'default' : 'outline'}
            size="sm"
            onClick={() => setSearchType('subject')}
            className="flex-1 h-8 text-xs"
          >
            Subject
          </Button>
          <Button
            variant={searchType === 'full_text' ? 'default' : 'outline'}
            size="sm"
            onClick={() => setSearchType('full_text')}
            className="flex-1 h-8 text-xs"
          >
            Full Text
          </Button>
        </div>

        {/* Sort controls */}
        <div className="flex gap-1">
          <Button
            variant={sortBy === 'last_date' ? 'secondary' : 'ghost'}
            size="sm"
            onClick={() => handleSortChange('last_date')}
            className="flex-1 h-8 text-xs"
          >
            Last
            {sortBy === 'last_date' && (
              order === 'desc' ? <SortDesc className="ml-1 h-3 w-3" /> : <SortAsc className="ml-1 h-3 w-3" />
            )}
          </Button>
          <Button
            variant={sortBy === 'start_date' ? 'secondary' : 'ghost'}
            size="sm"
            onClick={() => handleSortChange('start_date')}
            className="flex-1 h-8 text-xs"
          >
            Start
            {sortBy === 'start_date' && (
              order === 'desc' ? <SortDesc className="ml-1 h-3 w-3" /> : <SortAsc className="ml-1 h-3 w-3" />
            )}
          </Button>
          <Button
            variant={sortBy === 'message_count' ? 'secondary' : 'ghost'}
            size="sm"
            onClick={() => handleSortChange('message_count')}
            className="flex-1 h-8 text-xs"
          >
            Count
            {sortBy === 'message_count' && (
              order === 'desc' ? <SortDesc className="ml-1 h-3 w-3" /> : <SortAsc className="ml-1 h-3 w-3" />
            )}
          </Button>
        </div>
      </div>

      {/* Thread list */}
      <ScrollArea className="flex-1">
        <div className="p-2">
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
                    to={`/${mailingList}/threads/${thread.id}`}
                    className="block border-b last:border-b-0"
                  >
                    <div
                      className={cn(
                        "p-3 transition-colors hover:bg-accent",
                        isSelected && "bg-accent"
                      )}
                    >
                      <div className="text-sm font-medium line-clamp-2 mb-2">
                        {thread.subject}
                      </div>
                      <div className="space-y-1 text-xs">
                        <div className="flex items-center gap-1.5">
                          <span className="text-muted-foreground font-medium">Messages:</span>
                          <Badge variant="secondary" className="text-xs px-1.5 py-0 h-5">
                            {thread.message_count || 0}
                          </Badge>
                        </div>
                        <div className="flex items-center gap-1.5">
                          <span className="text-muted-foreground font-medium">Started:</span>
                          <span className="text-foreground">{formatStartDate(thread.start_date)}</span>
                        </div>
                        <div className="flex items-center gap-1.5">
                          <span className="text-muted-foreground font-medium">Last Activity:</span>
                          <span className="text-foreground">{formatRelativeTime(thread.last_date)}</span>
                        </div>
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
