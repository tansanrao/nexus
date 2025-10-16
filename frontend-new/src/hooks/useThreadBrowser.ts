import { useCallback, useEffect, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import { apiClient } from '../lib/api';
import { useApiConfig } from '../contexts/ApiConfigContext';
import type { Thread, PaginatedResponse } from '../types';
import type { ThreadFilters } from '../components/ThreadListHeader';

interface FetchThreadsParams {
  mailingList: string;
  page: number;
  pageSize: number;
  filters: ThreadFilters;
  searchTerm: string;
}

interface UseThreadBrowserOptions {
  fetchThreads?: (params: FetchThreadsParams) => Promise<PaginatedResponse<Thread>>;
  reloadDeps?: unknown[];
  pageSize?: number;
}

export function useThreadBrowser(options: UseThreadBrowserOptions = {}) {
  const { selectedMailingList } = useApiConfig();
  const [searchParams, setSearchParams] = useSearchParams();
  const [threads, setThreads] = useState<Thread[]>([]);
  const [loading, setLoading] = useState(false);
  const [selectedThread, setSelectedThread] = useState<Thread | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [currentPage, setCurrentPage] = useState(1);
  const [hasMore, setHasMore] = useState(true);
  const [totalThreads, setTotalThreads] = useState<number | null>(null);
  const [maxPage, setMaxPage] = useState(1);
  const [filters, setFilters] = useState<ThreadFilters>({
    sortBy: 'last_date',
    order: 'desc',
    searchType: 'subject',
  });
  const pageSize = options.pageSize ?? 50;
  const reloadDeps = options.reloadDeps ?? [];
  const customFetchThreads = options.fetchThreads;

  useEffect(() => {
    if (selectedMailingList) {
      setCurrentPage(1);
      loadThreads(1);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- intentionally exclude loadThreads to avoid duplicate fetches when filters change
  }, [selectedMailingList, customFetchThreads, ...reloadDeps]);

  // If URL contains ?thread=ID, select that thread and ensure it's loaded
  useEffect(() => {
    const threadParam = searchParams.get('thread');
    if (threadParam) {
      const threadIdFromUrl = parseInt(threadParam, 10);
      if (Number.isFinite(threadIdFromUrl)) {
        const existing = threads.find(t => t.id === threadIdFromUrl);
        if (existing) {
          setSelectedThread(existing);
        } else if (selectedMailingList) {
          // Fetch single thread and set selection without replacing list
          apiClient
            .getThread(selectedMailingList, threadIdFromUrl)
            .then(detail => setSelectedThread(detail.thread))
            .catch(() => {});
        }
      }
    }
  }, [searchParams, threads, selectedMailingList]);

  // Auto-select first thread when threads are loaded and no thread is selected
  useEffect(() => {
    if (threads.length > 0 && !selectedThread) {
      setSelectedThread(threads[0]);
    }
  }, [threads, selectedThread]);

  const loadThreads = useCallback(
    async (
      requestedPage: number = currentPage,
      customFilters?: ThreadFilters,
      overrideSearchQuery?: string
    ) => {
      if (!selectedMailingList) return;

      setLoading(true);
      const activeFilters = customFilters || filters;
      const searchTerm = (overrideSearchQuery ?? searchQuery).trim();

      try {
        let currentPageToFetch = requestedPage;

        const fetchPage = async (targetPage: number) => {
          if (customFetchThreads) {
            return customFetchThreads({
              mailingList: selectedMailingList,
              page: targetPage,
              pageSize,
              filters: activeFilters,
              searchTerm,
            });
          }

          if (searchTerm) {
            return apiClient.searchThreads(
              selectedMailingList,
              searchTerm,
              activeFilters.searchType,
              targetPage,
              pageSize,
              activeFilters.sortBy,
              activeFilters.order
            );
          }

          return apiClient.getThreads(
            selectedMailingList,
            targetPage,
            pageSize,
            activeFilters.sortBy,
            activeFilters.order
          );
        };

        let result = await fetchPage(currentPageToFetch);
        let { totalPages: totalPagesFromApi } = result.page;

        if (totalPagesFromApi > 0 && currentPageToFetch > totalPagesFromApi) {
          currentPageToFetch = totalPagesFromApi;
          result = await fetchPage(currentPageToFetch);
          totalPagesFromApi = result.page.totalPages;
        }

        const threadsData = result.data;
        const totalElements =
          typeof result.page.totalElements === 'number'
            ? result.page.totalElements
            : threadsData.length;

        let resolvedPage =
          result.page.page && result.page.page > 0 ? result.page.page : currentPageToFetch;
        if (totalElements === 0) {
          resolvedPage = 1;
        }

        const totalPagesComputed =
          totalPagesFromApi && totalPagesFromApi > 0
            ? totalPagesFromApi
            : totalElements > 0
            ? Math.ceil(totalElements / pageSize)
            : 0;
        const normalizedMaxPage =
          totalPagesComputed > 0 ? totalPagesComputed : threadsData.length > 0 ? resolvedPage : 1;
        const hasMorePages =
          totalPagesComputed > 0 ? resolvedPage < totalPagesComputed : threadsData.length === pageSize;

        setThreads(threadsData);
        setHasMore(hasMorePages);
        setMaxPage(normalizedMaxPage);
        setTotalThreads(totalElements);
        setCurrentPage(Math.max(1, resolvedPage));
      } catch (err) {
        console.error('Error loading threads:', err);
      } finally {
        setLoading(false);
      }
    },
    [
      filters,
      customFetchThreads,
      currentPage,
      pageSize,
      searchQuery,
      selectedMailingList,
    ]
  );

  const handleSearch = useCallback(
    (query: string) => {
      setSearchQuery(query);
      setCurrentPage(1);
      loadThreads(1, undefined, query);
    },
    [loadThreads]
  );

  const handleThreadSelect = (thread: Thread) => {
    setSelectedThread(thread);
    // Update URL param
    const params = new URLSearchParams(searchParams);
    params.set('thread', String(thread.id));
    setSearchParams(params, { replace: false });
  };

  const handlePageChange = async (newPage: number) => {
    await loadThreads(newPage);
  };

  const handleFiltersChange = (newFilters: ThreadFilters) => {
    setFilters(newFilters);
    setCurrentPage(1);
    loadThreads(1, newFilters);
  };

  return {
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
  };
}
