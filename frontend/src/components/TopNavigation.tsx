import { Link, useLocation } from 'react-router-dom';
import { Mail, Settings } from 'lucide-react';
import { cn } from '@/lib/utils';

export function TopNavigation() {
  const location = useLocation();

  return (
    <header className="h-14 px-6 flex items-center justify-between border-b bg-card">
      <Link to="/" className="flex items-center">
        <h1 className="text-lg font-semibold">Linux Kernel KB</h1>
      </Link>

      <nav className="flex items-center gap-1">
        <Link
          to="/threads"
          className={cn(
            "flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-colors",
            !location.pathname.startsWith('/settings')
              ? "bg-primary text-primary-foreground"
              : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
          )}
        >
          <Mail className="h-4 w-4" />
          Browse
        </Link>
        <Link
          to="/settings"
          className={cn(
            "flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-colors",
            location.pathname.startsWith('/settings')
              ? "bg-primary text-primary-foreground"
              : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
          )}
        >
          <Settings className="h-4 w-4" />
          Settings
        </Link>
      </nav>
    </header>
  );
}
