import Link from "next/link";

export function ContactCommunity() {
  return (
    <section className="container max-w-4xl pb-24 lg:pb-32">
      <div className="rounded-3xl border border-border/60 bg-accent/10 px-6 py-10 text-center shadow-xs md:px-10 md:py-14">
        <h2 className="text-2xl font-semibold tracking-tight md:text-3xl">
          Contribute or deploy Nexus
        </h2>
        <p className="text-muted-foreground mx-auto mt-4 max-w-2xl text-base leading-relaxed md:text-lg">
          We track the roadmap and active work items in GitHub. If you want to
          help with hybrid search, UI polish, or new ingest connectors, grab an
          issue or open a proposal—every discussion happens in the open.
        </p>
        <div className="mt-8 flex flex-col items-center justify-center gap-3 sm:flex-row">
          <Link
            href="https://github.com/tansanrao/nexus"
            className="rounded-full border border-border bg-background px-5 py-3 text-sm font-medium transition-colors hover:bg-foreground hover:text-background"
          >
            View repository
          </Link>
          <Link
            href="https://github.com/tansanrao/nexus/issues"
            className="rounded-full border border-border/60 px-5 py-3 text-sm font-medium text-foreground transition-colors hover:border-foreground"
          >
            Browse open issues
          </Link>
        </div>
      </div>
    </section>
  );
}
