import { useQuery } from "@tanstack/react-query"
import { getEmail } from "../emails"
import { queryKeys } from "../queryKeys"

export function useEmail(slug: string | undefined, emailId: number | undefined) {
  return useQuery({
    queryKey: slug && emailId !== undefined ? queryKeys.emails.detail(slug, emailId) : ["emails", "detail", "empty"],
    queryFn: () => {
      if (!slug || emailId === undefined) {
        throw new Error("slug and emailId are required")
      }
      return getEmail(slug, emailId)
    },
    enabled: Boolean(slug && emailId !== undefined),
    staleTime: 1000 * 30,
  })
}
