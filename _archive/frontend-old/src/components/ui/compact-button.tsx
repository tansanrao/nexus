import { forwardRef } from 'react';
import { cn } from '../../lib/utils';

export interface CompactButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  active?: boolean;
}

export const CompactButton = forwardRef<HTMLButtonElement, CompactButtonProps>(
  ({ active = false, className, type = 'button', children, ...props }, ref) => (
    <button
      ref={ref}
      type={type}
      className={cn(
        'inline-flex items-center justify-center gap-1 rounded-sm border border-border/60 px-2 py-1 text-[11px] uppercase tracking-[0.08em] text-muted-foreground transition-colors hover:border-border hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/40 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50',
        active && 'border-primary bg-primary/10 text-foreground',
        className
      )}
      {...props}
    >
      {children}
    </button>
  )
);

CompactButton.displayName = 'CompactButton';
