import { useEffect, useMemo, useState } from 'react';
import { apiClient } from '../../lib/api';
import type { JobType } from '../../types';
import { Section } from '../ui/section';
import { CompactButton } from '../ui/compact-button';
import { Input } from '../ui/input';
import { Badge } from '../ui/badge';
import { cn } from '../../lib/utils';

const STORAGE_KEY = 'nexus::search-index-history';
const MAX_HISTORY_ENTRIES = 8;

type MaintenanceOperation = 'refresh_index' | 'reset_index';

interface HistoryEntry {
  id: string;
  timestamp: string;
  scope: string;
  operation: MaintenanceOperation;
  jobId: number | null;
  jobType?: JobType | string;
  success: boolean;
  message: string;
  durationMs?: number;
}

interface ActiveRun {
  scope: string;
  operation: MaintenanceOperation;
  startedAt: number;
}

const loadHistory = (): HistoryEntry[] => {
  if (typeof window === 'undefined') return [];
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as Partial<HistoryEntry>[];
    if (Array.isArray(parsed)) {
      return parsed
        .filter((entry) => typeof entry?.id === 'string' && typeof entry?.timestamp === 'string')
        .map((entry) => ({
          id: entry.id as string,
          timestamp: entry.timestamp as string,
          scope: entry.scope ?? 'All mailing lists',
          operation: entry.operation ?? 'refresh_index',
          jobId: entry.jobId ?? null,
          jobType: entry.jobType,
          success: entry.success ?? false,
          message: entry.message ?? '',
          durationMs: entry.durationMs,
        }))
        .slice(0, MAX_HISTORY_ENTRIES);
    }
    return [];
  } catch {
    return [];
  }
};

const persistHistory = (entries: HistoryEntry[]) => {
  if (typeof window === 'undefined') return;
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(entries.slice(0, MAX_HISTORY_ENTRIES)));
  } catch (err) {
    console.error('Failed to persist search history:', err);
  }
};

const formatDuration = (durationMs?: number) => {
  if (!durationMs || Number.isNaN(durationMs)) return null;
  if (durationMs < 1000) {
    return `${Math.round(durationMs)} ms`;
  }
  const seconds = durationMs / 1000;
  if (seconds < 60) {
    return `${seconds.toFixed(1)} s`;
  }
  const minutes = Math.floor(seconds / 60);
  const remainder = (seconds % 60).toFixed(0);
  return `${minutes}m ${remainder}s`;
};

