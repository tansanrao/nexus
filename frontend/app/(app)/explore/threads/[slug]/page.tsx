import { redirect } from "next/navigation"

type ThreadsSlugPageProps = {
  params: { slug: string }
}

export default function ThreadsSlugRedirect({
  params,
}: ThreadsSlugPageProps): never {
  const slug = decodeURIComponent(params.slug)
  redirect(`/explore/threads/${encodeURIComponent(slug)}/1`)
}
