import { useCallback, useEffect, useMemo, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import { X } from 'lucide-react';
import { Button } from '../components/ui/button';
import { ThreadBrowserLayout } from '../components/ThreadBrowserLayout';
import { useThreadBrowser } from '../hooks/useThreadBrowser';
import { apiClient } from '../lib/api';
import { useApiConfig } from '../contexts/ApiConfigContext';
import type { AuthorWithStats, Thread, ThreadWithStarter, PaginatedResponse } from '../types';
import type { ThreadFilters } from '../components/ThreadListHeader';

interface AuthorViewProps {
  authorId: string;
}

export function AuthorView({ authorId }: AuthorViewProps) {
  const { selectedMailingList } = useApiConfig();
  const [searchParams, setSearchParams] = useSearchParams();
  const [author, setAuthor] = useState<AuthorWithStats | null>(null);
  const [activeTab, setActiveTab] = useState<'created' | 'participated'>('created');
  const [threadsCreated, setThreadsCreated] = useState<ThreadWithStarter[] | null>(null);
  const [threadsParticipated, setThreadsParticipated] = useState<Thread[] | null>(null);

  const fetchAuthorThreads = useCallback(
    async ({
      page,
      pageSize,
      filters: activeFilters,
      searchTerm,
    }: {
      page: number;
      pageSize: number;
      filters: ThreadFilters;
      searchTerm: string;
    }) => {
      const sourceThreadsRaw =
        activeTab === 'created' ? threadsCreated : threadsParticipated;

      if (!sourceThreadsRaw) {
        return {
          data: [],
          page: {
            page: 1,
            size: pageSize,
            totalPages: 0,
            totalElements: 0,
          },
        };
      }

      let filteredThreads = [...sourceThreadsRaw];

      if (searchTerm) {
        const searchLower = searchTerm.toLowerCase();
        filteredThreads = filteredThreads.filter(thread =>
          thread.subject.toLowerCase().includes(searchLower)
        );
      }

      filteredThreads.sort((a, b) => {
        let aValue: number;
        let bValue: number;

        switch (activeFilters.sortBy) {
          case 'start_date':
            aValue = new Date(a.start_date).getTime();
            bValue = new Date(b.start_date).getTime();
            break;
          case 'message_count':
            aValue = a.message_count || 0;
            bValue = b.message_count || 0;
            break;
          case 'last_date':
          default:
            aValue = new Date(a.last_date).getTime();
            bValue = new Date(b.last_date).getTime();
            break;
        }

        return activeFilters.order === 'asc' ? aValue - bValue : bValue - aValue;
      });

      const total = filteredThreads.length;
      const totalPages = total > 0 ? Math.ceil(total / pageSize) : 0;
      const safePage =
        totalPages > 0 ? Math.min(Math.max(page, 1), totalPages) : 1;
      const startIndex = (safePage - 1) * pageSize;
      const pageItems = filteredThreads.slice(startIndex, startIndex + pageSize) as Thread[];

      return {
        data: pageItems,
        page: {
          page: safePage,
          size: pageSize,
          totalPages,
          totalElements: total,
        },
      };
    },
    [activeTab, threadsCreated, threadsParticipated]
  );

  // Use the shared thread browser hook
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
  } = useThreadBrowser({
    fetchThreads: fetchAuthorThreads,
    reloadDeps: [authorId, activeTab, threadsCreated, threadsParticipated],
  });

  const createdCountLabel = useMemo(() => {
    if (threadsCreated === null) return '...';
    return threadsCreated.length;
  }, [threadsCreated]);

  const participatedCountLabel = useMemo(() => {
    if (threadsParticipated === null) return '...';
    return threadsParticipated.length;
  }, [threadsParticipated]);

  const isAuthorThreadsLoading = threadsCreated === null || threadsParticipated === null;

  useEffect(() => {
    if (selectedMailingList && authorId) {
      loadAuthorData();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedMailingList, authorId]);

  // Fetch all author threads so we can reuse the shared list features (search, sort, pagination).
  const fetchAllAuthorThreads = async <T,>(
    fetchPage: (page: number, size: number) => Promise<PaginatedResponse<T>>,
    pageSize: number,
    maxPages: number = 200
  ): Promise<T[]> => {
    const results: T[] = [];
    let page = 1;

    while (page <= maxPages) {
      const response = await fetchPage(page, pageSize);
      results.push(...response.data);

      const totalPages = response.page.totalPages;
      if (page >= totalPages || response.data.length < pageSize) {
        break;
      }

      page += 1;
    }

    return results;
  };

  const loadAuthorData = async () => {
    if (!selectedMailingList || !authorId) return;

    const mailingList = selectedMailingList;
    const authorIdNumber = parseInt(authorId, 10);
    if (!Number.isFinite(authorIdNumber)) {
      console.warn('Invalid author id for author view:', authorId);
      return;
    }
    const API_PAGE_SIZE = 100; // API caps author thread pagination at 100 items

    setThreadsCreated(null);
    setThreadsParticipated(null);

    try {
      const [authorData, created, participated] = await Promise.all([
        apiClient.getAuthor(mailingList, authorIdNumber),
        fetchAllAuthorThreads<ThreadWithStarter>(
          (pageNumber, size) =>
            apiClient.getAuthorThreadsStarted(mailingList, authorIdNumber, pageNumber, size),
          API_PAGE_SIZE
        ),
        fetchAllAuthorThreads<Thread>(
          (pageNumber, size) =>
            apiClient.getAuthorThreadsParticipated(mailingList, authorIdNumber, pageNumber, size),
          API_PAGE_SIZE
        ),
      ]);
      setAuthor(authorData);
      setThreadsCreated(created);
      setThreadsParticipated(participated);

      // Auto-select the first tab that has threads
      setActiveTab((prevTab) => {
        if (created.length > 0) {
          return 'created';
        }
        if (participated.length > 0) {
          return 'participated';
        }
        return prevTab;
      });
    } catch (err) {
      console.error('Error loading author:', err);
    }
  };

  const handleTabChange = (tab: 'created' | 'participated') => {
    setActiveTab(tab);
  };

  const clearAuthorFilter = () => {
    const params = new URLSearchParams(searchParams);
    params.delete('author');
    setSearchParams(params, { replace: false });
  };

  // Create the author header component
  const authorHeader = author ? (
    <div className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="p-3">
        <div className="flex items-center justify-between">
          <div className="flex-1 min-w-0">
            <h2 className="text-lg font-semibold truncate">
              {author.canonical_name || author.email.split('@')[0]}
            </h2>
            <div className="text-sm text-muted-foreground truncate">{author.email}</div>
            <div className="flex items-center gap-4 text-xs text-muted-foreground mt-1">
              <span>{author.email_count} emails</span>
              <span>{author.thread_count} threads</span>
              {author.first_email_date && (
                <span>First: {new Date(author.first_email_date).toLocaleDateString()}</span>
              )}
              {author.last_email_date && (
                <span>Last: {new Date(author.last_email_date).toLocaleDateString()}</span>
              )}
            </div>
          </div>
          <Button
            variant="ghost"
            size="sm"
            onClick={clearAuthorFilter}
            className="h-8 w-8 p-0 shrink-0"
            title="Clear author filter"
          >
            <X className="h-4 w-4" />
          </Button>
        </div>
      </div>
      
      {/* Tabs */}
      <div className="flex border-t">
        <button
          onClick={() => handleTabChange('created')}
          className={`flex-1 px-3 py-2 text-sm font-medium border-b-2 transition-all duration-150 ${
            activeTab === 'created'
              ? 'border-primary text-foreground bg-background/50'
              : 'border-transparent text-muted-foreground hover:text-foreground hover:bg-background/30'
          }`}
        >
          Created ({createdCountLabel})
        </button>
        <button
          onClick={() => handleTabChange('participated')}
          className={`flex-1 px-3 py-2 text-sm font-medium border-b-2 transition-all duration-150 ${
            activeTab === 'participated'
              ? 'border-primary text-foreground bg-background/50'
              : 'border-transparent text-muted-foreground hover:text-foreground hover:bg-background/30'
          }`}
        >
          Participated ({participatedCountLabel})
        </button>
      </div>
    </div>
  ) : null;

  return (
    <ThreadBrowserLayout
      threads={threads}
      loading={loading || isAuthorThreadsLoading}
      selectedThreadId={selectedThread?.id || null}
      onThreadSelect={handleThreadSelect}
      currentPage={currentPage}
      hasMore={hasMore}
      onPageChange={handlePageChange}
      filters={filters}
      onFiltersChange={handleFiltersChange}
      onSearch={handleSearch}
      searchQuery={searchQuery}
      totalThreads={isAuthorThreadsLoading ? null : totalThreads}
      maxPage={maxPage}
      leftPanelHeader={authorHeader}
    />
  );
}
