import { type ReactNode, useEffect, useRef, useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { Grid3x3, Mail, Monitor, Moon, Settings2, Sun } from 'lucide-react';
import { CompactButton } from '../ui/compact-button';
import { cn } from '@/lib/utils';
import { useTheme } from '@/contexts/ThemeContext';
import type { LucideIcon } from 'lucide-react';

interface NavItem {
  path: string;
  label: string;
  icon: LucideIcon;
}

const navItems: NavItem[] = [
  {
    path: '/threads',
    label: 'Mailing List Browser',
    icon: Mail,
  },
  {
    path: '/settings',
    label: 'Settings',
    icon: Settings2,
  },
];

export function AppHeader() {
  return (
    <header className="relative z-30 h-14 border-b border-border/60 bg-background/95 text-foreground backdrop-blur flex items-center gap-3 px-3 md:px-5">
      <div className="flex items-center">
        <AppSwitcher />
      </div>

      <div className="flex-1 flex justify-center">
        <span className="text-sm md:text-base font-semibold uppercase tracking-[0.18em] text-foreground">
          NEXUS
        </span>
      </div>

      <div className="flex items-center gap-1">
        <ThemeModeButtons />
      </div>
    </header>
  );
}

function AppSwitcher() {
  const location = useLocation();
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    setOpen(false);
  }, [location.pathname]);

  useEffect(() => {
    if (!open) {
      return;
    }

    function handleClick(event: MouseEvent) {
      if (!containerRef.current) {
        return;
      }
      if (!containerRef.current.contains(event.target as Node)) {
        setOpen(false);
      }
    }

    function handleEscape(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        setOpen(false);
      }
    }

    document.addEventListener('mousedown', handleClick);
    document.addEventListener('keydown', handleEscape);
    return () => {
      document.removeEventListener('mousedown', handleClick);
      document.removeEventListener('keydown', handleEscape);
    };
  }, [open]);

  const isActive = (path: string) => {
    if (path === '/threads') {
      return location.pathname.startsWith('/threads') || location.pathname.startsWith('/authors');
    }
    return location.pathname.startsWith(path);
  };

  return (
    <div className="relative z-30" ref={containerRef}>
      <button
        type="button"
        aria-haspopup="true"
        aria-expanded={open}
        onClick={() => setOpen((prev) => !prev)}
        className="h-10 w-10 inline-flex items-center justify-center rounded-full border border-border/60 bg-surface-muted/60 text-muted-foreground hover:text-foreground hover:bg-surface-overlay transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"
      >
        <Grid3x3 className="h-5 w-5" />
        <span className="sr-only">Open app switcher</span>
      </button>

      {open && (
        <div className="absolute left-0 top-full z-50 mt-3 w-64 max-w-[90vw] rounded-2xl border border-border/60 bg-background shadow-2xl shadow-black/10">
          <nav className="grid grid-cols-2 gap-2 p-4 text-sm">
            {navItems.map((item) => (
              <AppSwitcherItem
                key={item.path}
                item={item}
                active={isActive(item.path)}
                onSelect={() => setOpen(false)}
              />
            ))}
          </nav>
        </div>
      )}
    </div>
  );
}

function AppSwitcherItem({ item, active, onSelect }: { item: NavItem; active: boolean; onSelect: () => void }) {
  const Icon = item.icon;

  return (
    <Link
      to={item.path}
      className={cn(
        'group flex flex-col items-center gap-2 rounded-xl border border-transparent bg-surface-muted/40 px-4 py-5 text-center transition-all',
        active ? 'border-primary/80 bg-surface-overlay text-foreground shadow-sm' : 'hover:border-border/80 hover:bg-surface-overlay/80 hover:text-foreground text-muted-foreground'
      )}
      onClick={onSelect}
    >
      <div className="h-12 w-12 rounded-full bg-primary/5 text-primary flex items-center justify-center transition-colors group-hover:bg-primary/10">
        <Icon className="h-5 w-5" />
      </div>
      <div className="text-xs font-semibold uppercase tracking-[0.1em]">{item.label}</div>
    </Link>
  );
}

function ThemeModeButtons() {
  const { modePreference, setModePreference } = useTheme();

  const buttons: Array<{ mode: 'light' | 'dark' | 'system'; label: string; icon: ReactNode }> = [
    { mode: 'light', label: 'Light', icon: <Sun className="h-3.5 w-3.5" /> },
    { mode: 'dark', label: 'Dark', icon: <Moon className="h-3.5 w-3.5" /> },
    { mode: 'system', label: 'Sys', icon: <Monitor className="h-3.5 w-3.5" /> },
  ];

  return (
    <div className="flex items-center gap-1 rounded-md border border-border/60 bg-surface-muted p-1">
      {buttons.map(({ mode, label, icon }) => (
        <CompactButton
          key={mode}
          active={modePreference === mode}
          onClick={() => setModePreference(mode)}
          aria-label={`Switch to ${label.toLowerCase()} theme`}
          className="px-2 py-1 text-xs uppercase tracking-[0.08em]"
        >
          <span className="hidden sm:inline">{label}</span>
          <span className="sm:hidden">{icon}</span>
        </CompactButton>
      ))}
    </div>
  );
}
