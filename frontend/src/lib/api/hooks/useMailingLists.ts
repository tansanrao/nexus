import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import {
  getMailingList,
  getMailingListWithRepos,
  listMailingListRepositories,
  listMailingLists,
  seedMailingLists,
  toggleMailingList,
} from "../mailingLists"
import type { ToggleRequest } from "../types"
import { queryKeys } from "../queryKeys"

export function useMailingLists() {
  return useQuery({
    queryKey: queryKeys.mailingLists.all(),
    queryFn: () => listMailingLists(),
  })
}

export function useMailingList(slug: string | undefined) {
  return useQuery({
    queryKey: slug ? queryKeys.mailingLists.detail(slug) : ["mailingLists", "detail", "empty"],
    queryFn: () => {
      if (!slug) {
        throw new Error("slug is required")
      }
      return getMailingList(slug)
    },
    enabled: Boolean(slug),
  })
}

export function useMailingListRepositories(slug: string | undefined) {
  return useQuery({
    queryKey: slug ? queryKeys.mailingLists.repositories(slug) : ["mailingLists", "repositories", "empty"],
    queryFn: () => {
      if (!slug) {
        throw new Error("slug is required")
      }
      return listMailingListRepositories(slug)
    },
    enabled: Boolean(slug),
  })
}

export function useMailingListWithRepos(slug: string | undefined) {
  return useQuery({
    queryKey: slug ? queryKeys.mailingLists.withRepos(slug) : ["mailingLists", "withRepos", "empty"],
    queryFn: () => {
      if (!slug) {
        throw new Error("slug is required")
      }
      return getMailingListWithRepos(slug)
    },
    enabled: Boolean(slug),
  })
}

export function useToggleMailingList() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ slug, body }: { slug: string; body: ToggleRequest }) => toggleMailingList(slug, body),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.mailingLists.all() })
      queryClient.invalidateQueries({ queryKey: queryKeys.mailingLists.detail(variables.slug) })
      queryClient.invalidateQueries({ queryKey: queryKeys.mailingLists.repositories(variables.slug) })
      queryClient.invalidateQueries({ queryKey: queryKeys.mailingLists.withRepos(variables.slug) })
    },
  })
}

export function useSeedMailingLists() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: () => seedMailingLists(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.mailingLists.all() })
    },
  })
}
