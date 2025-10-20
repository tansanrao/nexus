import type { ReactNode } from 'react';
import { ThreadList } from './ThreadList';
import { ThreadView } from './ThreadView';
import type { Thread } from '../types';

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
}: ThreadBrowserLayoutProps) {
  return (
    <div className="flex-1 flex flex-col relative bg-background overflow-hidden">
      <div className="flex-1 grid grid-cols-1 md:grid-cols-5 bg-background min-h-0">
        {/* Left panel */}
        <div className="md:col-span-2 flex flex-col min-h-0" 
             style={{ 
               borderRight: '3px solid hsl(var(--color-border) / 0.6)'
             }}>
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
        <div className="md:col-span-3 hidden md:flex flex-col min-w-0 min-h-0">
          <ThreadView threadId={selectedThreadId} />
        </div>
      </div>
    </div>
  );
}
