export function AboutIntro() {
  return (
    <section className="container max-w-3xl space-y-5 pb-16 pt-24 text-center md:pb-20 md:pt-28 lg:pb-24 lg:pt-32">
      <span className="text-muted-foreground text-sm font-medium tracking-[0.18em] uppercase">
        about nexus
      </span>
      <h1 className="text-balance text-3xl font-semibold tracking-tight md:text-4xl">
        Open tools for working with the Linux kernel
      </h1>
      <p className="text-muted-foreground text-pretty text-base leading-relaxed md:text-lg">
        Nexus mirrors lore.kernel.org, keeps the archives searchable, and gives
        users a fast way to follow the threads and patches they care about.
        It&apos;s maintained in the open by people who work with the Linux
        kernel every day.
      </p>
    </section>
  );
}
