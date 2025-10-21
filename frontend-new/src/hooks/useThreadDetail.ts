import { useCallback, useEffect, useState } from 'react';
import { apiClient } from '../lib/api';
import type { ThreadDetail } from '../types';
import { useApiConfig } from '../contexts/ApiConfigContext';

interface UseThreadDetailResult {
  threadDetail: ThreadDetail | null;
  loading: boolean;
  error: string | null;
  reload: () => void;
}

export function useThreadDetail(threadId: number | null): UseThreadDetailResult {
  const { selectedMailingList } = useApiConfig();
  const [threadDetail, setThreadDetail] = useState<ThreadDetail | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadThreadDetail = useCallback(
    async (id: number) => {
      if (!selectedMailingList) {
        return;
      }

      setLoading(true);
      setError(null);
      try {
        const detail = await apiClient.getThread(selectedMailingList, id);
        setThreadDetail(detail);
      } catch (err) {
        console.error('Error loading thread detail:', err);
        setError('Failed to load thread details');
        setThreadDetail(null);
      } finally {
        setLoading(false);
      }
    },
    [selectedMailingList]
  );

  useEffect(() => {
    if (threadId && selectedMailingList) {
      void loadThreadDetail(threadId);
    } else {
      setThreadDetail(null);
      setError(null);
      setLoading(false);
    }
  }, [threadId, selectedMailingList, loadThreadDetail]);

  const reload = useCallback(() => {
    if (threadId) {
      void loadThreadDetail(threadId);
    }
  }, [threadId, loadThreadDetail]);

  return {
    threadDetail,
    loading,
    error,
    reload,
  };
}
