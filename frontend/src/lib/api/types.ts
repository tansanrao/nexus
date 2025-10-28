import type { components as PublicComponents } from "./schema"
import type { components as AdminComponents } from "./schema-admin"

type PublicSchemas = PublicComponents["schemas"]
type AdminSchemas = AdminComponents["schemas"]

export type ResponseMeta = PublicSchemas["ResponseMeta"]
export type PaginationMeta = PublicSchemas["PaginationMeta"]
export type SortDescriptor = PublicSchemas["SortDescriptor"]
export type SortDirection = PublicSchemas["SortDirection"]

export type ApiResponse<T> = {
  data: T
  meta?: ResponseMeta
}

export type NormalizedResponse<T> = {
  data: T
  meta: ResponseMeta
}

export type NormalizedPaginatedResponse<T> = NormalizedResponse<T> & {
  pagination: PaginationMeta
}

export type MailingList = PublicSchemas["MailingList"]
export type MailingListStats = PublicSchemas["MailingListStats"]
export type ListAggregateStats = PublicSchemas["ListAggregateStats"]
export type ListQueryParams = PublicSchemas["ListQueryParams"]
export type Thread = PublicSchemas["Thread"]
export type ThreadDetail = PublicSchemas["ThreadDetail"]
export type ThreadWithStarter = PublicSchemas["ThreadWithStarter"]
export type ThreadListParams = PublicSchemas["ThreadListParams"]
export type EmailHierarchy = PublicSchemas["EmailHierarchy"]
export type EmailWithAuthor = PublicSchemas["EmailWithAuthor"]
export type AuthorWithStats = PublicSchemas["AuthorWithStats"]
export type AuthorListParams = PublicSchemas["AuthorListParams"]
export type PaginationParams = PublicSchemas["PaginationParams"]
export type LoginRequest = PublicSchemas["LoginRequest"]
export type LoginResponse = PublicSchemas["LoginResponse"]
export type LogoutRequest = PublicSchemas["LogoutRequest"]
export type RefreshResponse = PublicSchemas["RefreshResponse"]
export type SessionResponse = PublicSchemas["SessionResponse"]
export type SigningKeyMetadata = PublicSchemas["SigningKeyMetadata"]
export type AuthErrorResponse = PublicSchemas["AuthErrorResponse"]
export type UserSummary = PublicSchemas["UserSummary"]

export type AdminMailingList = AdminSchemas["MailingList"]
export type AdminMailingListWithRepos = AdminSchemas["MailingListWithRepos"]
export type AdminMailingListRepository = AdminSchemas["MailingListRepository"]
export type ToggleRequest = AdminSchemas["ToggleRequest"]
export type ToggleResponse = AdminSchemas["ToggleResponse"]
export type SeedResponse = AdminSchemas["SeedResponse"]
export type DatabaseStatusResponse = AdminSchemas["DatabaseStatusResponse"]
export type MessageResponse = AdminSchemas["MessageResponse"]
export type JobListParams = AdminSchemas["JobListParams"]
export type JobRecord = AdminSchemas["JobRecord"]
export type JobStatus = AdminSchemas["JobStatus"]
export type JobType = AdminSchemas["JobType"]
export type CreateJobRequest = AdminSchemas["CreateJobRequest"]
export type UpdateJobRequest = AdminSchemas["UpdateJobRequest"]
