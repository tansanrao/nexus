import { useCallback, useEffect, useMemo, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import { X } from 'lucide-react';
import { Button } from '../components/ui/button';
import { ThreadBrowserLayout } from '../components/ThreadBrowserLayout';
import { useThreadBrowser } from '../hooks/useThreadBrowser';
import { apiClient } from '../lib/api';
import { useApiConfig } from '../contexts/ApiConfigContext';
import type { AuthorWithStats, Thread, PaginatedResponse } from '../types';
import type { ThreadFilters } from '../components/ThreadListHeader';

interface AuthorViewProps {
  authorId: string;
}

export function AuthorView({ authorId }: AuthorViewProps) {
  const { selectedMailingList } = useApiConfig();
  const [searchParams, setSearchParams] = useSearchParams();
  const [author, setAuthor] = useState<AuthorWithStats | null>(null);
  const [activeTab, setActiveTab] = useState<'created' | 'participated'>('created');
  const [createdTotal, setCreatedTotal] = useState<number | null>(null);
  const [participatedTotal, setParticipatedTotal] = useState<number | null>(null);

  const getEmptyPage = useCallback(
    (page: number, size: number): PaginatedResponse<Thread> => ({
      data: [],
      page: {
        page,
        size,
        totalPages: 0,
        totalElements: 0,
      },
    }),
    []
  );

  const fetchAuthorThreads = useCallback(
    async ({
      page,
      pageSize,
      filters: _filters,
      searchTerm,
      mailingList,
    }: {
      mailingList: string;
      page: number;
      pageSize: number;
      filters: ThreadFilters;
      searchTerm: string;
    }) => {
      const activeMailingList = selectedMailingList ?? mailingList;
      const authorIdNumber = parseInt(authorId, 10);

      if (!activeMailingList || !Number.isFinite(authorIdNumber)) {
        return getEmptyPage(page, pageSize);
      }

      const query = searchTerm.trim();
      const paginationParams = {
        page,
        size: pageSize,
      };

      const shouldUpdateTotals = query.length === 0;

      try {
        if (activeTab === 'created') {
          const response = await apiClient.getAuthorThreadsStarted(
            activeMailingList,
            authorIdNumber,
            paginationParams
          );
          if (shouldUpdateTotals) {
            setCreatedTotal(response.page.totalElements);
          }
          return response as PaginatedResponse<Thread>;
        }

        const response = await apiClient.getAuthorThreadsParticipated(
          activeMailingList,
          authorIdNumber,
          paginationParams
        );
        if (shouldUpdateTotals) {
          setParticipatedTotal(response.page.totalElements);
        }
        return response;
      } catch (err) {
        console.error('Error fetching author threads:', err);
        return getEmptyPage(page, pageSize);
      }
    },
    [activeTab, authorId, getEmptyPage, selectedMailingList]
  );

  // Use the shared thread browser hook
  const {
    threads,
    loading,
    selectedThread,
    searchQuery,
    currentPage,
    hasMore,
    maxPage,
    handleSearch,
    handleThreadSelect,
    handlePageChange,
  } = useThreadBrowser({
    fetchThreads: fetchAuthorThreads,
    reloadDeps: [authorId, activeTab],
  });

  const createdCountLabel = useMemo(() => {
    if (createdTotal === null) return '...';
    return createdTotal;
  }, [createdTotal]);

  const participatedCountLabel = useMemo(() => {
    if (participatedTotal === null) return '...';
    return participatedTotal;
  }, [participatedTotal]);

  const isAuthorThreadsLoading =
    author === null || createdTotal === null || participatedTotal === null;

  useEffect(() => {
    if (!selectedMailingList || !authorId) {
      return;
    }

    const mailingList = selectedMailingList;
    const authorIdNumber = parseInt(authorId, 10);
    if (!Number.isFinite(authorIdNumber)) {
      console.warn('Invalid author id for author view:', authorId);
      return;
    }

    setAuthor(null);
    setCreatedTotal(null);
    setParticipatedTotal(null);

    const loadAuthorData = async () => {
      try {
        const [authorData, createdPage, participatedPage] = await Promise.all([
          apiClient.getAuthor(mailingList, authorIdNumber),
          apiClient.getAuthorThreadsStarted(mailingList, authorIdNumber, { page: 1, size: 1 }),
          apiClient.getAuthorThreadsParticipated(mailingList, authorIdNumber, { page: 1, size: 1 }),
        ]);

        setAuthor(authorData);
        setCreatedTotal(createdPage.page.totalElements);
        setParticipatedTotal(participatedPage.page.totalElements);

        setActiveTab((prevTab) => {
          if (createdPage.page.totalElements > 0) {
            return 'created';
          }
          if (participatedPage.page.totalElements > 0) {
            return 'participated';
          }
          return prevTab;
        });
      } catch (err) {
        console.error('Error loading author:', err);
        setAuthor(null);
        setCreatedTotal(0);
        setParticipatedTotal(0);
      }
    };

    loadAuthorData();
  }, [selectedMailingList, authorId]);

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
    <div className="border-b border-surface-border/60 bg-surface-raised/95 backdrop-blur supports-[backdrop-filter]:bg-surface-raised/80 shadow-sm">
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
      <div className="flex border-t border-surface-border/60 bg-surface-raised/80">
        <button
          onClick={() => handleTabChange('created')}
          className={`flex-1 px-3 py-2 text-sm font-medium border-b-2 transition-all duration-150 ${
            activeTab === 'created'
              ? 'border-primary text-foreground bg-surface-inset'
              : 'border-transparent text-muted-foreground hover:text-foreground hover:bg-surface-inset/60'
          }`}
        >
          Created ({createdCountLabel})
        </button>
        <button
          onClick={() => handleTabChange('participated')}
          className={`flex-1 px-3 py-2 text-sm font-medium border-b-2 transition-all duration-150 ${
            activeTab === 'participated'
              ? 'border-primary text-foreground bg-surface-inset'
              : 'border-transparent text-muted-foreground hover:text-foreground hover:bg-surface-inset/60'
          }`}
        >
          Participated ({participatedCountLabel})
        </button>
      </div>
    </div>
  ) : null;

  return (
    <div className="h-full flex flex-col">
      <ThreadBrowserLayout
        threads={threads}
        loading={loading || isAuthorThreadsLoading}
        selectedThreadId={selectedThread?.id || null}
        onThreadSelect={handleThreadSelect}
        currentPage={currentPage}
        hasMore={hasMore}
        onPageChange={handlePageChange}
        maxPage={maxPage}
        onSearch={handleSearch}
        searchQuery={searchQuery}
        leftPanelHeader={authorHeader}
      />
    </div>
  );
}
