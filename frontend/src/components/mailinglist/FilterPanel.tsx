import { SlidersHorizontal, SortAsc, SortDesc } from 'lucide-react';
import { Button } from '../ui/button';
import type { ThreadSortBy, SortOrder, SearchType } from '../../types';

interface FilterPanelProps {
  searchType: SearchType;
  setSearchType: (type: SearchType) => void;
  sortBy: ThreadSortBy;
  order: SortOrder;
  onSortChange: (sortBy: ThreadSortBy) => void;
  isExpanded: boolean;
  onToggle: () => void;
}

export function FilterPanel({
  searchType,
  setSearchType,
  sortBy,
  order,
  onSortChange,
  isExpanded,
  onToggle,
}: FilterPanelProps) {
  return (
    <div className="space-y-3">
      {/* Filter toggle button */}
      <Button
        variant="ghost"
        size="sm"
        onClick={onToggle}
        className="w-full justify-start text-xs"
      >
        <SlidersHorizontal className="h-3.5 w-3.5 mr-2" />
        {isExpanded ? 'Hide' : 'Show'} Filters
      </Button>

      {/* Collapsible filter content */}
      {isExpanded && (
        <div className="space-y-3 animate-fade-in">
          {/* Search type */}
          <div>
            <label className="text-xs font-medium text-muted-foreground mb-2 block">
              Search In:
            </label>
            <div className="flex gap-1">
              <Button
                variant={searchType === 'subject' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setSearchType('subject')}
                className="flex-1 h-8 text-xs"
              >
                Subject
              </Button>
              <Button
                variant={searchType === 'full_text' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setSearchType('full_text')}
                className="flex-1 h-8 text-xs"
              >
                Full Text
              </Button>
            </div>
          </div>

          {/* Sort controls */}
          <div>
            <label className="text-xs font-medium text-muted-foreground mb-2 block">
              Sort By:
            </label>
            <div className="flex gap-1">
              <Button
                variant={sortBy === 'last_date' ? 'secondary' : 'ghost'}
                size="sm"
                onClick={() => onSortChange('last_date')}
                className="flex-1 h-8 text-xs"
              >
                Last
                {sortBy === 'last_date' && (
                  order === 'desc' ? (
                    <SortDesc className="ml-1 h-3 w-3" />
                  ) : (
                    <SortAsc className="ml-1 h-3 w-3" />
                  )
                )}
              </Button>
              <Button
                variant={sortBy === 'start_date' ? 'secondary' : 'ghost'}
                size="sm"
                onClick={() => onSortChange('start_date')}
                className="flex-1 h-8 text-xs"
              >
                Start
                {sortBy === 'start_date' && (
                  order === 'desc' ? (
                    <SortDesc className="ml-1 h-3 w-3" />
                  ) : (
                    <SortAsc className="ml-1 h-3 w-3" />
                  )
                )}
              </Button>
              <Button
                variant={sortBy === 'message_count' ? 'secondary' : 'ghost'}
                size="sm"
                onClick={() => onSortChange('message_count')}
                className="flex-1 h-8 text-xs"
              >
                Count
                {sortBy === 'message_count' && (
                  order === 'desc' ? (
                    <SortDesc className="ml-1 h-3 w-3" />
                  ) : (
                    <SortAsc className="ml-1 h-3 w-3" />
                  )
                )}
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
