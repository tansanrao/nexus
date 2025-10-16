import { useState, useEffect, useRef } from 'react';
import { Filter, ArrowUpDown, Search } from 'lucide-react';
import { Button } from './ui/button';
import { Input } from './ui/input';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
} from './ui/dropdown-menu';

export interface ThreadFilters {
  sortBy: 'start_date' | 'last_date' | 'message_count';
  order: 'asc' | 'desc';
  searchType: 'subject' | 'full_text';
}

export interface ThreadListHeaderProps {
  filters: ThreadFilters;
  onFiltersChange: (filters: ThreadFilters) => void;
  threadCount: number;
  onSearch: (query: string) => void;
  searchQuery: string;
  // Additional aggregate stats can be added when available (e.g., authors total)
}

export function ThreadListHeader({ filters, onFiltersChange, threadCount, onSearch, searchQuery }: ThreadListHeaderProps) {
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

  const sortByLabels = {
    start_date: 'Start Date',
    last_date: 'Last Activity',
    message_count: 'Message Count',
  };

  const orderLabels = {
    asc: 'Ascending',
    desc: 'Descending',
  };

  const searchTypeLabels = {
    subject: 'Subject Only',
    full_text: 'Full Text',
  };

  return (
    <div className="border-b bg-background">
      <div className="px-3 py-2 flex items-center justify-between">
        <div className="flex items-center gap-2 group relative">
          <h2 className="text-sm font-semibold text-foreground">
            Threads
          </h2>
          {threadCount > 0 && (
            <span className="text-xs text-muted-foreground">
              ({threadCount})
            </span>
          )}
          {/* Hover stats tooltip - exact total results */}
          <div className="absolute left-0 top-full mt-1 hidden group-hover:block z-50 rounded-md border bg-popover text-popover-foreground text-xs shadow-md p-2 min-w-48">
            <div className="flex items-center justify-between gap-6">
              <span className="text-muted-foreground">Total results</span>
              <span className="font-medium">{threadCount}</span>
            </div>
          </div>
        </div>
        
        <div className="flex items-center gap-1">
          {/* Sort Options */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="sm" className="h-7 text-xs gap-1">
                <ArrowUpDown className="h-3 w-3" />
                Sort
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-48" onCloseAutoFocus={(e) => e.preventDefault()}>
              <DropdownMenuLabel>Sort By</DropdownMenuLabel>
              <DropdownMenuSeparator />
              <DropdownMenuRadioGroup
                value={filters.sortBy}
                onValueChange={(value) => {
                  onFiltersChange({ ...filters, sortBy: value as ThreadFilters['sortBy'] });
                }}
              >
                <DropdownMenuRadioItem value="last_date">
                  {sortByLabels.last_date}
                </DropdownMenuRadioItem>
                <DropdownMenuRadioItem value="start_date">
                  {sortByLabels.start_date}
                </DropdownMenuRadioItem>
                <DropdownMenuRadioItem value="message_count">
                  {sortByLabels.message_count}
                </DropdownMenuRadioItem>
              </DropdownMenuRadioGroup>
              
              <DropdownMenuSeparator />
              <DropdownMenuLabel>Order</DropdownMenuLabel>
              <DropdownMenuSeparator />
              <DropdownMenuRadioGroup
                value={filters.order}
                onValueChange={(value) => {
                  onFiltersChange({ ...filters, order: value as ThreadFilters['order'] });
                }}
              >
                <DropdownMenuRadioItem value="desc">
                  {orderLabels.desc}
                </DropdownMenuRadioItem>
                <DropdownMenuRadioItem value="asc">
                  {orderLabels.asc}
                </DropdownMenuRadioItem>
              </DropdownMenuRadioGroup>
            </DropdownMenuContent>
          </DropdownMenu>

          {/* Filter Options */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="sm" className="h-7 text-xs gap-1">
                <Filter className="h-3 w-3" />
                Filter
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-48" onCloseAutoFocus={(e) => e.preventDefault()}>
              <DropdownMenuLabel>Search Mode</DropdownMenuLabel>
              <DropdownMenuSeparator />
              <DropdownMenuRadioGroup
                value={filters.searchType}
                onValueChange={(value) => {
                  onFiltersChange({ ...filters, searchType: value as ThreadFilters['searchType'] });
                }}
              >
                <DropdownMenuRadioItem value="subject">
                  {searchTypeLabels.subject}
                </DropdownMenuRadioItem>
                <DropdownMenuRadioItem value="full_text">
                  {searchTypeLabels.full_text}
                </DropdownMenuRadioItem>
              </DropdownMenuRadioGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
      
      {/* Search bar */}
      <div className="px-3 pb-2">
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
          <kbd className="absolute right-2 top-2 pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground opacity-100">
            /
          </kbd>
        </div>
      </div>
    </div>
  );
}

