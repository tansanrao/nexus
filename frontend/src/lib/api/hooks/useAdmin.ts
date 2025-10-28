import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"

import {
  createJob,
  deleteJob,
  getDatabaseConfig,
  getDatabaseStatus,
  getJob,
  listJobs,
  resetDatabase,
  updateJob,
} from "../admin"
import type { CreateJobRequest, JobListParams, UpdateJobRequest } from "../types"
import { queryKeys } from "../queryKeys"

export function useDatabaseStatus() {
  return useQuery({
    queryKey: queryKeys.admin.databaseStatus(),
    queryFn: () => getDatabaseStatus(),
    select: (response) => response.data,
    refetchInterval: 30_000,
  })
}

export function useDatabaseConfig() {
  return useQuery({
    queryKey: queryKeys.admin.databaseConfig(),
    queryFn: () => getDatabaseConfig(),
    select: (response) => response.data,
  })
}

export function useJobs(params?: JobListParams) {
  return useQuery({
    queryKey: queryKeys.admin.jobs(params),
    queryFn: () => listJobs(params),
  })
}

export function useJob(jobId: number | undefined) {
  return useQuery({
    queryKey: typeof jobId === "number" ? queryKeys.admin.job(jobId) : ["admin", "jobs", "detail", "empty"],
    queryFn: () => {
      if (typeof jobId !== "number") {
        throw new Error("jobId is required")
      }
      return getJob(jobId)
    },
    enabled: typeof jobId === "number",
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

export function useCreateJob() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (body: CreateJobRequest) => createJob(body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.admin.jobs() })
    },
  })
}

export function useUpdateJob(jobId: number | undefined) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (body: UpdateJobRequest) => {
      if (typeof jobId !== "number") {
        throw new Error("jobId is required for update")
      }
      return updateJob(jobId, body)
    },
    onSuccess: () => {
      if (typeof jobId === "number") {
        queryClient.invalidateQueries({ queryKey: queryKeys.admin.job(jobId) })
      }
      queryClient.invalidateQueries({ queryKey: queryKeys.admin.jobs() })
    },
  })
}

export function useDeleteJob() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (jobId: number) => deleteJob(jobId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.admin.jobs() })
    },
  })
}
