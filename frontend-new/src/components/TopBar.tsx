import { useState, useEffect, useRef } from 'react';
import { Search } from 'lucide-react';
import { Input } from './ui/input';
import { ThemeToggle } from './ThemeToggle';
import { SettingsDropdown } from './SettingsDropdown';

interface TopBarProps {
  onSearch: (query: string) => void;
  searchQuery: string;
}

export function TopBar({ onSearch, searchQuery }: TopBarProps) {
  const [localQuery, setLocalQuery] = useState(searchQuery);
  const searchInputRef = useRef<HTMLInputElement>(null);

  const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setLocalQuery(value);
    onSearch(value);
  };

  // Handle "/" hotkey to focus search
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === '/' && !['INPUT', 'TEXTAREA'].includes((e.target as HTMLElement).tagName)) {
        e.preventDefault();
        searchInputRef.current?.focus();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  return (
    <div className="sticky top-0 z-40 w-full border-b border-surface-border/60 bg-surface-base/95 backdrop-blur supports-[backdrop-filter]:bg-surface-base/80 shadow-sm">
      <div className="flex h-14 items-center justify-center px-4 relative">
        <div className="flex-1 max-w-md">
          <div className="relative">
            <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              ref={searchInputRef}
              type="search"
              placeholder="Search threads..."
              className="pl-8 pr-12 h-9"
              value={localQuery}
              onChange={handleSearchChange}
            />
            <kbd className="absolute right-2 top-2 pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border border-surface-border/60 bg-surface-inset px-1.5 font-mono text-[10px] font-medium text-muted-foreground opacity-100">
              /
            </kbd>
          </div>
        </div>
        <div className="absolute right-4 flex items-center space-x-2">
          <ThemeToggle />
          <SettingsDropdown />
        </div>
      </div>
    </div>
  );
}
