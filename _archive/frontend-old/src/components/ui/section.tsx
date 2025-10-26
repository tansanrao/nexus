import type { ReactNode } from 'react';
import { cn } from '../../lib/utils';

interface SectionProps {
  title: string;
  description?: string;
  actions?: ReactNode;
  className?: string;
  children: ReactNode;
}

export function Section({ title, description, actions, className, children }: SectionProps) {
  return (
    <section className={cn('surface p-4 sm:p-5 space-y-3', className)}>
      <header className="flex flex-wrap items-start justify-between gap-2">
        <div className="space-y-1">
          <h3 className="text-sm font-semibold uppercase tracking-[0.08em] text-muted-foreground">
            {title}
          </h3>
          {description && (
            <p className="text-xs text-muted-foreground/80 max-w-prose">
              {description}
            </p>
          )}
        </div>
        {actions && <div className="flex items-center gap-2 text-sm">{actions}</div>}
      </header>
      <div className="space-y-3 text-sm text-foreground">{children}</div>
    </section>
  );
}
