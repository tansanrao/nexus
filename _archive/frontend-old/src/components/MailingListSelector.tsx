import { useEffect, useState } from 'react';
import { Check, Loader2, RefreshCw } from 'lucide-react';
import { Button } from './ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';
import { useApiConfig } from '../contexts/ApiConfigContext';
import { apiClient } from '../lib/api';
import type { MailingList } from '../types';
import { cn } from '../lib/utils';
import { Link } from 'react-router-dom';

export function MailingListSelector() {
  const { selectedMailingList, setSelectedMailingList } = useApiConfig();
  const [mailingLists, setMailingLists] = useState<MailingList[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void loadMailingLists();
  }, []);

  useEffect(() => {
    if (!selectedMailingList && mailingLists.length > 0) {
      const firstEnabled = mailingLists.find((list) => list.enabled);
      if (firstEnabled) {
        setSelectedMailingList(firstEnabled.slug);
      }
    }
  }, [mailingLists, selectedMailingList, setSelectedMailingList]);

  const loadMailingLists = async () => {
    setLoading(true);
    setError(null);
    try {
      const lists = await apiClient.getMailingLists();
      setMailingLists(lists);
    } catch (err) {
      setError('Failed to load mailing lists');
      console.error('Error loading mailing lists:', err);
    } finally {
      setLoading(false);
    }
  };

  const enabledLists = mailingLists.filter((list) => list.enabled);
  const activeLabel =
    mailingLists.find((list) => list.slug === selectedMailingList)?.name || 'No list';

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          size="sm"
          className="h-6 px-2 text-xs uppercase tracking-[0.08em] text-accent-foreground hover:bg-accent-foreground/10"
        >
          {activeLabel}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-72 space-y-1">
        <div className="flex items-center justify-between gap-3 px-2 py-1.5">
          <div>
            <DropdownMenuLabel className="px-0 text-sm">Mailing Lists</DropdownMenuLabel>
            <p className="text-xs text-muted-foreground">Choose a list to browse</p>
          </div>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={() => void loadMailingLists()}
            disabled={loading}
            title="Refresh lists"
          >
            {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
          </Button>
        </div>
        <DropdownMenuSeparator />

        <div className="rounded-md border border-surface-border/60 bg-muted/20 p-1">
          <div className="max-h-52 overflow-y-auto space-y-1">
            {loading ? (
              <DropdownMenuItem disabled>Loading…</DropdownMenuItem>
            ) : enabledLists.length === 0 ? (
              <DropdownMenuItem disabled>No mailing lists available</DropdownMenuItem>
            ) : (
              enabledLists.map((list) => (
                <DropdownMenuItem
                  key={list.slug}
                  className="flex-col items-start gap-1.5 rounded-md px-2.5 py-1.5 data-[highlighted]:bg-primary/10"
                  onClick={() => setSelectedMailingList(list.slug)}
                >
                  <div className="flex w-full items-center gap-2">
                    <Check
                      className={cn(
                        'h-3.5 w-3.5 shrink-0 transition-opacity',
                        selectedMailingList === list.slug ? 'opacity-100' : 'opacity-0'
                      )}
                    />
                    <div className="flex flex-col">
                      <span className="font-medium leading-none">{list.name}</span>
                      <span className="text-[11px] text-muted-foreground">
                        {list.description || 'No description provided'}
                      </span>
                    </div>
                    <span className="ml-auto shrink-0 rounded-full bg-muted/60 px-2 py-0.5 text-[10px] uppercase tracking-wide text-muted-foreground">
                      {list.slug}
                    </span>
                  </div>
                </DropdownMenuItem>
              ))
            )}
          </div>
        </div>

        {error && (
          <>
            <DropdownMenuSeparator />
            <div className="px-2 py-1">
              <p className="text-xs text-destructive">{error}</p>
            </div>
          </>
        )}

        <DropdownMenuSeparator />
        <DropdownMenuItem asChild className="px-2.5 py-1.5 text-xs uppercase tracking-[0.08em]">
          <Link to="/settings/general">Open settings</Link>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
