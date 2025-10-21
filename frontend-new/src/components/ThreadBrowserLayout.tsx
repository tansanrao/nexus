import type { ReactNode } from 'react';
import { ThreadList } from './ThreadList';
import { ThreadView } from './ThreadView';
import { ThreadDiffView } from './ThreadDiffView';
import type { ThreadWithStarter } from '../types';
import { cn } from '../lib/utils';
import { useThreadDetail } from '../hooks/useThreadDetail';

interface ThreadBrowserLayoutProps {
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
  leftPanelHeader?: ReactNode;
  threadsCollapsed: boolean;
  activeRightView?: 'thread' | 'diff';
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
  activeRightView = 'thread',
}: ThreadBrowserLayoutProps) {
  const {
    threadDetail,
    loading: threadDetailLoading,
    error: threadDetailError,
  } = useThreadDetail(selectedThreadId);

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
          {activeRightView === 'diff' ? (
            <ThreadDiffView
              threadId={selectedThreadId}
              threadDetail={threadDetail}
              loading={threadDetailLoading}
              error={threadDetailError}
            />
          ) : (
            <ThreadView
              threadId={selectedThreadId}
              threadDetail={threadDetail}
              loading={threadDetailLoading}
              error={threadDetailError}
            />
          )}
        </div>
      </div>
    </div>
  );
}
