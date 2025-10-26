import type { components } from "./schema"

export type PageMetadata = components["schemas"]["PageMetadata"]

export type DataResponse<T> = {
  data: T
}

export type PaginatedResponse<T> = {
  data: T
  page: PageMetadata
}

export type MailingList = components["schemas"]["MailingList"]
export type MailingListWithRepos = components["schemas"]["MailingListWithRepos"]
export type MailingListRepository = components["schemas"]["MailingListRepository"]

export type Thread = components["schemas"]["Thread"]
export type ThreadDetail = components["schemas"]["ThreadDetail"]
export type ThreadWithStarter = components["schemas"]["ThreadWithStarter"]
export type ThreadSearchHit = components["schemas"]["ThreadSearchHit"]
export type ThreadListParams = components["schemas"]["ThreadListParams"]
export type ThreadSearchParams = components["schemas"]["ThreadSearchParams"]

export type AuthorWithStats = components["schemas"]["AuthorWithStats"]
export type EmailWithAuthor = components["schemas"]["EmailWithAuthor"]
export type EmailHierarchy = components["schemas"]["EmailHierarchy"]
export type StatsOverview = components["schemas"]["Stats"]
export type AuthorSearchParams = components["schemas"]["AuthorSearchParams"]

export type PaginationParams = components["schemas"]["PaginationParams"]
export type ToggleRequest = components["schemas"]["ToggleRequest"]
export type ToggleResponse = components["schemas"]["ToggleResponse"]
export type SearchRefreshRequest = components["schemas"]["SearchRefreshRequest"]
export type IndexMaintenanceRequest = components["schemas"]["IndexMaintenanceRequest"]
export type SyncRequest = components["schemas"]["SyncRequest"]
export type SyncStatusResponse = components["schemas"]["SyncStatusResponse"]
export type SyncStartResponse = components["schemas"]["SyncStartResponse"]
export type JobStatusInfo = components["schemas"]["JobStatusInfo"]
export type JobEnqueueResponse = components["schemas"]["JobEnqueueResponse"]
export type HealthResponse = components["schemas"]["HealthResponse"]
export type SeedResponse = components["schemas"]["SeedResponse"]
export type DatabaseStatusResponse = components["schemas"]["DatabaseStatusResponse"]
export type ThreadSearchResponse = components["schemas"]["ThreadSearchResponse"]
export type QueuedJobInfo = components["schemas"]["QueuedJobInfo"]
export type MessageResponse = components["schemas"]["MessageResponse"]
