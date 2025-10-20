import { Filter, List, ArrowDown, ArrowUp } from 'lucide-react';
import { Button } from './ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuTrigger,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
} from './ui/dropdown-menu';
import { ThemeToggle } from './ThemeToggle';
import { SettingsDropdown } from './SettingsDropdown';
import type { ThreadFilters } from './ThreadListHeader';

interface TopBarProps {
  filters: ThreadFilters;
  onFiltersChange: (filters: ThreadFilters) => void;
  threadCount: number;
}

export function TopBar({ 
  filters, 
  onFiltersChange, 
  threadCount
}: TopBarProps) {
  const sortByLabels = {
    startDate: 'Start Date',
    lastDate: 'Last Activity',
    messageCount: 'Message Count',
  };


  const searchTypeLabels = {
    subject: 'Subject Only',
    fullText: 'Full Text',
  };

  return (
    <div className="sticky top-0 z-40 w-full border-b border-surface-border/60 shadow-sm" style={{ backgroundColor: 'hsl(var(--color-accent))' }}>
      <div className="h-full grid grid-cols-1 md:grid-cols-5">
        {/* Left section - Threads header and Sort/Filter controls */}
        <div className="md:col-span-2 flex h-10 items-center justify-between" style={{ borderRight: '3px solid hsl(var(--color-border) / 0.6)' }}>
          {/* Threads header */}
          <div className="flex items-center gap-2 group relative px-4">
            <h2 className="text-sm font-semibold text-accent-foreground">
              Threads
            </h2>
            {threadCount > 0 && (
              <span className="text-xs text-accent-foreground/70">
                ({threadCount})
              </span>
            )}
            {/* Hover stats tooltip */}
            <div className="absolute left-0 top-full mt-1 hidden group-hover:block z-50 rounded-md border bg-popover text-popover-foreground text-xs shadow-md p-2 min-w-48">
              <div className="flex items-center justify-between gap-6">
                <span className="text-muted-foreground">Total results</span>
                <span className="font-medium">{threadCount}</span>
              </div>
            </div>
          </div>
          
          {/* Sort and Filter controls aligned to the right side of threads column */}
          <div className="flex items-center gap-2 px-4">
            {/* Sort Direction Toggle */}
            <Button
              variant="ghost"
              size="sm"
              className="h-6 w-6 p-0 text-accent-foreground hover:bg-accent-foreground/10"
              onClick={() => onFiltersChange({ ...filters, order: filters.order === 'asc' ? 'desc' : 'asc' })}
              title={`Sort ${filters.order === 'asc' ? 'Descending' : 'Ascending'}`}
            >
              {filters.order === 'asc' ? (
                <ArrowUp className="h-3 w-3" />
              ) : (
                <ArrowDown className="h-3 w-3" />
              )}
            </Button>

            {/* Sort Options */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="sm" className="h-6 w-6 p-0 text-accent-foreground hover:bg-accent-foreground/10">
                  <List className="h-3 w-3" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-48 space-y-1" onCloseAutoFocus={(e) => e.preventDefault()}>
                <DropdownMenuLabel>Sort By</DropdownMenuLabel>
                <DropdownMenuRadioGroup
                  value={filters.sortBy}
                  onValueChange={(value) => {
                    onFiltersChange({ ...filters, sortBy: value as ThreadFilters['sortBy'] });
                  }}
                >
                  <DropdownMenuRadioItem value="lastDate">
                    {sortByLabels.lastDate}
                  </DropdownMenuRadioItem>
                  <DropdownMenuRadioItem value="startDate">
                    {sortByLabels.startDate}
                  </DropdownMenuRadioItem>
                  <DropdownMenuRadioItem value="messageCount">
                    {sortByLabels.messageCount}
                  </DropdownMenuRadioItem>
                </DropdownMenuRadioGroup>
              </DropdownMenuContent>
            </DropdownMenu>

            {/* Filter Options */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="sm" className="h-6 w-6 p-0 text-accent-foreground hover:bg-accent-foreground/10">
                  <Filter className="h-3 w-3" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-48 space-y-1" onCloseAutoFocus={(e) => e.preventDefault()}>
                <DropdownMenuLabel>Search Mode</DropdownMenuLabel>
                <DropdownMenuRadioGroup
                  value={filters.searchType}
                  onValueChange={(value) => {
                    onFiltersChange({ ...filters, searchType: value as ThreadFilters['searchType'] });
                  }}
                >
                  <DropdownMenuRadioItem value="subject">
                    {searchTypeLabels.subject}
                  </DropdownMenuRadioItem>
                  <DropdownMenuRadioItem value="fullText">
                    {searchTypeLabels.fullText}
                  </DropdownMenuRadioItem>
                </DropdownMenuRadioGroup>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
        
        {/* Right section - Theme and Settings */}
        <div className="md:col-span-3 flex h-10 items-center justify-end px-4">
          <div className="flex items-center space-x-2">
            <ThemeToggle />
            <SettingsDropdown />
          </div>
        </div>
      </div>
    </div>
  );
}
