import { del, getJson, patchJson, postJson } from "./http"
import { normalizePaginated, normalizeResponse } from "./adapters"
import type {
  ApiResponse,
  CreateJobRequest,
  DatabaseStatusResponse,
  JobListParams,
  JobRecord,
  MessageResponse,
  NormalizedPaginatedResponse,
  NormalizedResponse,
  UpdateJobRequest,
} from "./types"

export async function getDatabaseStatus(): Promise<NormalizedResponse<DatabaseStatusResponse>> {
  const response = await getJson<ApiResponse<DatabaseStatusResponse>>("database/status", {
    client: "admin",
  })
  return normalizeResponse(response)
}

export async function resetDatabase(): Promise<MessageResponse> {
  const response = await postJson<ApiResponse<MessageResponse>>("database/reset", undefined, {
    client: "admin",
  })
  return response.data
}

export async function getDatabaseConfig(): Promise<NormalizedResponse<unknown>> {
  const response = await getJson<ApiResponse<unknown>>("database/config", {
    client: "admin",
  })
  return normalizeResponse(response)
}

export async function listJobs(
  params?: JobListParams
): Promise<NormalizedPaginatedResponse<JobRecord[]>> {
  const response = await getJson<ApiResponse<JobRecord[]>>("jobs", {
    client: "admin",
    searchParams: params ? { params } : undefined,
  })
  return normalizePaginated(response)
}

export async function createJob(body: CreateJobRequest): Promise<NormalizedResponse<JobRecord>> {
  const response = await postJson<ApiResponse<JobRecord>>("jobs", body, {
    client: "admin",
  })
  return normalizeResponse(response)
}

export async function getJob(jobId: number): Promise<NormalizedResponse<JobRecord>> {
  const response = await getJson<ApiResponse<JobRecord>>(`jobs/${jobId}`, {
    client: "admin",
  })
  return normalizeResponse(response)
}

export async function updateJob(
  jobId: number,
  body: UpdateJobRequest
): Promise<NormalizedResponse<JobRecord>> {
  const response = await patchJson<ApiResponse<JobRecord>>(`jobs/${jobId}`, body, {
    client: "admin",
  })
  return normalizeResponse(response)
}

export async function deleteJob(jobId: number): Promise<void> {
  await del(`jobs/${jobId}`, { client: "admin" })
}
