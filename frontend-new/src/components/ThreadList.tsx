import { useState, useEffect, useRef, useCallback } from 'react';
import { Search } from 'lucide-react';
import { ScrollArea } from './ui/scroll-area';
import { Input } from './ui/input';
import type { ThreadWithStarter } from '../types';
import { Pagination } from './Pagination';
import { useTimezone } from '../contexts/TimezoneContext';
import { formatDateCompact } from '../utils/timezone';

const isWithinLastWeek = (dateString: string): boolean => {
  if (!dateString) return false;
  const date = new Date(dateString);
  if (Number.isNaN(date.getTime())) return false;
  const now = new Date();
  const diffInMs = now.getTime() - date.getTime();
  if (diffInMs < 0) {
    return false;
  }
  const msInDay = 1000 * 60 * 60 * 24;
  return diffInMs <= 7 * msInDay;
};

interface ThreadListProps {
  threads: ThreadWithStarter[];
  loading: boolean;
  selectedThreadId: number | null;
  onThreadSelect: (thread: ThreadWithStarter) => void;
  currentPage: number;
  hasMore: boolean;
  onPageChange: (page: number) => void;
  maxPage: number;
  onSearch: (query: string) => void;
  searchQuery: string;
}

export function ThreadList({
  threads,
  loading,
  selectedThreadId,
  onThreadSelect,
  currentPage,
  hasMore,
  onPageChange,
  maxPage,
  onSearch,
  searchQuery,
}: ThreadListProps) {
  const { timezone } = useTimezone();
  const [localQuery, setLocalQuery] = useState(searchQuery);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const debounceTimeoutRef = useRef<number | null>(null);

  const debouncedSearch = useCallback((query: string) => {
    if (debounceTimeoutRef.current) {
      clearTimeout(debounceTimeoutRef.current);
    }
    debounceTimeoutRef.current = window.setTimeout(() => {
      onSearch(query);
    }, 500);
  }, [onSearch]);

  const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setLocalQuery(value);
    debouncedSearch(value);
  };

  // Handle "/" hotkey to focus search
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === '/' && !['INPUT', 'TEXTAREA'].includes((e.target as HTMLElement).tagName)) {
        e.preventDefault();
        searchInputRef.current?.focus();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  // Cleanup timeout on unmount
  useEffect(() => {
    return () => {
      if (debounceTimeoutRef.current) {
        clearTimeout(debounceTimeoutRef.current);
      }
    };
  }, []);

  const renderSearchBar = () => (
    <div className="px-3 py-2 border-b border-surface-border/60 flex-shrink-0">
      <div className="flex items-center gap-2">
        <div className="relative flex-1">
          <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            ref={searchInputRef}
            type="search"
            placeholder="Search threads..."
            className="pl-8 pr-12 h-9 w-full"
            value={localQuery}
            onChange={handleSearchChange}
          />
          <kbd className="absolute right-2 top-2 pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground opacity-100">
            /
          </kbd>
        </div>
      </div>
    </div>
  );

  if (loading) {
    return (
      <div className="h-full flex flex-col min-h-0 bg-background">
        {renderSearchBar()}
        <div className="flex-1 overflow-y-auto p-4 space-y-3">
          {Array.from({ length: 10 }).map((_, i) => (
            <div key={i} className="py-2 pl-3 animate-pulse">
              <div className="h-4 bg-muted rounded w-3/4 mb-1"></div>
              <div className="h-3 bg-muted/60 rounded w-1/2"></div>
            </div>
          ))}
        </div>
      </div>
    );
  }

  if (threads.length === 0) {
    return (
      <div className="h-full flex flex-col min-h-0 bg-background">
        {renderSearchBar()}
        <div className="flex items-center justify-center flex-1 p-8 text-center">
          <div className="space-y-2">
            <p className="text-sm text-muted-foreground">No threads found</p>
            <p className="text-xs text-muted-foreground">
              Try selecting a different mailing list or search term
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col min-h-0 bg-background">
      {renderSearchBar()}

      <ScrollArea className="flex-1 min-h-0">
        <div className="py-1">
          {threads.map((thread) => {
            const startDateText = formatDateCompact(thread.start_date, timezone);
            const shouldPrefix =
              isWithinLastWeek(thread.start_date) &&
              startDateText !== 'Invalid Date' &&
              startDateText !== 'N/A';
            const startDateDisplay = shouldPrefix ? `Posted ${startDateText}` : startDateText;
            const showLastDate = thread.last_date !== thread.start_date && thread.last_date;
            const lastDateText = showLastDate
              ? formatDateCompact(thread.last_date, timezone)
              : null;

            return (
              <div
                key={thread.id}
                data-selected={selectedThreadId === thread.id}
                className={`thread-list-item px-3 py-2 outline-none transition-all duration-150 select-none hover:shadow-sm ${
                  selectedThreadId === thread.id ? 'thread-list-item--selected' : ''
                }`}
                onClick={() => onThreadSelect(thread)}
                role="button"
                tabIndex={0}
                onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onThreadSelect(thread); } }}
                aria-selected={selectedThreadId === thread.id}
              >
                <div className="flex items-start justify-between gap-2 mb-1">
                  <h3 className="text-sm font-semibold text-foreground leading-tight flex-1 min-w-0 break-words line-clamp-2">
                    {thread.subject}
                  </h3>
                  <span className="text-xs text-muted-foreground shrink-0">
                    [{thread.message_count || 0}]
                  </span>
                </div>
                <div className="flex items-center justify-between text-xs text-muted-foreground">
                  <span>{thread.starter_name || thread.starter_email}</span>
                  <div className="flex items-center gap-2">
                    <span>{startDateDisplay}</span>
                    {showLastDate && lastDateText && (
                      <>
                        <span>•</span>
                        <span>{lastDateText}</span>
                      </>
                    )}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </ScrollArea>
      
      {/* Pagination controls */}
      <div className="flex-shrink-0">
        <Pagination
          currentPage={currentPage}
          maxPage={maxPage}
          onPageChange={onPageChange}
          hasMore={hasMore}
        />
      </div>
    </div>
  );
}
