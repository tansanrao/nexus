import { Link, useLocation } from 'react-router-dom';
import { cn } from '@/lib/utils';

export function SettingsSubHeader() {
  const location = useLocation();

  return (
    <div className="h-14 px-6 py-3 flex items-center justify-between border-b border-border bg-card">
      {/* Left: Page Title */}
      <h2 className="text-base font-semibold">Settings</h2>

      {/* Right: Settings Navigation */}
      <nav className="flex gap-1">
        <Link
          to="/settings/general"
          className={cn(
            "px-3 py-1.5 rounded-md text-sm font-medium transition-colors",
            location.pathname === '/settings/general'
              ? "bg-primary text-primary-foreground"
              : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
          )}
        >
          General
        </Link>
        <Link
          to="/settings/database"
          className={cn(
            "px-3 py-1.5 rounded-md text-sm font-medium transition-colors",
            location.pathname === '/settings/database'
              ? "bg-primary text-primary-foreground"
              : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
          )}
        >
          Database
        </Link>
        <Link
          to="/settings/system"
          className={cn(
            "px-3 py-1.5 rounded-md text-sm font-medium transition-colors",
            location.pathname === '/settings/system'
              ? "bg-primary text-primary-foreground"
              : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
          )}
        >
          System Statistics
        </Link>
      </nav>
    </div>
  );
}
