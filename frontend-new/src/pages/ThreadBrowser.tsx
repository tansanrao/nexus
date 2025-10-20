import { useState } from 'react';
import { AuthorView } from './AuthorView';
import { TopBar } from '../components/TopBar';
import { ThreadBrowserLayout } from '../components/ThreadBrowserLayout';
import { useThreadBrowser } from '../hooks/useThreadBrowser';
import { useSearchParams } from 'react-router-dom';

export function ThreadBrowser() {
  const [searchParams] = useSearchParams();
  const authorId = searchParams.get('author');
  const [threadsCollapsed, setThreadsCollapsed] = useState(false);
  
  const handleCollapseThreads = () => setThreadsCollapsed(true);
  const handleExpandThreads = () => setThreadsCollapsed(false);
  
  const {
    threads,
    loading,
    selectedThread,
    searchQuery,
    currentPage,
    hasMore,
    totalThreads,
    maxPage,
    filters,
    handleSearch,
    handleThreadSelect,
    handlePageChange,
    handleFiltersChange,
  } = useThreadBrowser();

  // If showing author view, use different layout
  if (authorId) {
    return (
      <div className="h-screen flex flex-col relative bg-background">
        <TopBar
          filters={filters}
          onFiltersChange={handleFiltersChange}
          threadCount={totalThreads ?? 0}
          threadsCollapsed={threadsCollapsed}
          onCollapseThreads={handleCollapseThreads}
          onExpandThreads={handleExpandThreads}
        />
        <div className="flex-1 overflow-hidden">
          <AuthorView
            authorId={authorId}
            threadsCollapsed={threadsCollapsed}
          />
        </div>
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col relative bg-background">
      <TopBar
        filters={filters}
        onFiltersChange={handleFiltersChange}
        threadCount={totalThreads ?? 0}
        threadsCollapsed={threadsCollapsed}
        onCollapseThreads={handleCollapseThreads}
        onExpandThreads={handleExpandThreads}
      />
      
      <ThreadBrowserLayout
        threads={threads}
        loading={loading}
        selectedThreadId={selectedThread?.id || null}
        onThreadSelect={handleThreadSelect}
        currentPage={currentPage}
        hasMore={hasMore}
        onPageChange={handlePageChange}
        maxPage={maxPage}
        onSearch={handleSearch}
        searchQuery={searchQuery}
        threadsCollapsed={threadsCollapsed}
      />
    </div>
  );
}
