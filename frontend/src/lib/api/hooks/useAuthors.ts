import { useQuery } from "@tanstack/react-query"
import {
  getAuthor,
  getAuthorEmails,
  getAuthorThreadsParticipated,
  getAuthorThreadsStarted,
  searchAuthors,
} from "../authors"
import type { AuthorListParams, PaginationParams } from "../types"
import { queryKeys } from "../queryKeys"

export function useAuthorSearch(slug: string | undefined, params?: AuthorListParams) {
  return useQuery({
    queryKey: slug ? queryKeys.authors.search(slug, params) : ["authors", "search", "empty"],
    queryFn: () => {
      if (!slug) {
        throw new Error("slug is required")
      }
      return searchAuthors(slug, params)
    },
    enabled: Boolean(slug),
    staleTime: 1000 * 60 * 5,
  })
}

export function useAuthorDetail(slug: string | undefined, authorId: number | undefined) {
  return useQuery({
    queryKey: slug && authorId !== undefined ? queryKeys.authors.detail(slug, authorId) : ["authors", "detail", "empty"],
    queryFn: () => {
      if (!slug || authorId === undefined) {
        throw new Error("slug and authorId are required")
      }
      return getAuthor(slug, authorId)
    },
    enabled: Boolean(slug && authorId !== undefined),
    staleTime: 1000 * 60 * 5,
  })
}

export function useAuthorEmails(
  slug: string | undefined,
  authorId: number | undefined,
  params?: PaginationParams
) {
  return useQuery({
    queryKey:
      slug && authorId !== undefined
        ? queryKeys.authors.emails(slug, authorId, params)
        : ["authors", "emails", "empty"],
    queryFn: () => {
      if (!slug || authorId === undefined) {
        throw new Error("slug and authorId are required")
      }
      return getAuthorEmails(slug, authorId, params)
    },
    enabled: Boolean(slug && authorId !== undefined),
    staleTime: 1000 * 30,
  })
}

export function useAuthorThreadsStarted(
  slug: string | undefined,
  authorId: number | undefined,
  params?: PaginationParams
) {
  return useQuery({
    queryKey:
      slug && authorId !== undefined
        ? queryKeys.authors.threadsStarted(slug, authorId, params)
        : ["authors", "threads-started", "empty"],
    queryFn: () => {
      if (!slug || authorId === undefined) {
        throw new Error("slug and authorId are required")
      }
      return getAuthorThreadsStarted(slug, authorId, params)
    },
    enabled: Boolean(slug && authorId !== undefined),
    staleTime: 1000 * 30,
  })
}

export function useAuthorThreadsParticipated(
  slug: string | undefined,
  authorId: number | undefined,
  params?: PaginationParams
) {
  return useQuery({
    queryKey:
      slug && authorId !== undefined
        ? queryKeys.authors.threadsParticipated(slug, authorId, params)
        : ["authors", "threads-participated", "empty"],
    queryFn: () => {
      if (!slug || authorId === undefined) {
        throw new Error("slug and authorId are required")
      }
      return getAuthorThreadsParticipated(slug, authorId, params)
    },
    enabled: Boolean(slug && authorId !== undefined),
    staleTime: 1000 * 30,
  })
}
