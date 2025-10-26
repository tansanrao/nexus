import { Monitor, Moon, Sun } from 'lucide-react';
import { Button } from './ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  DropdownMenuLabel,
} from './ui/dropdown-menu';
import { useThemeSettings } from '../contexts/theme-settings-context';
import { cn } from '../lib/utils';

export function ThemeToggle() {
  const {
    modePreference,
    lightSchemeId,
    darkSchemeId,
    setModePreference,
    setLightScheme,
    setDarkScheme,
  } = useThemeSettings();

  const isActive = (mode: 'light' | 'dark' | 'system', scheme?: string) => {
    if (modePreference !== mode) return false;
    if (mode === 'system') return true;
    if (mode === 'light') {
      return scheme ? lightSchemeId === scheme : true;
    }
    return scheme ? darkSchemeId === scheme : true;
  };

  const handleSelect = (mode: 'light' | 'dark' | 'system', scheme?: string) => {
    if (mode === 'system') {
      setModePreference('system');
      return;
    }

    setModePreference(mode);
    if (mode === 'light' && scheme) {
      setLightScheme(scheme);
    }
    if (mode === 'dark' && scheme) {
      setDarkScheme(scheme);
    }
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon" className="hover:bg-muted/70 bg-transparent">
          <Sun className="h-5 w-5 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
          <Moon className="absolute h-5 w-5 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
          <span className="sr-only">Toggle theme</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-56 space-y-0">
        <DropdownMenuLabel className="text-xs text-muted-foreground px-2 py-1.5">
          Interface
        </DropdownMenuLabel>
        <div className="space-y-0">
          <DropdownMenuItem
            onClick={() => handleSelect('light', 'light')}
            className={cn(
              'gap-2 px-2 py-1.5 text-sm',
              isActive('light', 'light') && 'text-primary font-semibold'
            )}
          >
            <Sun className="h-4 w-4" />
            Light
          </DropdownMenuItem>
          <DropdownMenuItem
            onClick={() => handleSelect('dark', 'dark')}
            className={cn(
              'gap-2 px-2 py-1.5 text-sm',
              isActive('dark', 'dark') && 'text-primary font-semibold'
            )}
          >
            <Moon className="h-4 w-4" />
            Dark
          </DropdownMenuItem>
          <DropdownMenuItem
            onClick={() => handleSelect('light', 'solarized-light')}
            className={cn(
              'gap-2 px-2 py-1.5 text-sm',
              isActive('light', 'solarized-light') && 'text-primary font-semibold'
            )}
          >
            <Sun className="h-4 w-4" />
            Solarized Light
          </DropdownMenuItem>
          <DropdownMenuItem
            onClick={() => handleSelect('dark', 'solarized-dark')}
            className={cn(
              'gap-2 px-2 py-1.5 text-sm',
              isActive('dark', 'solarized-dark') && 'text-primary font-semibold'
            )}
          >
            <Moon className="h-4 w-4" />
            Solarized Dark
          </DropdownMenuItem>
          <DropdownMenuItem
            onClick={() => handleSelect('system')}
            className={cn(
              'gap-2 px-2 py-1.5 text-sm',
              isActive('system') && 'text-primary font-semibold'
            )}
          >
            <Monitor className="h-4 w-4" />
            System
          </DropdownMenuItem>
        </div>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
