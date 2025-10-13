import { Link, useLocation } from 'react-router-dom';
import { Mail, Settings, Sun, Moon, Monitor } from 'lucide-react';
import { useTheme } from '../../contexts/ThemeContext';
import { cn } from '@/lib/utils';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../ui/select';

interface NavItem {
  path: string;
  icon: React.ElementType;
  label: string;
}

const navItems: NavItem[] = [
  { path: '/threads', icon: Mail, label: 'Mailing List' },
  { path: '/settings', icon: Settings, label: 'Settings' },
];

export function AppSidebar() {
  const location = useLocation();
  const { themeMode, setThemeMode } = useTheme();

  const isActive = (path: string) => {
    if (path === '/threads') {
      return location.pathname.startsWith('/threads') || location.pathname.startsWith('/authors');
    }
    return location.pathname.startsWith(path);
  };

  const getThemeIcon = () => {
    switch (themeMode) {
      case 'light':
        return Sun;
      case 'dark':
        return Moon;
      default:
        return Monitor;
    }
  };

  const ThemeIcon = getThemeIcon();

  return (
    <aside className="hidden md:flex w-16 xl:w-56 flex-shrink-0 border-r border-border bg-card flex-col">
      {/* Logo/Branding */}
      <div className="h-14 px-2 xl:px-4 flex items-center justify-center xl:justify-start border-b border-border">
        <Link to="/" className="flex items-center gap-2 group">
          <div className="h-8 w-8 rounded-lg bg-primary/10 flex items-center justify-center group-hover:bg-primary/20 transition-colors">
            <span className="text-base font-bold text-primary">LK</span>
          </div>
          <span className="hidden xl:block text-base font-bold tracking-tight">Kernel KB</span>
        </Link>
      </div>

      {/* Navigation */}
      <nav className="flex-1 p-3 space-y-1">
        {navItems.map((item) => {
          const Icon = item.icon;
          const active = isActive(item.path);

          return (
            <Link key={item.path} to={item.path}>
              <div
                className={cn(
                  "flex items-center justify-center xl:justify-start gap-3 px-2 xl:px-3 py-2 rounded-md text-sm font-medium transition-all duration-200",
                  active
                    ? "bg-primary/10 text-primary xl:border-l-4 xl:border-primary xl:pl-[8px]"
                    : "text-muted-foreground hover:bg-accent/50 hover:text-foreground"
                )}
              >
                <Icon className="h-4 w-4 flex-shrink-0" />
                <span className="hidden xl:inline">{item.label}</span>
              </div>
            </Link>
          );
        })}
      </nav>

      {/* Theme Toggle */}
      <div className="p-2 xl:p-3 border-t border-border">
        <div className="hidden xl:block mb-2">
          <label htmlFor="theme-mode-sidebar" className="text-xs font-medium text-muted-foreground mb-1.5 block">
            Theme Mode
          </label>
          <Select value={themeMode} onValueChange={setThemeMode}>
            <SelectTrigger id="theme-mode-sidebar" className="w-full h-9">
              <SelectValue>
                <div className="flex items-center gap-2">
                  <ThemeIcon className="h-3.5 w-3.5" />
                  <span className="text-xs capitalize">{themeMode}</span>
                </div>
              </SelectValue>
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="light">
                <div className="flex items-center gap-2">
                  <Sun className="h-3.5 w-3.5" />
                  <span>Light</span>
                </div>
              </SelectItem>
              <SelectItem value="dark">
                <div className="flex items-center gap-2">
                  <Moon className="h-3.5 w-3.5" />
                  <span>Dark</span>
                </div>
              </SelectItem>
              <SelectItem value="system">
                <div className="flex items-center gap-2">
                  <Monitor className="h-3.5 w-3.5" />
                  <span>System</span>
                </div>
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        {/* Compact theme toggle for narrow sidebar */}
        <div className="xl:hidden flex flex-col gap-1">
          <button
            onClick={() => setThemeMode('light')}
            className={cn(
              "p-2 rounded-md transition-colors",
              themeMode === 'light' ? "bg-primary/10 text-primary" : "text-muted-foreground hover:bg-accent/50"
            )}
            aria-label="Light mode"
          >
            <Sun className="h-4 w-4" />
          </button>
          <button
            onClick={() => setThemeMode('dark')}
            className={cn(
              "p-2 rounded-md transition-colors",
              themeMode === 'dark' ? "bg-primary/10 text-primary" : "text-muted-foreground hover:bg-accent/50"
            )}
            aria-label="Dark mode"
          >
            <Moon className="h-4 w-4" />
          </button>
          <button
            onClick={() => setThemeMode('system')}
            className={cn(
              "p-2 rounded-md transition-colors",
              themeMode === 'system' ? "bg-primary/10 text-primary" : "text-muted-foreground hover:bg-accent/50"
            )}
            aria-label="System mode"
          >
            <Monitor className="h-4 w-4" />
          </button>
        </div>
      </div>
    </aside>
  );
}
