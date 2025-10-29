import { redirect } from "next/navigation"

type ThreadsSlugPageProps = {
  params: Promise<{ slug: string }>
}

export default async function ThreadsSlugRedirect({
  params,
}: ThreadsSlugPageProps): Promise<never> {
  const resolvedParams = await params
  const slug = decodeURIComponent(resolvedParams.slug)
  redirect(`/explore/threads/${encodeURIComponent(slug)}/1`)
}
