import { useEffect, useMemo, useState } from 'react';
import { api } from '../../api/client';
import type { GlobalSyncStatus, MailingList } from '../../types';
import { Section } from '../ui/section';
import { CompactButton } from '../ui/compact-button';
import { Input } from '../ui/input';
import { cn } from '@/lib/utils';

const ITEMS_PER_PAGE = 20;

export function SyncPanel() {
  const [lists, setLists] = useState<MailingList[]>([]);
  const [syncStatus, setSyncStatus] = useState<GlobalSyncStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [currentPage, setCurrentPage] = useState(1);
  const [showEnabledOnly, setShowEnabledOnly] = useState(false);

  useEffect(() => {
    void loadMailingLists();
  }, []);

  useEffect(() => {
    const pollStatus = async () => {
      try {
        const status = await api.admin.sync.getStatus();
        setSyncStatus(status);
      } catch (err) {
        console.error('Failed to fetch sync status:', err);
      }
    };

    void pollStatus();
    const isActive = syncStatus?.isRunning;
    const interval = setInterval(pollStatus, isActive ? 1000 : 5000);
    return () => clearInterval(interval);
  }, [syncStatus?.isRunning]);

  useEffect(() => {
    setCurrentPage(1);
  }, [searchQuery, showEnabledOnly]);

  const loadMailingLists = async () => {
    try {
      const data = await api.mailingLists.list();
      setLists(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load mailing lists');
    }
  };

  const handleToggle = async (slug: string, currentEnabled: boolean) => {
    try {
      await api.mailingLists.toggle(slug, !currentEnabled);
      await loadMailingLists();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to toggle mailing list');
    }
  };

  const handleStartSync = async () => {
    const enabledSlugs = lists.filter((l) => l.enabled).map((l) => l.slug);
    if (enabledSlugs.length === 0) {
      setError('No mailing lists are enabled for sync');
      return;
    }

    setLoading(true);
    setError(null);
    try {
      await api.admin.sync.queue(enabledSlugs);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start sync');
    } finally {
      setLoading(false);
    }
  };

  const handleCancelSync = async () => {
    setLoading(true);
    setError(null);
    try {
      await api.admin.sync.cancel();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to cancel queue');
    } finally {
      setLoading(false);
    }
  };

  const filteredLists = useMemo(() => {
    const query = searchQuery.trim().toLowerCase();
    return lists.filter((list) => {
      const matches =
        query.length === 0 ||
        list.name.toLowerCase().includes(query) ||
        list.slug.toLowerCase().includes(query) ||
        (list.description?.toLowerCase().includes(query) ?? false);
      const enabledMatch = !showEnabledOnly || list.enabled;
      return matches && enabledMatch;
    });
  }, [lists, searchQuery, showEnabledOnly]);

  const totalPages = Math.max(1, Math.ceil(filteredLists.length / ITEMS_PER_PAGE));
  const paginatedLists = filteredLists.slice(
    (currentPage - 1) * ITEMS_PER_PAGE,
    currentPage * ITEMS_PER_PAGE,
  );

  const isRunning = syncStatus?.isRunning ?? false;
  const hasQueue = (syncStatus?.queuedJobs.length ?? 0) > 0;
  const currentJob = syncStatus?.currentJob ?? null;

  return (
    <div className="space-y-6">
      <Section
        title="Sync jobs"
        description="Queue updates for enabled mailing lists."
        actions={
          <div className="flex gap-2">
            <CompactButton onClick={handleStartSync} disabled={loading || isRunning}>
              {isRunning ? 'Syncing…' : 'Sync now'}
            </CompactButton>
            {(isRunning || hasQueue) && (
              <CompactButton
                onClick={handleCancelSync}
                disabled={loading}
                className="border-destructive/60 text-destructive hover:border-destructive hover:text-destructive"
              >
                Cancel queue
              </CompactButton>
            )}
          </div>
        }
      >
        {error && (
          <p className="text-[12px] uppercase tracking-[0.08em] text-destructive">
            {error}
          </p>
        )}

        {currentJob && (
          <div className="surface-muted px-3 py-3 text-sm">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div>
                <div className="font-semibold">{currentJob.name}</div>
                <div className="text-xs uppercase tracking-[0.08em] text-muted-foreground">
                  {currentJob.slug}
                </div>
              </div>
              <span className={cn(
                "rounded-sm border px-2 py-1 text-[11px] uppercase tracking-[0.08em]",
                statusTone(currentJob.phase)
              )}>
                {phaseLabel(currentJob.phase)}
              </span>
            </div>
            {currentJob.started_at && (
              <div className="mt-2 text-[11px] text-muted-foreground">
                Started {new Date(currentJob.started_at).toLocaleString()}
              </div>
            )}
          </div>
        )}

        {syncStatus && syncStatus.queuedJobs.length > 0 && (
          <div className="surface-muted px-3 py-2 text-[12px] text-muted-foreground uppercase tracking-[0.08em]">
            {syncStatus.queuedJobs.length} job
            {syncStatus.queuedJobs.length > 1 ? 's' : ''} in queue
          </div>
        )}

        {!isRunning && !hasQueue && (
          <p className="text-[12px] text-muted-foreground uppercase tracking-[0.08em]">
            No sync jobs running. Enable lists below and start a sync.
          </p>
        )}
      </Section>

      <Section
        title="Mailing lists"
        description="Enable the lists you want to include in sync jobs."
      >
        <div className="flex flex-wrap gap-2">
          <Input
            type="text"
            value={searchQuery}
            onChange={(event) => setSearchQuery(event.target.value)}
            placeholder="Filter by name, slug, or description"
            className="h-8 flex-1 min-w-[180px] text-sm"
          />
          <label className="inline-flex items-center gap-2 text-[12px] uppercase tracking-[0.08em] text-muted-foreground cursor-pointer">
            <input
              type="checkbox"
              checked={showEnabledOnly}
              onChange={(event) => setShowEnabledOnly(event.target.checked)}
              className="h-4 w-4 rounded border border-border/70 bg-background"
            />
            Enabled only
          </label>
        </div>

        <div className="surface-muted px-3 py-2 text-[12px] text-muted-foreground">
          Grokmirror mirrors all lists. The enabled toggle controls which lists the API parses.
        </div>

        <div className="border border-border/60 rounded-sm divide-y divide-border/50 overflow-hidden">
          {paginatedLists.length === 0 ? (
            <div className="px-3 py-6 text-center text-[12px] text-muted-foreground uppercase tracking-[0.08em]">
              {searchQuery || showEnabledOnly
                ? 'No mailing lists match your filters'
                : 'No mailing lists available. Seed lists from the database panel.'}
            </div>
          ) : (
            paginatedLists.map((list) => (
              <div
                key={list.id}
                className="flex flex-wrap items-center gap-3 px-3 py-2 text-sm"
              >
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={list.enabled}
                    onChange={() => handleToggle(list.slug, list.enabled)}
                    className="h-4 w-4 rounded border border-border/70 bg-background"
                    disabled={isRunning}
                  />
                  <span className="font-medium">{list.name}</span>
                </label>
                <span className="text-xs uppercase tracking-[0.08em] text-muted-foreground">
                  ({list.slug})
                </span>
                {list.description && (
                  <span className="text-xs text-muted-foreground flex-1 min-w-[120px] truncate">
                    {list.description}
                  </span>
                )}
                {list.last_synced_at && (
                  <span className="ml-auto text-xs text-muted-foreground whitespace-nowrap">
                    Last synced {new Date(list.last_synced_at).toLocaleString()}
                  </span>
                )}
              </div>
            ))
          )}
        </div>

        {totalPages > 1 && (
          <div className="flex items-center justify-between text-[12px] uppercase tracking-[0.08em] text-muted-foreground">
            <span>
              Showing {Math.min((currentPage - 1) * ITEMS_PER_PAGE + 1, filteredLists.length)}-
              {Math.min(currentPage * ITEMS_PER_PAGE, filteredLists.length)} of {filteredLists.length}
            </span>
            <div className="flex items-center gap-2">
              <CompactButton
                onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
                disabled={currentPage === 1}
              >
                Prev
              </CompactButton>
              <span>Page {currentPage} / {totalPages}</span>
              <CompactButton
                onClick={() => setCurrentPage((p) => Math.min(totalPages, p + 1))}
                disabled={currentPage === totalPages}
              >
                Next
              </CompactButton>
            </div>
          </div>
        )}
      </Section>
    </div>
  );
}

function phaseLabel(phase: string) {
  const map: Record<string, string> = {
    waiting: 'Waiting',
    parsing: 'Parsing',
    threading: 'Threading',
    done: 'Done',
    errored: 'Error',
  };
  return map[phase] ?? 'Waiting';
}

function statusTone(phase: string) {
  switch (phase) {
    case 'errored':
      return 'border-destructive/60 text-destructive';
    case 'done':
      return 'border-green-500/60 text-green-600';
    default:
      return 'border-border/60 text-muted-foreground';
  }
}
