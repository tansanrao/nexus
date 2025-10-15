import { Link, useLocation } from 'react-router-dom';
import { cn } from '@/lib/utils';
import { Separator } from './ui/separator';

export function SettingsSubHeader() {
  const location = useLocation();

  return (
    <div className="toolbar h-14 px-3 md:px-5 justify-between items-center text-sm">
      <div className="flex items-center gap-3">
        <h2 className="text-xs font-semibold uppercase tracking-[0.14em] text-foreground">
          Settings
        </h2>
        <Separator orientation="vertical" className="hidden md:block h-6 bg-border/70" decorative={false} />
      </div>
      <nav className="flex gap-2 text-xs uppercase tracking-[0.08em]">
        <Link
          to="/settings/general"
          className={cn(
            "px-2 py-1 border border-transparent hover:border-border hover:text-foreground transition-colors",
            location.pathname === '/settings/general' && "border-primary text-foreground"
          )}
        >
          General
        </Link>
        <Link
          to="/settings/database"
          className={cn(
            "px-2 py-1 border border-transparent hover:border-border hover:text-foreground transition-colors",
            location.pathname === '/settings/database' && "border-primary text-foreground"
          )}
        >
          Database
        </Link>
        <Link
          to="/settings/system"
          className={cn(
            "px-2 py-1 border border-transparent hover:border-border hover:text-foreground transition-colors",
            location.pathname === '/settings/system' && "border-primary text-foreground"
          )}
        >
          System Statistics
        </Link>
      </nav>
    </div>
  );
}
