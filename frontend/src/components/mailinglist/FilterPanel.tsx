import { SlidersHorizontal, SortAsc, SortDesc } from 'lucide-react';
import type { ThreadSortBy, SortOrder, SearchType } from '../../types';
import { CompactButton } from '../ui/compact-button';

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
    <div className="space-y-2">
      <CompactButton onClick={onToggle} aria-expanded={isExpanded}>
        <SlidersHorizontal className="h-3.5 w-3.5" />
        <span>{isExpanded ? 'Hide filters' : 'Show filters'}</span>
      </CompactButton>

      {isExpanded && (
        <div className="space-y-3 border border-border/50 rounded-sm p-3 text-[12px] leading-tight">
          <div className="space-y-2">
            <span className="block uppercase tracking-[0.08em] text-[11px] text-muted-foreground">
              Search In
            </span>
            <div className="grid grid-cols-2 gap-1">
              <CompactButton
                active={searchType === 'subject'}
                onClick={() => setSearchType('subject')}
              >
                Subject
              </CompactButton>
              <CompactButton
                active={searchType === 'fullText'}
                onClick={() => setSearchType('fullText')}
              >
                Full Text
              </CompactButton>
            </div>
          </div>

          <div className="space-y-2">
            <span className="block uppercase tracking-[0.08em] text-[11px] text-muted-foreground">
              Sort By
            </span>
            <div className="grid grid-cols-3 gap-1">
              <CompactButton
                active={sortBy === 'lastDate'}
                onClick={() => onSortChange('lastDate')}
              >
                Last
                {sortBy === 'lastDate' && (
                  order === 'desc' ? (
                    <SortDesc className="ml-1 h-3 w-3" />
                  ) : (
                    <SortAsc className="ml-1 h-3 w-3" />
                  )
                )}
              </CompactButton>
              <CompactButton
                active={sortBy === 'startDate'}
                onClick={() => onSortChange('startDate')}
              >
                Start
                {sortBy === 'startDate' && (
                  order === 'desc' ? (
                    <SortDesc className="ml-1 h-3 w-3" />
                  ) : (
                    <SortAsc className="ml-1 h-3 w-3" />
                  )
                )}
              </CompactButton>
              <CompactButton
                active={sortBy === 'messageCount'}
                onClick={() => onSortChange('messageCount')}
              >
                Count
                {sortBy === 'messageCount' && (
                  order === 'desc' ? (
                    <SortDesc className="ml-1 h-3 w-3" />
                  ) : (
                    <SortAsc className="ml-1 h-3 w-3" />
                  )
                )}
              </CompactButton>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
