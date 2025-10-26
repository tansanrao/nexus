"use client"

import { ReactNode, useState } from "react"
import { QueryClient, QueryClientConfig, QueryClientProvider } from "@tanstack/react-query"
import { ReactQueryDevtools } from "@tanstack/react-query-devtools"

const enableDevtools = process.env.NEXT_PUBLIC_ENABLE_QUERY_DEVTOOLS === "true"

const defaultOptions: QueryClientConfig["defaultOptions"] = {
  queries: {
    refetchOnWindowFocus: false,
    retry: false,
  },
  mutations: {
    retry: false,
  },
}

export function QueryProvider({ children }: { children: ReactNode }) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions,
      })
  )

  return (
    <QueryClientProvider client={queryClient}>
      {children}
      {enableDevtools ? <ReactQueryDevtools initialIsOpen={false} /> : null}
    </QueryClientProvider>
  )
}
