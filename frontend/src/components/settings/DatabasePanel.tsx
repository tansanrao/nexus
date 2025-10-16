import { useEffect, useState } from 'react';
import { api } from '../../api/client';
import type { DatabaseStatus, GlobalSyncStatus } from '../../types';
import { Section } from '../ui/section';
import { CompactButton } from '../ui/compact-button';
import { Input } from '../ui/input';
import { cn } from '@/lib/utils';

export function DatabasePanel() {
  const [status, setStatus] = useState<DatabaseStatus | null>(null);
  const [syncStatus, setSyncStatus] = useState<GlobalSyncStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [seedLoading, setSeedLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [seedSuccess, setSeedSuccess] = useState<string | null>(null);
  const [showConfirmModal, setShowConfirmModal] = useState(false);
  const [confirmText, setConfirmText] = useState('');

  useEffect(() => {
    void loadStatus();
  }, []);

  useEffect(() => {
    const pollSyncStatus = async () => {
      try {
        const newStatus = await api.admin.sync.getStatus();
        if (syncStatus?.isRunning && !newStatus.isRunning) {
          await loadStatus();
        }
        setSyncStatus(newStatus);
      } catch (err) {
        console.error('Failed to poll sync status:', err);
      }
    };

    const interval = setInterval(pollSyncStatus, 3000);
    return () => clearInterval(interval);
  }, [syncStatus?.isRunning]);

  const loadStatus = async () => {
    try {
      const data = await api.admin.database.getStatus();
      setStatus(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch database status');
    }
  };

  const handleResetDatabase = async () => {
    if (confirmText !== 'RESET') return;
    setLoading(true);
    setError(null);
    try {
      await api.admin.database.reset();
      setShowConfirmModal(false);
      setConfirmText('');
      await loadStatus();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to reset database');
    } finally {
      setLoading(false);
    }
  };

  const handleSeedMailingLists = async () => {
    setSeedLoading(true);
    setError(null);
    setSeedSuccess(null);
    try {
      const result = await api.mailingLists.seed();
      setSeedSuccess(
        `Seeded ${result.mailingListsCreated} lists, ${result.repositoriesCreated} repos, ${result.partitionsCreated} partitions.`,
      );
      setTimeout(() => setSeedSuccess(null), 5000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to seed mailing lists');
    } finally {
      setSeedLoading(false);
    }
  };

  const formatNumber = (num: number) => num.toLocaleString();
  const formatDate = (date: string | null) =>
    date ? new Date(date).toLocaleDateString() : 'N/A';

  return (
    <>
      <div className="space-y-6">
        <Section
          title="Database maintenance"
          description="Seed initial data or reset the schema."
          actions={
            <div className="flex gap-2">
              <CompactButton onClick={handleSeedMailingLists} disabled={seedLoading}>
                {seedLoading ? 'Seeding…' : 'Seed lists'}
              </CompactButton>
              <CompactButton
                onClick={() => setShowConfirmModal(true)}
                className="border-destructive/60 text-destructive hover:border-destructive hover:text-destructive"
              >
                Reset DB
              </CompactButton>
            </div>
          }
        >
          {error && (
            <p className="text-[12px] uppercase tracking-[0.08em] text-destructive">
              {error}
            </p>
          )}
          {seedSuccess && (
            <p className="text-[12px] uppercase tracking-[0.08em] text-green-600">
              {seedSuccess}
            </p>
          )}
          <ol className="list-decimal list-inside text-sm text-muted-foreground space-y-1">
            <li>Ensure grokmirror has synced repositories.</li>
            <li>Reset the database to create fresh schema.</li>
            <li>Seed mailing lists to populate lore.kernel.org data.</li>
            <li>Enable and sync lists from the panel above.</li>
          </ol>
        </Section>

        {status && (
          <Section
            title="Statistics"
            description="Current totals across the indexed dataset."
            actions={
              <CompactButton onClick={() => void loadStatus()} disabled={loading}>
                {loading ? 'Refreshing…' : 'Refresh'}
              </CompactButton>
            }
          >
            <div className="grid grid-cols-2 md:grid-cols-3 gap-3 text-sm">
              <Stat label="Authors" value={formatNumber(status.totalAuthors)} />
              <Stat label="Emails" value={formatNumber(status.totalEmails)} />
              <Stat label="Threads" value={formatNumber(status.totalThreads)} />
              <Stat label="Recipients" value={formatNumber(status.totalRecipients)} />
              <Stat label="References" value={formatNumber(status.totalReferences)} />
              <Stat
                label="Thread memberships"
                value={formatNumber(status.totalThreadMemberships)}
              />
            </div>

            {status.dateRangeStart && status.dateRangeEnd && (
              <div className="surface-muted px-3 py-3 text-[12px] text-muted-foreground">
                <div className="flex flex-wrap items-center gap-3">
                  <span className="uppercase tracking-[0.08em]">Range</span>
                  <span>{formatDate(status.dateRangeStart)}</span>
                  <span aria-hidden="true">→</span>
                  <span>{formatDate(status.dateRangeEnd)}</span>
                </div>
              </div>
            )}
          </Section>
        )}
      </div>

      {showConfirmModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4">
          <div className="surface max-w-md w-full space-y-4 p-5">
            <h3 className="text-lg font-semibold">Reset database</h3>
            <div className="surface-muted border-destructive/60 px-3 py-3 text-sm text-destructive">
              This permanently deletes all imported data. You will need to run sync jobs to repopulate.
            </div>

            <label className="text-sm font-medium">
              Type <span className="font-mono text-destructive">RESET</span> to confirm
            </label>
            <Input
              type="text"
              value={confirmText}
              onChange={(event) => setConfirmText(event.target.value)}
              placeholder="RESET"
            />

            <div className="flex gap-2">
              <CompactButton
                onClick={handleResetDatabase}
                disabled={confirmText !== 'RESET' || loading}
                className={cn(
                  "flex-1 border-destructive/60 text-destructive hover:border-destructive hover:text-destructive",
                  loading && "cursor-progress"
                )}
              >
                {loading ? 'Resetting…' : 'Confirm reset'}
              </CompactButton>
              <CompactButton
                onClick={() => {
                  setShowConfirmModal(false);
                  setConfirmText('');
                }}
                disabled={loading}
                className="flex-1"
              >
                Cancel
              </CompactButton>
            </div>
          </div>
        </div>
      )}
    </>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="surface-muted px-3 py-3 rounded-sm text-sm">
      <div className="text-xs uppercase tracking-[0.08em] text-muted-foreground">{label}</div>
      <div className="text-lg font-semibold text-foreground">{value}</div>
    </div>
  );
}
