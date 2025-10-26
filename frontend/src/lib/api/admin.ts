import { getJson, postJson } from "./http"
import type {
  DatabaseStatusResponse,
  IndexMaintenanceRequest,
  JobEnqueueResponse,
  MessageResponse,
  SearchRefreshRequest,
  SyncRequest,
  SyncStartResponse,
  SyncStatusResponse,
} from "./types"

export async function startSync(): Promise<MessageResponse> {
  return postJson<MessageResponse>("admin/sync/start")
}

export async function queueSync(body: SyncRequest): Promise<SyncStartResponse> {
  return postJson<SyncStartResponse>("admin/sync/queue", body)
}

export async function getSyncStatus(): Promise<SyncStatusResponse> {
  return getJson<SyncStatusResponse>("admin/sync/status")
}

export async function cancelSync(): Promise<MessageResponse> {
  return postJson<MessageResponse>("admin/sync/cancel")
}

export async function resetDatabase(): Promise<MessageResponse> {
  return postJson<MessageResponse>("admin/database/reset")
}

export async function getDatabaseStatus(): Promise<DatabaseStatusResponse> {
  return getJson<DatabaseStatusResponse>("admin/database/status")
}

export async function getDatabaseConfig(): Promise<Record<string, unknown>> {
  return getJson<Record<string, unknown>>("admin/database/config")
}

export async function refreshSearchIndex(body: SearchRefreshRequest): Promise<JobEnqueueResponse> {
  return postJson<JobEnqueueResponse>("admin/search/index/refresh", body)
}

export async function resetSearchIndexes(body: IndexMaintenanceRequest): Promise<JobEnqueueResponse> {
  return postJson<JobEnqueueResponse>("admin/search/index/reset", body)
}
