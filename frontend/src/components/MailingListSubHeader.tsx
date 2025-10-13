import { Link, useLocation } from 'react-router-dom';
import { useMailingList } from '../contexts/MailingListContext';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './ui/select';
import { cn } from '@/lib/utils';

export function MailingListSubHeader() {
  const location = useLocation();
  const { selectedMailingList, setSelectedMailingList, mailingLists, loading } = useMailingList();

  // Determine page title
  const getPageTitle = () => {
    if (location.pathname.startsWith('/settings')) {
      return 'Settings';
    }
    if (location.pathname.includes('/authors')) {
      return 'Authors';
    }
    if (location.pathname.includes('/threads')) {
      return 'Threads';
    }
    return 'Home';
  };

  const pageTitle = getPageTitle();

  // Check if we're on a mailing list page (not settings)
  const isMailingListPage = !location.pathname.startsWith('/settings');

  return (
    <div className="h-12 px-6 flex items-center justify-between border-b bg-card">
      {/* Left: Page Title */}
      <h2 className="text-base font-semibold">{pageTitle}</h2>

      {/* Right: Mailing List Controls (only for mailing list pages) */}
      {isMailingListPage && selectedMailingList && (
        <div className="flex items-center gap-4">
          {/* Mailing List Selector */}
          <div className="w-56">
            <Select
              value={selectedMailingList}
              onValueChange={setSelectedMailingList}
              disabled={loading}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select mailing list" />
              </SelectTrigger>
              <SelectContent>
                {mailingLists.map((list) => (
                  <SelectItem key={list.id} value={list.slug}>
                    {list.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Secondary Navigation */}
          <nav className="flex gap-1">
            <Link
              to="/threads"
              className={cn(
                "px-3 py-1.5 rounded-md text-sm font-medium transition-colors",
                location.pathname.includes('/threads')
                  ? "bg-primary text-primary-foreground"
                  : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
              )}
            >
              Threads
            </Link>
            <Link
              to="/authors"
              className={cn(
                "px-3 py-1.5 rounded-md text-sm font-medium transition-colors",
                location.pathname.includes('/authors')
                  ? "bg-primary text-primary-foreground"
                  : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
              )}
            >
              Authors
            </Link>
          </nav>
        </div>
      )}
    </div>
  );
}
