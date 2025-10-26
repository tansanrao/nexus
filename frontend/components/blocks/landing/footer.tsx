import Link from "next/link";

import { ArrowUpRight } from "lucide-react";

import { Button } from "@/components/ui/button";

export function Footer() {
  const navigation = [
    { name: "Features", href: "/#features" },
    // { name: "Workflows", href: "/#workflows" },
    { name: "About", href: "/about" },
    { name: "Contact", href: "/contact" },
  ];

  const social = [
    // { name: "Design doc", href: "/docs/design" },
    { name: "GitHub", href: "https://github.com/tansanrao/nexus" },
  ];

  const legal = [{ name: "Privacy Policy", href: "/privacy" }];

  return (
    <footer className="flex flex-col items-center gap-14 pt-28 lg:pt-32">
      <div className="container space-y-3 text-center">
        <h2 className="text-2xl tracking-tight md:text-4xl lg:text-5xl">
          Request access to Nexus
        </h2>
        <p className="text-muted-foreground mx-auto max-w-xl leading-snug text-balance">
          Nexus is an open-source browser and knowledge base for the Linux kernel—self-host it or join our community cloud.
        </p>
        <div>
          <Button size="lg" className="mt-4" asChild>
            <Link href="/signup">Request access</Link>
          </Button>
        </div>
      </div>

      <nav className="container flex flex-col items-center gap-4">
        <ul className="flex flex-wrap items-center justify-center gap-6">
          {navigation.map((item) => (
            <li key={item.name}>
              <Link
                href={item.href}
                className="font-medium transition-opacity hover:opacity-75"
              >
                {item.name}
              </Link>
            </li>
          ))}
          {social.map((item) => {
            const isExternal = item.href.startsWith("http");
            return (
              <li key={item.name}>
                <Link
                  href={item.href}
                  className="flex items-center gap-0.5 font-medium transition-opacity hover:opacity-75"
                  target={isExternal ? "_blank" : undefined}
                  rel={isExternal ? "noreferrer" : undefined}
                >
                  {item.name} <ArrowUpRight className="size-4" />
                </Link>
              </li>
            );
          })}
        </ul>
        <ul className="flex flex-wrap items-center justify-center gap-6">
          {legal.map((item) => (
            <li key={item.name}>
              <Link
                href={item.href}
                className="text-muted-foreground text-sm transition-opacity hover:opacity-75"
              >
                {item.name}
              </Link>
            </li>
          ))}
        </ul>
      </nav>

      <div className="mt-10 h-16 w-full md:mt-14 lg:mt-20" aria-hidden />
    </footer>
  );
}
