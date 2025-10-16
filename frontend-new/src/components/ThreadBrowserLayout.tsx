import type { ReactNode } from 'react';
import { ThreadList } from './ThreadList';
import { ThreadView } from './ThreadView';
import type { Thread } from '../types';
import type { ThreadFilters } from './ThreadListHeader';

interface ThreadBrowserLayoutProps {
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
  totalThreads: number | null;
  maxPage: number;
  leftPanelHeader?: ReactNode;
}

export function ThreadBrowserLayout({
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
  leftPanelHeader,
}: ThreadBrowserLayoutProps) {
  return (
    <div className="h-screen flex flex-col relative bg-background">
      <div className="flex-1 overflow-hidden">
        <div className="h-full grid grid-cols-1 md:grid-cols-5 bg-background">
          {/* Left panel */}
          <div className="md:col-span-2 h-full overflow-hidden bg-surface-inset flex flex-col border-r border-surface-border/60">
            {leftPanelHeader}
            <div className="flex-1 overflow-hidden">
              <ThreadList
                threads={threads}
                loading={loading}
                selectedThreadId={selectedThreadId}
                onThreadSelect={onThreadSelect}
                currentPage={currentPage}
                hasMore={hasMore}
                onPageChange={onPageChange}
                filters={filters}
                onFiltersChange={onFiltersChange}
                onSearch={onSearch}
                searchQuery={searchQuery}
                totalThreads={totalThreads}
                maxPage={maxPage}
              />
            </div>
          </div>
          
          {/* Right panel */}
          <div className="md:col-span-3 hidden md:flex h-full flex-col overflow-hidden bg-surface-inset border-l border-surface-border/60 min-w-0">
            <ThreadView threadId={selectedThreadId} />
          </div>
        </div>
      </div>
    </div>
  );
}
