import Link from "next/link";

import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { cn } from "@/lib/utils";

const categories = [
  {
    title: "Product",
    questions: [
      {
        question: "What data does Nexus index?",
        answer:
          "Nexus mirrors public-inbox repositories from lore.kernel.org, covering upstream Linux kernel mailing lists as well as subsystem lists. The mirror pipeline preserves raw mbox content so you can replay or re-export threads at any time.",
      },
      {
        question: "How fresh are the archives?",
        answer:
          "Incremental sync jobs run continuously—grokmirror fetches new epochs, and Nexus ingests them through a streaming pipeline that de-duplicates messages, updates thread state, and refreshes search indexes. Fresh mail usually appears within minutes of hitting lore.",
      },
      {
        question: "How does search work?",
        answer:
          "Subject, full-text, and author queries are fused together. Nexus maintains PostgreSQL full-text indexes for lexical matches, pg_trgm for fuzzy subjects, and pgvector embeddings for semantic lookups. Results can be re-ranked so the most relevant mail floats to the top.",
      },
    ],
  },
  {
    title: "Self-hosting",
    questions: [
      {
        question: "What do I need to run Nexus locally?",
        answer:
          "The reference stack uses Docker Compose with Postgres 18 (+pgvector), the Rust API server (Rocket), grokmirror, and the Next.js frontend. You can also compile the services directly; just point Nexus to a Postgres instance and a writable mirror directory.",
      },
      {
        question: "Can I index private or custom archives?",
        answer:
          "Yes. Point grokmirror at any public-inbox compatible repository (local or remote) and register the list in the Nexus configuration. The importer will treat it like any lore.kernel.org source.",
      },
    ],
  },
  {
    title: "Using Nexus",
    questions: [
      {
        question: "Does Nexus detect patches automatically?",
        answer:
          "Every message is parsed for patch trailers, diffstats, and attachments. Nexus classifies patch series, links revisions, and highlights what changed so reviewers can skim without leaving the thread.",
      },
      {
        question: "Can I follow specific authors or topics?",
        answer:
          "You can save searches, pin authors, and subscribe to threads. Nexus keeps lightweight notification cursors so you know when a tracked conversation receives new mail.",
      },
    ],
  },
];

export const FAQ = ({
  headerTag = "h2",
  className,
  className2,
}: {
  headerTag?: "h1" | "h2";
  className?: string;
  className2?: string;
}) => {
  return (
    <section className={cn("py-28 lg:py-32", className)}>
      <div className="container max-w-5xl">
        <div className={cn("mx-auto grid gap-16 lg:grid-cols-2", className2)}>
          <div className="space-y-4">
            {headerTag === "h1" ? (
              <h1 className="text-2xl tracking-tight md:text-4xl lg:text-5xl">
                Got Questions?
              </h1>
            ) : (
              <h2 className="text-2xl tracking-tight md:text-4xl lg:text-5xl">
                Got Questions?
              </h2>
            )}
            <p className="text-muted-foreground max-w-md leading-snug lg:mx-auto">
              If you can&apos;t find what you&apos;re looking for,{" "}
              <Link href="/contact" className="underline underline-offset-4">
                get in touch
              </Link>
              .
            </p>
          </div>

          <div className="grid gap-6 text-start">
            {categories.map((category, categoryIndex) => (
              <div key={category.title} className="">
                <h3 className="text-muted-foreground border-b py-4">
                  {category.title}
                </h3>
                <Accordion type="single" collapsible className="w-full">
                  {category.questions.map((item, i) => (
                    <AccordionItem key={i} value={`${categoryIndex}-${i}`}>
                      <AccordionTrigger>{item.question}</AccordionTrigger>
                      <AccordionContent className="text-muted-foreground">
                        {item.answer}
                      </AccordionContent>
                    </AccordionItem>
                  ))}
                </Accordion>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
};
