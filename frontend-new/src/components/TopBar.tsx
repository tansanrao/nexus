import {
  Filter,
  List,
  ArrowDown,
  ArrowUp,
  PanelLeftClose,
  PanelLeftOpen,
  GitBranch,
  MessageSquare,
  Settings,
} from 'lucide-react';
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
import { MailingListSelector } from './MailingListSelector';
import type { ThreadFilters } from './ThreadListHeader';
import { cn } from '../lib/utils';
import { Link } from 'react-router-dom';

interface TopBarProps {
  filters: ThreadFilters;
  onFiltersChange: (filters: ThreadFilters) => void;
  threadCount: number;
  threadsCollapsed: boolean;
  onCollapseThreads: () => void;
  onExpandThreads: () => void;
  rightPanelView: 'thread' | 'diff';
  onRightPanelViewChange: (view: 'thread' | 'diff') => void;
}

export function TopBar({ 
  filters, 
  onFiltersChange, 
  threadCount,
  threadsCollapsed,
  onCollapseThreads,
  onExpandThreads,
  rightPanelView,
  onRightPanelViewChange,
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

  const collapseTitle = threadsCollapsed ? 'Expand thread list' : 'Collapse thread list';
  const collapseIcon = threadsCollapsed ? <PanelLeftOpen className="h-3 w-3" /> : <PanelLeftClose className="h-3 w-3" />;
  const isDiffView = rightPanelView === 'diff';
  const toggleTitle = isDiffView ? 'Show thread conversation' : 'Show combined git diffs';

  return (
    <div className="sticky top-0 z-40 w-full border-b border-surface-border/60 shadow-sm" style={{ backgroundColor: 'hsl(var(--color-accent))' }}>
      <div className="flex flex-col md:flex-row w-full">
        {/* Left section - Threads header and Sort/Filter controls */}
        <div
          className={cn(
            'flex h-10 w-full items-center border-b border-surface-border/60 md:border-b-0 transition-all duration-300 px-3 gap-2',
            threadsCollapsed
              ? 'md:w-16 md:min-w-[4rem] md:px-2'
              : 'md:w-[26rem] md:min-w-[18rem] md:px-4 gap-3'
          )}
          style={{
            borderRight: threadsCollapsed ? undefined : '3px solid hsl(var(--color-border) / 0.6)',
          }}
        >
          <Button
            variant="ghost"
            size="sm"
            className="h-6 w-6 p-0 text-accent-foreground hover:bg-accent-foreground/10"
            onClick={threadsCollapsed ? onExpandThreads : onCollapseThreads}
            title={collapseTitle}
            aria-label={collapseTitle}
          >
            {collapseIcon}
          </Button>

          {!threadsCollapsed && (
            <div className="flex w-full items-center gap-2">
              {/* Threads header */}
              <div className="flex items-center gap-2 group relative">
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
              <div className="ml-auto flex items-center gap-2">
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
          )}
        </div>
        
        {/* Right section - Theme and Settings */}
        <div className="flex h-10 w-full items-center justify-end gap-2 px-4 md:flex-1">
          <div className="flex items-center space-x-2">
            <MailingListSelector />
            <Button
              variant="ghost"
              size="sm"
              className={cn(
                'h-6 w-6 p-0 text-accent-foreground hover:bg-accent-foreground/10',
                isDiffView && 'bg-accent-foreground/10'
              )}
              onClick={() => onRightPanelViewChange(isDiffView ? 'thread' : 'diff')}
              title={toggleTitle}
              aria-label={toggleTitle}
              aria-pressed={isDiffView}
            >
              {isDiffView ? (
                <MessageSquare className="h-3 w-3" />
              ) : (
                <GitBranch className="h-3 w-3" />
              )}
            </Button>
            <ThemeToggle />
            <Button
              variant="ghost"
              size="icon"
              className="hover:bg-muted/70 bg-transparent"
              asChild
              title="Open settings"
              aria-label="Open settings"
            >
              <Link to="/settings/general">
                <Settings className="h-5 w-5" />
              </Link>
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
