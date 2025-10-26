import Image from "next/image";
import Link from "next/link";

import { ChevronRight } from "lucide-react";

import { DashedLine } from "./dashed-line";

import { Card, CardContent } from "@/components/ui/card";

const items = [
  {
    title: "Every list, continuously mirrored",
    description:
      "Mirror dozens of mailing lists locally with pagination, filters, and cross-list views.",
    image: "/features/triage-card.svg",
    href: "/app",
  },
  {
    title: "Hybrid search that keeps up",
    description:
      "Lexical ranking, trigram similarity, and activity boosts work together so the freshest, most relevant replies stay on top.",
    image: "/features/cycle-card.svg",
    href: "/app",
  },
  {
    title: "Patch-aware exploration",
    description:
      "Inline diffs, trailers, and author timelines make patches discoverable at a glance.",
    image: "/features/overview-card.svg",
    href: "/app",
  },
];

export const Features = () => {
  return (
    <section id="features" className="pb-28 lg:pb-32">
      <div className="container">
        {/* Top dashed line with text */}
        <div className="relative flex items-center justify-center">
          <DashedLine className="text-muted-foreground" />
          <span className="bg-muted text-muted-foreground absolute px-3 font-mono text-sm font-medium tracking-wide max-md:hidden">
            KERNEL MAILING LIST INSIGHTS
          </span>
        </div>

        {/* Content */}
        <div className="mx-auto mt-10 grid max-w-4xl items-center gap-3 md:gap-0 lg:mt-24 lg:grid-cols-2">
          <h2 className="text-2xl tracking-tight md:text-4xl lg:text-5xl">
            Built for kernel maintainers and contributors
          </h2>
          <p className="text-muted-foreground leading-snug">
            Nexus keeps the firehose manageable—quickly jump between lists,
            follow threads end-to-end, and surface the patch data reviewers rely
            on.
          </p>
        </div>

        {/* Features Card */}
        <Card className="mt-8 rounded-3xl md:mt-12 lg:mt-20">
          <CardContent className="flex p-0 max-md:flex-col">
            {items.map((item, i) => (
              <div key={i} className="flex flex-1 max-md:flex-col">
                <div className="flex-1 p-4 pe-0! md:p-6">
                  <div className="relative aspect-[1.28/1] overflow-hidden">
                    <Image
                      src={item.image}
                      alt={`${item.title} interface`}
                      fill
                      className="object-cover object-left-top ps-4 pt-2"
                    />
                    <div className="from-background absolute inset-0 z-10 bg-linear-to-t via-transparent to-transparent" />
                  </div>

                  <Link
                    href={item.href}
                    className="group flex items-center justify-between gap-4 pe-4 pt-4 md:pe-6 md:pt-6"
                  >
                    <div className="space-y-2 pe-4">
                      <h3 className="font-display max-w-60 text-2xl leading-tight font-bold tracking-tight">
                        {item.title}
                      </h3>
                      <p className="text-muted-foreground text-sm leading-snug">
                        {item.description}
                      </p>
                    </div>
                    <div className="rounded-full border p-2">
                      <ChevronRight className="size-6 transition-transform group-hover:translate-x-1 lg:size-9" />
                    </div>
                  </Link>
                </div>
                {i < items.length - 1 && (
                  <div className="relative hidden md:block">
                    <DashedLine orientation="vertical" />
                  </div>
                )}
                {i < items.length - 1 && (
                  <div className="relative block md:hidden">
                    <DashedLine orientation="horizontal" />
                  </div>
                )}
              </div>
            ))}
          </CardContent>
        </Card>
      </div>
    </section>
  );
};
