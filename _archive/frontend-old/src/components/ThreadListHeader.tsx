// This component is now deprecated - functionality moved to TopBar
// Keeping only the type definitions for backward compatibility

export interface ThreadFilters {
  sortBy: 'startDate' | 'lastDate' | 'messageCount';
  order: 'asc' | 'desc';
}

// Empty component - all functionality moved to TopBar
export function ThreadListHeader() {
  return null;
}
