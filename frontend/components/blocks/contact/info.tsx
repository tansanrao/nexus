import Link from "next/link";

const contactChannels = [
  {
    title: "Project discussions",
    body: (
      <>
        Join the{" "}
        <Link
          href="https://github.com/tansanrao/nexus/discussions"
          className="text-foreground underline underline-offset-4"
        >
          GitHub discussions board
        </Link>{" "}
        to share ideas, report issues, or ask for guidance on deployments.
      </>
    ),
  },
  {
    title: "Email",
    body: (
      <>
        Prefer email? Reach us at{" "}
        <Link
          href="mailto:nexus@tansanrao.com"
          className="text-foreground underline underline-offset-4"
        >
          nexus@tansanrao.com
        </Link>{" "}
        and we&apos;ll reply within a couple of business days.
      </>
    ),
  },
  {
    title: "Security",
    body: (
      <>
        Found a security issue? Please contact{" "}
        <Link
          href="mailto:security@tansanrao.com"
          className="text-foreground underline underline-offset-4"
        >
          security@tansanrao.com
        </Link>{" "}
        with details and we&apos;ll coordinate a responsible disclosure.
      </>
    ),
  },
];

export function ContactSummary() {
  return (
    <section className="container max-w-3xl space-y-6 pb-20 pt-24 md:pb-24 md:pt-28 lg:pb-28 lg:pt-32">
      <div className="text-center">
        <span className="text-muted-foreground text-sm font-medium tracking-[0.18em] uppercase">
          contact
        </span>
        <h1 className="mt-4 text-balance text-3xl font-semibold tracking-tight md:text-4xl lg:text-5xl">
          Let&apos;s keep the kernel conversation moving
        </h1>
        <p className="text-muted-foreground mx-auto mt-4 max-w-2xl text-base leading-relaxed md:text-lg">
          Nexus is community-driven. Whether you have feedback on the search
          pipeline, want to self-host mirrors, or need help debugging an ingest,
          we&apos;re happy to chat.
        </p>
      </div>

      <div className="mt-10 space-y-6 md:mt-14">
        {contactChannels.map((channel) => (
          <article
            key={channel.title}
            className="rounded-3xl border border-border/60 bg-card p-6 text-left shadow-xs md:p-8"
          >
            <h2 className="text-xl font-semibold tracking-tight">
              {channel.title}
            </h2>
            <p className="text-muted-foreground mt-3 text-base leading-relaxed">
              {channel.body}
            </p>
          </article>
        ))}
      </div>
    </section>
  );
}
