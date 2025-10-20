import type { ReactNode } from 'react';
import { ThreadList } from './ThreadList';
import { ThreadView } from './ThreadView';
import type { Thread } from '../types';
import { cn } from '../lib/utils';

interface ThreadBrowserLayoutProps {
  threads: Thread[];
  loading: boolean;
  selectedThreadId: number | null;
  onThreadSelect: (thread: Thread) => void;
  currentPage: number;
  hasMore: boolean;
  onPageChange: (page: number) => void;
  maxPage: number;
  onSearch: (query: string) => void;
  searchQuery: string;
  leftPanelHeader?: ReactNode;
  threadsCollapsed: boolean;
}

export function ThreadBrowserLayout({
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
  leftPanelHeader,
  threadsCollapsed,
}: ThreadBrowserLayoutProps) {
  return (
    <div className="flex-1 flex flex-col relative bg-background overflow-hidden">
      <div className="flex-1 flex flex-col md:flex-row bg-background min-h-0">
        {/* Left panel */}
        <div
          className={cn(
            'w-full flex flex-col min-h-0 border-b border-surface-border/60 md:border-b-0 transition-all duration-300 ease-in-out bg-background',
            threadsCollapsed ? 'md:w-0 md:opacity-0 md:pointer-events-none md:-ml-[1px]' : 'md:w-[26rem] md:min-w-[18rem]'
          )}
          style={{
            borderRight: threadsCollapsed ? undefined : '3px solid hsl(var(--color-border) / 0.6)',
          }}
        >
          {leftPanelHeader}
          <div className="flex-1 min-h-0">
            <ThreadList
              threads={threads}
              loading={loading}
              selectedThreadId={selectedThreadId}
              onThreadSelect={onThreadSelect}
              currentPage={currentPage}
              hasMore={hasMore}
              onPageChange={onPageChange}
              maxPage={maxPage}
              onSearch={onSearch}
              searchQuery={searchQuery}
            />
          </div>
        </div>

        {/* Right panel */}
        <div className="flex-1 hidden md:flex flex-col min-w-0 min-h-0 relative">
          <ThreadView threadId={selectedThreadId} />
        </div>
      </div>
    </div>
  );
}
