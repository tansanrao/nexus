import { Moon, Sun, Palette } from 'lucide-react';
import { useTheme } from 'next-themes';
import { Button } from './ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';

export function ThemeToggle() {
  const { setTheme } = useTheme();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon" className="hover:bg-muted/70">
          <Sun className="h-5 w-5 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
          <Moon className="absolute h-5 w-5 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
          <span className="sr-only">Toggle theme</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-40 space-y-0">
        <DropdownMenuItem onClick={() => setTheme('light')} className="gap-2 px-2 py-1">
          <Sun className="h-4 w-4" />
          <span className="text-sm">Light</span>
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => setTheme('dark')} className="gap-2 px-2 py-1">
          <Moon className="h-4 w-4" />
          <span className="text-sm">Dark</span>
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => setTheme('hackernews')} className="gap-2 px-2 py-1">
          <Palette className="h-4 w-4" />
          <span className="text-sm">Hacker News</span>
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => setTheme('solarized-light')} className="gap-2 px-2 py-1">
          <Sun className="h-4 w-4" />
          <span className="text-sm">Solarized Light</span>
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => setTheme('solarized-dark')} className="gap-2 px-2 py-1">
          <Moon className="h-4 w-4" />
          <span className="text-sm">Solarized Dark</span>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
