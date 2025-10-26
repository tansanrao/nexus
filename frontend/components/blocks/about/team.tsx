import Link from "next/link";

const maintainers = [
  {
    name: "Tanuj Ravi Rao",
    role: "PhD student @ Virginia Tech",
    blurb:
      "I'm an engineer turned researcher with a passion for the practical, real-world side of systems research. My interests include all things operating systems, distributed systems, and networking. Right now, I'm focused on improving dataplane performance through kernel extensions.",
    href: "https://tansanrao.com",
  },
  {
    name: "Egor Lukiyanov",
    role: "PhD student @ Virginia Tech",
    blurb:
      "As well as pursuing kernel extensions I have been exploring rust and wasm as potential research topics. Beyond computer science, I enjoy 3D modeling, photography and anything at the whims of my imagination.",
    href: "https://egor.lukiyanov.name/",
  },
];

export function AboutTeam() {
  return (
    <section className="container max-w-5xl pb-24 lg:pb-28">
      <div className="space-y-4 text-center">
        <h2 className="text-2xl font-semibold tracking-tight md:text-3xl">
          Maintainers
        </h2>
        <p className="text-muted-foreground mx-auto max-w-2xl text-pretty text-base leading-relaxed md:text-lg">
          Nexus is a two-person side project supported by the community. We keep the roadmap public, and contributions are welcome.
        </p>
      </div>

      <div className="mt-10 grid gap-6 md:mt-14 md:grid-cols-2">
        {maintainers.map((person) => (
          <article
            key={person.name}
            className="rounded-3xl border border-border/60 bg-card p-6 text-left shadow-xs md:p-8"
          >
            <div className="flex items-center gap-2">
              <h3 className="text-xl font-semibold tracking-tight">
                {person.name}
              </h3>
              <Link
                href={person.href}
                target="_blank"
                rel="noreferrer"
                className="text-sm font-medium text-muted-foreground underline underline-offset-4 hover:text-foreground"
              >
                Website
              </Link>
            </div>
            <p className="text-muted-foreground mt-1 text-sm uppercase tracking-[0.18em]">
              {person.role}
            </p>
            <p className="text-muted-foreground mt-3 text-base leading-relaxed">
              {person.blurb}
            </p>
          </article>
        ))}
      </div>
    </section>
  );
}
