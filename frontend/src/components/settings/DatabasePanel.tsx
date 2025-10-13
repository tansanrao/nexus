import { useState, useEffect } from 'react';
import { api } from '../../api/client';
import type { DatabaseStatus, GlobalSyncStatus } from '../../types';
import { Button } from '../ui/button';
import { Card } from '../ui/card';
import { Input } from '../ui/input';

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
    loadStatus();
  }, []);

  // Poll for sync status to auto-refresh database stats when sync completes
  useEffect(() => {
    const checkSyncStatus = async () => {
      try {
        const newSyncStatus = await api.admin.sync.getStatus();

        // If sync just completed, refresh database stats
        if (syncStatus?.is_running && !newSyncStatus.is_running) {
          await loadStatus();
        }

        setSyncStatus(newSyncStatus);
      } catch (err) {
        // Silently fail - this is just for auto-refresh
        console.error('Failed to check sync status:', err);
      }
    };

    // Check every 3 seconds
    const interval = setInterval(checkSyncStatus, 3000);
    return () => clearInterval(interval);
  }, [syncStatus?.is_running]);

  const loadStatus = async () => {
    try {
      const data = await api.admin.database.getStatus();
      setStatus(data);
    } catch (err) {
      console.error('Failed to fetch database status:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch database status');
    }
  };

  const handleResetDatabase = async () => {
    if (confirmText !== 'RESET') {
      return;
    }

    setLoading(true);
    setError(null);
    try {
      await api.admin.database.reset();
      setShowConfirmModal(false);
      setConfirmText('');
      // Reload status after reset
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
        `Successfully seeded ${result.mailing_lists_created} mailing lists, ` +
        `${result.repositories_created} repositories, and created ${result.partitions_created} partitions.`
      );
      // Clear success message after 5 seconds
      setTimeout(() => setSeedSuccess(null), 5000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to seed mailing lists');
    } finally {
      setSeedLoading(false);
    }
  };

  const formatNumber = (num: number): string => {
    return num.toLocaleString();
  };

  const formatDate = (dateStr: string | null): string => {
    if (!dateStr) return 'N/A';
    return new Date(dateStr).toLocaleDateString();
  };

  return (
    <Card className="p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-semibold">Database Management</h2>
        <div className="flex gap-3">
          <Button
            onClick={handleSeedMailingLists}
            disabled={seedLoading}
          >
            {seedLoading ? 'Seeding...' : 'Seed Mailing Lists'}
          </Button>
          <Button
            onClick={() => setShowConfirmModal(true)}
            variant="destructive"
          >
            Reset Database
          </Button>
        </div>
      </div>

      {error && (
        <Card className="mb-4 p-4 bg-destructive/10 border-destructive">
          <div className="text-sm text-destructive">{error}</div>
        </Card>
      )}

      {seedSuccess && (
        <Card className="mb-4 p-4 bg-primary/10 border-primary/30">
          <div className="text-sm text-primary">{seedSuccess}</div>
        </Card>
      )}

      <Card className="mb-4 p-4 bg-primary/5 border-primary/20 text-sm">
        <p className="font-medium mb-1">Setup Instructions:</p>
        <ol className="list-decimal list-inside space-y-1">
          <li>Ensure grokmirror is running and has synced repositories (see grokmirror/README.md)</li>
          <li>Click "Reset Database" to create fresh schema</li>
          <li>Click "Seed Mailing Lists" to populate all ~341 lore.kernel.org lists</li>
          <li>Go to Sync panel to enable and sync specific lists</li>
        </ol>
      </Card>

      {status && (
        <div className="space-y-6">
          {/* Statistics Grid */}
          <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
            <StatCard label="Authors" value={formatNumber(status.total_authors)} />
            <StatCard label="Emails" value={formatNumber(status.total_emails)} />
            <StatCard label="Threads" value={formatNumber(status.total_threads)} />
            <StatCard label="Recipients" value={formatNumber(status.total_recipients)} />
            <StatCard label="References" value={formatNumber(status.total_references)} />
            <StatCard label="Thread Memberships" value={formatNumber(status.total_thread_memberships)} />
          </div>

          {/* Date Range */}
          {status.date_range_start && status.date_range_end && (
            <div className="pt-4 border-t">
              <h3 className="text-sm font-medium mb-2">Data Range</h3>
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <span>{formatDate(status.date_range_start)}</span>
                <span>→</span>
                <span>{formatDate(status.date_range_end)}</span>
              </div>
            </div>
          )}

          {/* Refresh Button */}
          <div className="pt-4 border-t">
            <Button
              onClick={loadStatus}
              variant="outline"
              size="sm"
            >
              Refresh Statistics
            </Button>
          </div>
        </div>
      )}

      {/* Confirmation Modal */}
      {showConfirmModal && (
        <div className="fixed inset-0 bg-black/95 backdrop-blur-md flex items-center justify-center z-50">
          <Card className="p-6 max-w-md w-full mx-4 shadow-2xl" style={{ backgroundColor: 'hsl(var(--card))' }}>
            <h3 className="text-xl font-semibold mb-4">Reset Database</h3>
            <div className="space-y-4">
              <div className="p-4 rounded-lg border-2 border-destructive" style={{ backgroundColor: 'hsl(var(--card))' }}>
                <p className="text-sm text-destructive font-medium mb-2">Warning: This action cannot be undone!</p>
                <p className="text-sm text-destructive">
                  This will delete all data from the database and recreate all tables. You will need to run a sync to
                  repopulate the data.
                </p>
              </div>

              <div>
                <label className="block text-sm font-medium mb-2">
                  Type <span className="font-mono font-bold">RESET</span> to confirm:
                </label>
                <Input
                  type="text"
                  value={confirmText}
                  onChange={(e) => setConfirmText(e.target.value)}
                  placeholder="RESET"
                />
              </div>

              <div className="flex gap-3 pt-2">
                <Button
                  onClick={handleResetDatabase}
                  disabled={confirmText !== 'RESET' || loading}
                  variant="destructive"
                  className="flex-1"
                >
                  {loading ? 'Resetting...' : 'Reset Database'}
                </Button>
                <Button
                  onClick={() => {
                    setShowConfirmModal(false);
                    setConfirmText('');
                  }}
                  disabled={loading}
                  variant="outline"
                  className="flex-1"
                >
                  Cancel
                </Button>
              </div>
            </div>
          </Card>
        </div>
      )}
    </Card>
  );
}

function StatCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="p-4 bg-muted rounded-lg border">
      <p className="text-sm text-muted-foreground mb-1">{label}</p>
      <p className="text-2xl font-semibold">{value}</p>
    </div>
  );
}
