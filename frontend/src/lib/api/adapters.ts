import type {
  ApiResponse,
  NormalizedPaginatedResponse,
  NormalizedResponse,
  PaginationMeta,
  ResponseMeta,
} from "./types"

const EMPTY_META: ResponseMeta = {}

export function normalizeResponse<T>(input: ApiResponse<T>): NormalizedResponse<T> {
  const meta = input.meta ?? EMPTY_META
  return {
    data: input.data,
    meta,
  }
}

export function normalizePaginated<T>(
  input: ApiResponse<T>
): NormalizedPaginatedResponse<T> {
  const meta = input.meta ?? EMPTY_META
  const pagination = extractPagination(meta)
  return {
    data: input.data,
    meta,
    pagination,
  }
}

function extractPagination(meta: ResponseMeta): PaginationMeta {
  if (!meta.pagination) {
    throw new Error("Pagination metadata is missing from the API response.")
  }
  return meta.pagination
}
