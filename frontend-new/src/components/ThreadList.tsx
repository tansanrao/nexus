import { ScrollArea } from './ui/scroll-area';
import type { Thread } from '../types';
import { formatRelativeTime } from '../utils/date';
import { cn } from '../lib/utils';
import { ThreadListHeader, type ThreadFilters } from './ThreadListHeader';
import { Pagination } from './Pagination';

interface ThreadListProps {
  threads: Thread[];
  loading: boolean;
  selectedThreadId: number | null;
  onThreadSelect: (thread: Thread) => void;
  currentPage: number;
  hasMore: boolean;
  onPageChange: (page: number) => void;
  filters: ThreadFilters;
  onFiltersChange: (filters: ThreadFilters) => void;
  onSearch: (query: string) => void;
  searchQuery: string;
  totalThreads?: number | null;
  maxPage: number;
}

export function ThreadList({
  threads,
  loading,
  selectedThreadId,
  onThreadSelect,
  currentPage,
  hasMore,
  onPageChange,
  filters,
  onFiltersChange,
  onSearch,
  searchQuery,
  totalThreads,
  maxPage,
}: ThreadListProps) {
  if (loading) {
    return (
      <div className="h-full flex flex-col bg-surface-inset">
        <ThreadListHeader 
          filters={filters} 
          onFiltersChange={onFiltersChange} 
          threadCount={totalThreads ?? 0}
          onSearch={onSearch}
          searchQuery={searchQuery}
        />
        <div className="p-4 space-y-3">
          {Array.from({ length: 10 }).map((_, i) => (
            <div key={i} className="py-2 border-l-2 border-transparent pl-3 animate-pulse">
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
      <div className="h-full flex flex-col bg-surface-inset">
        <ThreadListHeader 
          filters={filters} 
          onFiltersChange={onFiltersChange} 
          threadCount={totalThreads ?? 0}
          onSearch={onSearch}
          searchQuery={searchQuery}
        />
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
    <div className="h-full flex flex-col bg-surface-inset">
      <ThreadListHeader 
        filters={filters} 
        onFiltersChange={onFiltersChange} 
        threadCount={typeof totalThreads === 'number' ? totalThreads : threads.length}
        onSearch={onSearch}
        searchQuery={searchQuery}
      />
      <ScrollArea className="flex-1">
        <div className="py-1">
          {threads.map((thread) => (
            <div
              key={thread.id}
              data-selected={selectedThreadId === thread.id}
              className={cn(
                'px-3 py-2 border-l-2 border-l-transparent outline-none transition-all duration-150 cursor-pointer select-none',
                'hover:bg-black/5 dark:hover:bg-white/10 hover:shadow-sm',
                selectedThreadId === thread.id &&
                  'border-l-primary bg-black/15 dark:bg-white/15 text-foreground shadow-sm'
              )}
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
              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                <span>{formatRelativeTime(thread.last_date)}</span>
                {thread.last_date !== thread.start_date && (
                  <>
                    <span>•</span>
                    <span>started {formatRelativeTime(thread.start_date)}</span>
                  </>
                )}
              </div>
            </div>
          ))}
        </div>
      </ScrollArea>
      
      {/* Pagination controls */}
      <Pagination
        currentPage={currentPage}
        maxPage={maxPage}
        onPageChange={onPageChange}
        hasMore={hasMore}
      />
    </div>
  );
}
