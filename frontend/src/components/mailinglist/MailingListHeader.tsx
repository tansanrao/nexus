import { Link, useLocation } from 'react-router-dom';
import { Filter, Mail, Users } from 'lucide-react';
import { useMailingList } from '../../contexts/MailingListContext';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../ui/select';
import { cn } from '@/lib/utils';
import { Separator } from '../ui/separator';

export function MailingListHeader() {
  const location = useLocation();
  const { selectedMailingList, setSelectedMailingList, mailingLists, loading } = useMailingList();

  const isThreadsView = location.pathname.includes('/threads');
  const isAuthorsView = location.pathname.includes('/authors');

  return (
    <header className="toolbar h-14 px-3 md:px-5 text-sm">
      <div className="flex flex-1 items-center gap-3 md:gap-4 min-w-0">
        <div className="flex items-center gap-2">
          <h2 className="text-xs font-semibold uppercase tracking-[0.14em] text-foreground">
            Mailing List Browser
          </h2>
          <Separator orientation="vertical" className="hidden md:block h-6 bg-border/70" decorative={false} />
        </div>
        <div className="hidden md:flex items-center gap-2 rounded-md border border-border/60 px-2 py-1">
          <Filter className="h-4 w-4 text-muted-foreground" />
          <span className="text-label">Mailing list</span>
        </div>
        {selectedMailingList && (
          <div className="w-40 sm:w-60">
            <Select
              value={selectedMailingList}
              onValueChange={setSelectedMailingList}
              disabled={loading}
            >
              <SelectTrigger className="h-9 text-sm">
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
        )}
        <nav className="flex items-center gap-1 rounded-md border border-border/60 bg-surface-muted px-1 py-1 text-xs uppercase tracking-[0.08em]">
          <Link
            to="/threads"
            className={cn(
              "flex items-center gap-1 px-2 py-1 rounded-sm transition-colors",
              isThreadsView ? "bg-surface-overlay text-foreground" : "text-muted-foreground hover:text-foreground"
            )}
          >
            <Mail className="h-4 w-4" />
            <span className="hidden sm:inline">Threads</span>
          </Link>
          <Link
            to="/authors"
            className={cn(
              "flex items-center gap-1 px-2 py-1 rounded-sm transition-colors",
              isAuthorsView ? "bg-surface-overlay text-foreground" : "text-muted-foreground hover:text-foreground"
            )}
          >
            <Users className="h-4 w-4" />
            <span className="hidden sm:inline">Authors</span>
          </Link>
        </nav>
      </div>

      <div className="hidden lg:flex items-center gap-2 text-label">
        <span>Search</span>
        <kbd className="inline-flex items-center gap-1 rounded border border-border/60 bg-surface-muted px-1.5 py-0.5 font-mono text-[11px] text-muted-foreground">
          Ctrl + K
        </kbd>
      </div>
    </header>
  );
}
