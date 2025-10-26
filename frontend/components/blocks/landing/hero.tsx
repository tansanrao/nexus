import Image from "next/image";

import { ArrowRight, Bell, Layers, ScanSearch, Search } from "lucide-react";

import { DashedLine } from "./dashed-line";
import { Button } from "@/components/ui/button";

const features = [
  {
    title: "Mirror lore.kernel.org",
    description:
      "Continuously sync public-inbox archives so every kernel list is available offline.",
    icon: Layers,
  },
  {
    title: "Purpose-built search",
    description:
      "A hybrid Semantic + Lexical search engine tailored for kernel discussions.",
    icon: Search,
  },
  {
    title: "Patch intelligence",
    description:
      "Detect patches, group patch versions, and surface related patches so context lands alongside the mail.",
    icon: ScanSearch,
  },
  {
    title: "Stay in the loop",
    description:
      "Track authors and threads you care about and let notifications bring you back when conversations move.",
    icon: Bell,
  },
];

export const Hero = () => {
  return (
    <section className="py-28 lg:py-32 lg:pt-44">
      <div className="container flex flex-col justify-between gap-8 md:gap-14 lg:flex-row lg:gap-20">
        {/* Left side - Main content */}
        <div className="flex-1">
          <h1 className="text-foreground max-w-160 text-3xl tracking-tight md:text-4xl lg:text-5xl xl:whitespace-nowrap">
            Nexus
          </h1>

          <p className="text-muted-foreground text-xl mt-5 md:text-3xl">
            Nexus is an open-source browser and knowledge base that mirrors
            lore.kernel.org, reconstructs threads, and makes kernel discussion
            searchable for everyone.
          </p>

          <div className="mt-8 flex flex-wrap items-center gap-4 lg:flex-nowrap">
            <Button asChild>
              <a href="/signup">
                Request access
              </a>
            </Button>
            <Button
              variant="outline"
              className="from-background h-auto gap-2 bg-linear-to-r to-transparent shadow-md"
              asChild
            >
              <a href="/app" className="max-w-56 truncate text-start md:max-w-none">
                Explore the demo
                <ArrowRight className="stroke-3" />
              </a>
            </Button>
          </div>
        </div>

        {/* Right side - Features */}
        <div className="relative flex flex-1 flex-col justify-center space-y-5 max-lg:pt-10 lg:pl-10">
          <DashedLine
            orientation="vertical"
            className="absolute top-0 left-0 max-lg:hidden"
          />
          <DashedLine
            orientation="horizontal"
            className="absolute top-0 lg:hidden"
          />
          {features.map((feature) => {
            const Icon = feature.icon;
            return (
              <div key={feature.title} className="flex gap-2.5 lg:gap-5">
                <Icon className="text-foreground mt-1 size-4 shrink-0 lg:size-5" />
                <div>
                  <h2 className="font-text text-foreground font-semibold">
                    {feature.title}
                  </h2>
                  <p className="text-muted-foreground max-w-76 text-sm">
                    {feature.description}
                  </p>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      <div className="mt-12 max-lg:ml-6 max-lg:h-[550px] max-lg:overflow-hidden md:mt-20 lg:container lg:mt-24">
        <div className="relative h-[793px] w-full">
          <Image
            src="/hero.webp"
            alt="hero"
            fill
            className="rounded-2xl object-cover object-left-top shadow-lg max-lg:rounded-tr-none"
          />
        </div>
      </div>
    </section>
  );
};
