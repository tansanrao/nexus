import { Link, useLocation } from 'react-router-dom';
import { X } from 'lucide-react';
import { cn } from '../lib/utils';
import { Button } from './ui/button';
import { ThemeToggle } from './ThemeToggle';

export function SettingsSubHeader() {
  const location = useLocation();

  const items = [
    { label: 'General', href: '/settings/general' },
    { label: 'Database', href: '/settings/database' },
    { label: 'System Statistics', href: '/settings/system' },
  ];

  return (
    <header
      className="sticky top-0 z-40 w-full border-b border-surface-border/60 shadow-sm"
      style={{ backgroundColor: 'hsl(var(--color-accent))' }}
    >
      <div className="flex h-12 items-center justify-between gap-3 px-3 md:px-6">
        <div className="flex items-center gap-2">
          <span className="text-sm font-semibold uppercase tracking-[0.12em] text-accent-foreground">
            Settings
          </span>
        </div>
        <div className="flex items-center gap-3 text-accent-foreground">
          <nav className="flex items-center gap-1 text-xs uppercase tracking-[0.08em] text-accent-foreground/80">
            {items.map((item) => (
              <Link
                key={item.href}
                to={item.href}
                className={cn(
                  'px-2 py-1 rounded-md transition-colors',
                  location.pathname === item.href
                    ? 'bg-accent-foreground/15 text-accent-foreground font-medium shadow-sm'
                    : 'hover:bg-accent-foreground/10 hover:text-accent-foreground'
                )}
              >
                {item.label}
              </Link>
            ))}
          </nav>
          <ThemeToggle />
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8 rounded-full text-accent-foreground hover:bg-accent-foreground/10 hover:text-accent-foreground"
            asChild
          >
            <Link to="/">
              <span className="sr-only">Close settings</span>
              <X className="h-4 w-4" />
            </Link>
          </Button>
        </div>
      </div>
    </header>
  );
}
