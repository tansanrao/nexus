import { useCallback, useEffect, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import { apiClient } from '../lib/api';
import { useApiConfig } from '../contexts/ApiConfigContext';
import type { ThreadWithStarter, ThreadListItem, ThreadSearchResponse } from '../types';
import type { ThreadFilters } from '../components/ThreadListHeader';

interface FetchThreadsParams {
  mailingList: string;
  page: number;
  pageSize: number;
  filters: ThreadFilters;
  searchTerm: string;
  semanticRatio: number;
}

export interface ThreadFetchResult {
  items: ThreadListItem[];
  page: number;
  totalPages: number;
  total: number;
}

interface UseThreadBrowserOptions {
  fetchThreads?: (params: FetchThreadsParams) => Promise<ThreadFetchResult>;
  reloadDeps?: unknown[];
  pageSize?: number;
}

export function useThreadBrowser(options: UseThreadBrowserOptions = {}) {
  const { selectedMailingList } = useApiConfig();
  const [searchParams, setSearchParams] = useSearchParams();
  const [threadItems, setThreadItems] = useState<ThreadListItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [selectedThread, setSelectedThread] = useState<ThreadWithStarter | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [currentPage, setCurrentPage] = useState(1);
  const [hasMore, setHasMore] = useState(true);
  const [totalThreads, setTotalThreads] = useState<number | null>(null);
  const [maxPage, setMaxPage] = useState(1);
  const [filters, setFilters] = useState<ThreadFilters>({
    sortBy: 'lastDate',
    order: 'desc',
  });
  const [semanticRatio, setSemanticRatio] = useState(0.35);
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
        const existing = threadItems.find((item) => item.thread.id === threadIdFromUrl);
        if (existing) {
          setSelectedThread(existing.thread);
        } else if (selectedMailingList) {
          // Fetch single thread and set selection without replacing list
          apiClient
            .getThread(selectedMailingList, threadIdFromUrl)
            .then((detail) => setSelectedThread(detail.thread as ThreadWithStarter))
            .catch(() => {});
        }
      }
    }
  }, [searchParams, threadItems, selectedMailingList]);

  // Auto-select first thread when threads are loaded and no thread is selected
  useEffect(() => {
    if (threadItems.length === 0) {
      setSelectedThread(null);
      return;
    }

    if (selectedThread) {
      const stillVisible = threadItems.some((item) => item.thread.id === selectedThread.id);
      if (!stillVisible) {
        setSelectedThread(threadItems[0].thread);
      }
    } else {
      setSelectedThread(threadItems[0].thread);
    }
  }, [threadItems, selectedThread]);

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

        const fetchPage = async (targetPage: number): Promise<ThreadFetchResult> => {
          if (customFetchThreads) {
            return customFetchThreads({
              mailingList: selectedMailingList,
              page: targetPage,
              pageSize,
              filters: activeFilters,
              searchTerm,
              semanticRatio,
            });
          }

          if (searchTerm) {
            const response = await apiClient.searchThreads(
              selectedMailingList,
              searchTerm,
              targetPage,
              pageSize,
              semanticRatio,
            );
            const totalPages = response.total > 0 ? Math.ceil(response.total / response.size) : 0;
            return {
              items: mapSearchResultsToItems(response),
              page: response.page,
              totalPages,
              total: response.total,
            };
          }

          const result = await apiClient.getThreads(
            selectedMailingList,
            targetPage,
            pageSize,
            activeFilters.sortBy,
            activeFilters.order
          );
          return {
            items: result.data.map((thread) => ({ thread })),
            page: result.page.page ?? targetPage,
            totalPages: result.page.totalPages,
            total: result.page.totalElements,
          };
        };

        let result = await fetchPage(currentPageToFetch);
        let totalPagesFromApi = result.totalPages;

        if (totalPagesFromApi > 0 && currentPageToFetch > totalPagesFromApi) {
          currentPageToFetch = totalPagesFromApi;
          result = await fetchPage(currentPageToFetch);
          totalPagesFromApi = result.totalPages;
        }

        const totalElements = typeof result.total === 'number' ? result.total : threadItems.length;

        let resolvedPage = result.page && result.page > 0 ? result.page : currentPageToFetch;
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
          totalPagesComputed > 0 ? totalPagesComputed : result.items.length > 0 ? resolvedPage : 1;
        const hasMorePages =
          totalPagesComputed > 0 ? resolvedPage < totalPagesComputed : result.items.length === pageSize;

        setThreadItems(result.items);
        setHasMore(hasMorePages);
        setMaxPage(normalizedMaxPage);
        setTotalThreads(totalElements);
        setCurrentPage(Math.max(1, resolvedPage));

        // no additional search metadata to record when search is lexical-only
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
      threadItems,
      semanticRatio,
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

  const handleThreadSelect = (thread: ThreadWithStarter) => {
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

  const handleSemanticRatioChange = useCallback(
    (ratio: number) => {
      const clamped = Math.max(0, Math.min(1, ratio));
      setSemanticRatio(clamped);
      if (searchQuery.trim()) {
        setCurrentPage(1);
        loadThreads(1, undefined, searchQuery.trim());
      }
    },
    [loadThreads, searchQuery]
  );

  return {
    threads: threadItems,
    loading,
    selectedThread,
    searchQuery,
    currentPage,
    hasMore,
    totalThreads,
    maxPage,
    filters,
    pageSize,
    semanticRatio,
    handleSearch,
    handleThreadSelect,
    handlePageChange,
    handleFiltersChange,
    handleSemanticRatioChange,
  };
}

function mapSearchResultsToItems(response: ThreadSearchResponse): ThreadListItem[] {
  return response.results.map((hit) => ({
    thread: hit.thread,
    lexical_score: hit.lexical_score,
  }));
}
