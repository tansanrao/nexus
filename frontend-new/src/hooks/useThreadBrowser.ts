import { useCallback, useEffect, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import { apiClient } from '../lib/api';
import { useApiConfig } from '../contexts/ApiConfigContext';
import type { Thread } from '../types';
import type { ThreadFilters } from '../components/ThreadListHeader';

interface UseThreadBrowserOptions {
  authorId?: string;
  threadsCreated?: Thread[] | null;
  threadsParticipated?: Thread[] | null;
  activeTab?: 'created' | 'participated';
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
  const pageSize = 50;

  const {
    authorId,
    threadsCreated: providedThreadsCreated,
    threadsParticipated: providedThreadsParticipated,
    activeTab = 'created',
  } = options;

  useEffect(() => {
    if (selectedMailingList) {
      setCurrentPage(1);
      loadThreads(1);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedMailingList, authorId, activeTab, providedThreadsCreated, providedThreadsParticipated]);

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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [searchParams, threads, selectedMailingList]);

  // Auto-select first thread when threads are loaded and no thread is selected
  useEffect(() => {
    if (threads.length > 0 && !selectedThread) {
      setSelectedThread(threads[0]);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [threads]);

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
        let threadsData: Thread[] = [];
        let page = requestedPage;
        let totalFromApi: number | null = null;
        let forcedLastPage = false;

        if (authorId) {
          const sourceThreadsRaw =
            activeTab === 'created' ? providedThreadsCreated : providedThreadsParticipated;

          if (sourceThreadsRaw == null) {
            setThreads([]);
            setHasMore(false);
            setMaxPage(1);
            setTotalThreads(null);
            setLoading(false);
            return;
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

          const startIndex = (page - 1) * pageSize;
          const endIndex = startIndex + pageSize;
          threadsData = filteredThreads.slice(startIndex, endIndex);
          totalFromApi = filteredThreads.length;
        } else {
          const fetchPage = async (targetPage: number) => {
            if (searchTerm) {
              return apiClient.searchThreadsWithTotal(
                selectedMailingList,
                searchTerm,
                activeFilters.searchType,
                targetPage,
                pageSize,
                activeFilters.sortBy,
                activeFilters.order
              );
            }
            return apiClient.getThreadsWithTotal(
              selectedMailingList,
              targetPage,
              pageSize,
              activeFilters.sortBy,
              activeFilters.order
            );
          };

          let currentPageToFetch = page;
          let result = await fetchPage(currentPageToFetch);
          threadsData = result.items;
          totalFromApi = typeof result.total === 'number' ? result.total : null;

          while (threadsData.length === 0 && currentPageToFetch > 1) {
            forcedLastPage = true;
            currentPageToFetch -= 1;
            page = currentPageToFetch;
            result = await fetchPage(currentPageToFetch);
            threadsData = result.items;
            totalFromApi = typeof result.total === 'number' ? result.total : null;

            if (threadsData.length > 0 || currentPageToFetch === 1) {
              break;
            }
          }
        }

        setThreads(threadsData);

        const shownItems = (page - 1) * pageSize + threadsData.length;
        const hasMorePages = forcedLastPage
          ? false
          : totalFromApi != null
          ? shownItems < totalFromApi
          : threadsData.length === pageSize;
        const nextMaxPage =
          totalFromApi != null
            ? Math.max(1, Math.ceil(totalFromApi / pageSize))
            : forcedLastPage
            ? page
            : hasMorePages
            ? page + 1
            : page;
        const resolvedTotal =
          totalFromApi != null
            ? totalFromApi
            : hasMorePages
            ? null
            : shownItems;

        setHasMore(hasMorePages);
        setMaxPage(nextMaxPage);
        setTotalThreads(resolvedTotal);
        setCurrentPage(page);
      } catch (err) {
        console.error('Error loading threads:', err);
      } finally {
        setLoading(false);
      }
    },
    [
      activeTab,
      authorId,
      filters,
      pageSize,
      providedThreadsCreated,
      providedThreadsParticipated,
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