export function SearchMaintenancePanel() {
  const [mailingListSlug, setMailingListSlug] = useState('');
  const [isRunning, setIsRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [history, setHistory] = useState<HistoryEntry[]>(() => loadHistory());
  const [activeRun, setActiveRun] = useState<ActiveRun | null>(null);
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    const interval = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(interval);
  }, []);

  useEffect(() => {
    persistHistory(history);
  }, [history]);

  const scopeLabel = useMemo(() => {
    const trimmed = mailingListSlug.trim();
    if (trimmed.length === 0) return 'all mailing lists';
    return `mailing list ${trimmed}`;
  }, [mailingListSlug]);

  const handleClearHistory = () => {
    setHistory([]);
    persistHistory([]);
  };

  const handleResetIndexes = async () => {
    const trimmedSlug = mailingListSlug.trim();
    const scope = trimmedSlug || 'All mailing lists';
    if (
      !window.confirm(
        `This will drop and recreate search indexes for ${scope}. Continue?`
      )
    ) {
      return;
    }

    setIsRunning(true);
    setError(null);
    const startedAt = performance.now();
    setActiveRun({
      scope,
      operation: 'reset_index',
      startedAt: Date.now(),
    });

    try {
      const params = trimmedSlug.length > 0 ? { mailingListSlug: trimmedSlug } : {};
      const response = await apiClient.resetSearchIndexes(params);
      const duration = performance.now() - startedAt;
      const entry: HistoryEntry = {
        id: cryptoRandomId(),
        timestamp: new Date().toISOString(),
        scope,
        operation: 'reset_index',
        jobId: response.jobId ?? null,
        jobType: response.jobType,
        success: true,
        message: response.message,
        durationMs: duration,
      };
      setHistory((prev) => [entry, ...prev].slice(0, MAX_HISTORY_ENTRIES));
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to reset search indexes';
      setError(message);
      const duration = performance.now() - startedAt;
      const entry: HistoryEntry = {
        id: cryptoRandomId(),
        timestamp: new Date().toISOString(),
        scope,
        operation: 'reset_index',
        jobId: null,
        jobType: undefined,
        success: false,
        message,
        durationMs: duration,
      };
      setHistory((prev) => [entry, ...prev].slice(0, MAX_HISTORY_ENTRIES));
    } finally {
      setIsRunning(false);
      setActiveRun(null);
    }
  };

  const handleRefresh = async () => {
    const trimmedSlug = mailingListSlug.trim();
    const payload = trimmedSlug.length > 0 ? { mailingListSlug: trimmedSlug } : {};

    setIsRunning(true);
    setError(null);
    const startedAt = performance.now();
    setActiveRun({
      scope: trimmedSlug || 'All mailing lists',
      operation: 'refresh_index',
      startedAt: Date.now(),
    });

    try {
      const response = await apiClient.refreshSearchIndex(payload);
      const duration = performance.now() - startedAt;
      const entry: HistoryEntry = {
        id: cryptoRandomId(),
        timestamp: new Date().toISOString(),
        scope: trimmedSlug || 'All mailing lists',
        operation: 'refresh_index',
        jobId: response.jobId ?? null,
        jobType: response.jobType,
        success: true,
        message: response.message,
        durationMs: duration,
      };
      setHistory((prev) => [entry, ...prev].slice(0, MAX_HISTORY_ENTRIES));
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to refresh search index';
      setError(message);
      const duration = performance.now() - startedAt;
      const entry: HistoryEntry = {
        id: cryptoRandomId(),
        timestamp: new Date().toISOString(),
        scope: trimmedSlug || 'All mailing lists',
        operation: 'refresh_index',
        jobId: null,
        jobType: undefined,
        success: false,
        message,
        durationMs: duration,
      };
      setHistory((prev) => [entry, ...prev].slice(0, MAX_HISTORY_ENTRIES));
    } finally {
      setIsRunning(false);
      setActiveRun(null);
    }
  };

  const elapsedForActiveRun = useMemo(() => {
    if (!activeRun) return null;
    const elapsedMs = now - activeRun.startedAt;
    if (elapsedMs <= 0) return null;
    const seconds = (elapsedMs / 1000).toFixed(1);
    return `${seconds}s`;
  }, [activeRun, now]);

  const refreshBusy = isRunning && activeRun?.operation === 'refresh_index';
  const resetIndexBusy = isRunning && activeRun?.operation === 'reset_index';

  return (
    <Section
      title="Search maintenance"
      description="Recompute lexical vectors or rebuild indexes after imports."
      actions={
        <div className="flex flex-wrap gap-2">
          <CompactButton onClick={handleRefresh} disabled={isRunning}>
            {refreshBusy ? 'Refreshing…' : 'Refresh index'}
          </CompactButton>
          <CompactButton onClick={handleResetIndexes} disabled={isRunning}>
            {resetIndexBusy ? 'Resetting…' : 'Reset indexes'}
          </CompactButton>
          {history.length > 0 && (
            <CompactButton onClick={handleClearHistory}>
              Clear history
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

      <div className="space-y-3">
        <div className="flex flex-wrap items-center gap-3">
          <Input
            value={mailingListSlug}
            onChange={(event) => setMailingListSlug(event.target.value)}
            placeholder="Mailing list slug (leave blank for all)"
            className="h-8 w-full sm:w-auto min-w-[220px] text-sm"
          />
        </div>
        <p className="text-[12px] text-muted-foreground uppercase tracking-[0.08em]">
          Refresh operations update lexical and semantic materialized fields for {scopeLabel}. Reset fully drops and rebuilds indexes.
        </p>
      </div>

      {activeRun && (
        <div className="surface-muted border border-border/40 px-3 py-3 text-sm flex flex-col gap-1">
          <div className="flex items-center gap-2">
            <Badge variant="secondary">Running</Badge>
            <span className="font-semibold">{activeRun.scope}</span>
          </div>
          <div className="text-[12px] uppercase tracking-[0.08em] text-muted-foreground">
            {operationLabel(activeRun.operation)}
            {elapsedForActiveRun ? ` • ${elapsedForActiveRun}` : null}
          </div>
        </div>
      )}

      <div className="space-y-2">
        <div className="text-xs uppercase tracking-[0.08em] text-muted-foreground">
          Recent runs
        </div>
        {history.length === 0 ? (
          <div className="surface-muted px-3 py-3 text-[12px] uppercase tracking-[0.08em] text-muted-foreground">
            No maintenance jobs recorded yet.
          </div>
        ) : (
          <div className="space-y-2">
            {history.map((entry) => (
              <div
                key={entry.id}
                className={cn(
                  'surface-muted px-3 py-3 text-sm border border-transparent',
                  entry.success ? 'border-border/40' : 'border-destructive/40'
                )}
              >
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <div className="flex items-center gap-2">
                    <Badge variant={entry.success ? 'secondary' : 'destructive'}>
                      {entry.success ? 'Success' : 'Failed'}
                    </Badge>
                    <span className="font-semibold">{entry.scope}</span>
                  </div>
                  <span className="text-[11px] uppercase tracking-[0.08em] text-muted-foreground">
                    {new Date(entry.timestamp).toLocaleString()}
                  </span>
                </div>
                <div className="mt-1 text-[12px] text-muted-foreground uppercase tracking-[0.08em]">
                  {operationLabel(entry.operation)}
                  {entry.jobId != null ? ` • Job #${entry.jobId}` : null}
                  {entry.jobType ? ` • ${jobTypeLabel(entry.jobType)}` : null}
                  {entry.durationMs
                    ? ` • ${formatDuration(entry.durationMs)}`
                    : null}
                </div>
                <div className="mt-2 text-sm text-foreground">{entry.message}</div>
              </div>
            ))}
          </div>
        )}
      </div>
    </Section>
  );
}

function cryptoRandomId(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  return `search-run-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function operationLabel(operation: MaintenanceOperation): string {
  switch (operation) {
    case 'refresh_index':
      return 'Refresh index';
    case 'reset_index':
      return 'Reset indexes';
    default:
      return operation;
  }
}

function jobTypeLabel(jobType?: JobType | string): string {
  if (!jobType) return 'Job';
  const normalized = String(jobType);
  switch (normalized) {
    case 'import':
      return 'Sync job';
    case 'index_maintenance':
      return 'Index maintenance';
    default:
      return normalized.replace(/_/g, ' ');
  }
}
