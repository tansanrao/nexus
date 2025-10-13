import { Link, useLocation } from 'react-router-dom';
import { Mail, Users, Search } from 'lucide-react';
import { useMailingList } from '../../contexts/MailingListContext';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../ui/select';
import { Input } from '../ui/input';
import { cn } from '@/lib/utils';

export function MailingListHeader() {
  const location = useLocation();
  const { selectedMailingList, setSelectedMailingList, mailingLists, loading } = useMailingList();

  const isThreadsView = location.pathname.includes('/threads');
  const isAuthorsView = location.pathname.includes('/authors');

  return (
    <header className="h-14 border-b border-border bg-card px-3 md:px-6 py-3 flex items-center justify-between gap-2 md:gap-4">
      {/* Left Section: Mailing List Selector */}
      <div className="flex items-center gap-2 md:gap-3 flex-1 md:flex-none">
        {selectedMailingList && (
          <>
            <Mail className="hidden md:block h-4 w-4 text-muted-foreground flex-shrink-0" />
            <div className="w-full md:w-64">
              <Select
                value={selectedMailingList}
                onValueChange={setSelectedMailingList}
                disabled={loading}
              >
                <SelectTrigger className="h-9">
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
          </>
        )}
      </div>

      {/* Middle Section: View Tabs */}
      <nav className="flex items-center gap-1">
        <Link
          to="/threads"
          className={cn(
            "flex items-center gap-1 md:gap-2 px-2 md:px-4 py-2 rounded-md text-sm font-medium transition-colors",
            isThreadsView
              ? "bg-primary text-primary-foreground"
              : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
          )}
        >
          <Mail className="h-4 w-4" />
          <span className="hidden sm:inline">Threads</span>
        </Link>
        <Link
          to="/authors"
          className={cn(
            "flex items-center gap-1 md:gap-2 px-2 md:px-4 py-2 rounded-md text-sm font-medium transition-colors",
            isAuthorsView
              ? "bg-primary text-primary-foreground"
              : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
          )}
        >
          <Users className="h-4 w-4" />
          <span className="hidden sm:inline">Authors</span>
        </Link>
      </nav>

      {/* Right Section: Search Bar - hidden on mobile */}
      <div className="hidden lg:block w-72 relative flex-shrink-0">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground pointer-events-none" />
        <Input
          type="text"
          placeholder="Search threads, authors, messages..."
          className="pl-9 h-9"
          disabled
        />
        <kbd className="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border border-border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground opacity-100">
          Ctrl+K
        </kbd>
      </div>
    </header>
  );
}
