import { ChevronLeft, ChevronRight } from 'lucide-react';
import { cn } from '../lib/utils';

interface PaginationProps {
  currentPage: number;
  maxPage: number;
  onPageChange: (page: number) => void;
  hasMore: boolean;
  className?: string;
}

export function Pagination({ 
  currentPage, 
  maxPage, 
  onPageChange, 
  hasMore, 
  className 
}: PaginationProps) {
  // Don't show pagination if there are no pages or only one page
  if (maxPage <= 1) {
    return null;
  }

  // Generate page numbers to show
  const getPageNumbers = () => {
    const pages: (number | string)[] = [];
    const maxVisible = 7; // Show up to 7 page numbers
    
    if (maxPage <= maxVisible) {
      // Show all pages if total is small
      for (let i = 1; i <= maxPage; i++) {
        pages.push(i);
      }
    } else {
      // Always show first page
      pages.push(1);
      
      if (currentPage <= 4) {
        // Near the beginning
        for (let i = 2; i <= Math.min(5, maxPage - 1); i++) {
          pages.push(i);
        }
        if (maxPage > 5) {
          pages.push('...');
        }
        pages.push(maxPage);
      } else if (currentPage >= maxPage - 3) {
        // Near the end
        pages.push('...');
        for (let i = Math.max(maxPage - 4, 2); i <= maxPage; i++) {
          pages.push(i);
        }
      } else {
        // In the middle
        pages.push('...');
        for (let i = currentPage - 1; i <= currentPage + 1; i++) {
          pages.push(i);
        }
        pages.push('...');
        pages.push(maxPage);
      }
    }
    
    return pages;
  };

  const pageNumbers = getPageNumbers();

  return (
    <div className={cn(
      "border-t p-2 flex items-center justify-between bg-background",
      className
    )}>
      <button
        onClick={() => onPageChange(currentPage - 1)}
        disabled={currentPage === 1}
        className={cn(
          'flex items-center gap-1 px-3 py-1.5 text-xs rounded border transition-colors select-none',
          currentPage === 1
            ? 'opacity-50 cursor-not-allowed'
            : 'hover:bg-accent cursor-pointer'
        )}
      >
        <ChevronLeft className="h-3 w-3" />
        Previous
      </button>
      
      <div className="flex items-center gap-1">
        {pageNumbers.map((page, index) => (
          <button
            key={index}
            onClick={() => typeof page === 'number' ? onPageChange(page) : undefined}
            disabled={page === '...'}
            className={cn(
              'px-2 py-1 text-xs rounded transition-colors select-none min-w-[24px]',
              page === '...'
                ? 'cursor-default text-muted-foreground'
                : page === currentPage
                ? 'bg-primary text-primary-foreground cursor-default'
                : 'hover:bg-accent cursor-pointer text-muted-foreground hover:text-foreground'
            )}
          >
            {page}
          </button>
        ))}
      </div>
      
      <button
        onClick={() => onPageChange(currentPage + 1)}
        disabled={!hasMore}
        className={cn(
          'flex items-center gap-1 px-3 py-1.5 text-xs rounded border transition-colors select-none',
          !hasMore
            ? 'opacity-50 cursor-not-allowed'
            : 'hover:bg-accent cursor-pointer'
        )}
      >
        Next
        <ChevronRight className="h-3 w-3" />
      </button>
    </div>
  );
}
