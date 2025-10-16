import { AuthorView } from './AuthorView';
import { ThemeToggle } from '../components/ThemeToggle';
import { SettingsDropdown } from '../components/SettingsDropdown';
import { ThreadBrowserLayout } from '../components/ThreadBrowserLayout';
import { useThreadBrowser } from '../hooks/useThreadBrowser';
import { useSearchParams } from 'react-router-dom';

export function ThreadBrowser() {
  const [searchParams] = useSearchParams();
  const authorId = searchParams.get('author');
  
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
    pageSize,
    handleSearch,
    handleThreadSelect,
    handlePageChange,
    handleFiltersChange,
  } = useThreadBrowser();


  // If showing author view, use different layout
  if (authorId) {
    return (
      <div className="h-screen flex flex-col relative">
        {/* Floating settings buttons */}
        <div className="fixed top-4 right-4 z-50 flex items-center gap-2">
          <ThemeToggle />
          <SettingsDropdown />
        </div>
        <div className="flex-1 overflow-hidden">
          <AuthorView authorId={authorId} />
        </div>
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col relative">
      {/* Floating settings buttons */}
      <div className="fixed top-4 right-4 z-50 flex items-center gap-2">
        <ThemeToggle />
        <SettingsDropdown />
      </div>
      
      <ThreadBrowserLayout
        threads={threads}
        loading={loading}
        selectedThreadId={selectedThread?.id || null}
        onThreadSelect={handleThreadSelect}
        currentPage={currentPage}
        hasMore={hasMore}
        onPageChange={handlePageChange}
        filters={filters}
        onFiltersChange={handleFiltersChange}
        onSearch={handleSearch}
        searchQuery={searchQuery}
        totalThreads={totalThreads}
        maxPage={maxPage}
        pageSize={pageSize}
      />
    </div>
  );
}

