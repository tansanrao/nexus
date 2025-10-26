import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import {
  cancelSync,
  getDatabaseConfig,
  getDatabaseStatus,
  getSyncStatus,
  queueSync,
  refreshSearchIndex,
  resetDatabase,
  resetSearchIndexes,
  startSync,
} from "../admin"
import type {
  IndexMaintenanceRequest,
  SearchRefreshRequest,
  SyncRequest,
} from "../types"
import { queryKeys } from "../queryKeys"

export function useSyncStatus() {
  return useQuery({
    queryKey: queryKeys.admin.syncStatus(),
    queryFn: () => getSyncStatus(),
    refetchInterval: 10_000,
  })
}

export function useDatabaseStatus() {
  return useQuery({
    queryKey: queryKeys.admin.databaseStatus(),
    queryFn: () => getDatabaseStatus(),
  })
}

export function useDatabaseConfig() {
  return useQuery({
    queryKey: queryKeys.admin.databaseConfig(),
    queryFn: () => getDatabaseConfig(),
  })
}

export function useStartSync() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: () => startSync(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.admin.syncStatus() })
    },
  })
}

export function useQueueSync() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (body: SyncRequest) => queueSync(body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.admin.syncStatus() })
    },
  })
}

export function useCancelSync() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: () => cancelSync(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.admin.syncStatus() })
    },
  })
}

export function useResetDatabase() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: () => resetDatabase(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.admin.databaseStatus() })
    },
  })
}

export function useRefreshSearchIndex() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (body: SearchRefreshRequest) => refreshSearchIndex(body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.admin.syncStatus() })
    },
  })
}

export function useResetSearchIndexes() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (body: IndexMaintenanceRequest) => resetSearchIndexes(body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.admin.syncStatus() })
    },
  })
}
