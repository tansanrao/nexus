import { useState, useEffect } from 'react';
import { api } from '../../api/client';
import type { DatabaseStatus, GlobalSyncStatus } from '../../types';

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
    <div className="bg-white rounded-lg border border-gray-200 p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-semibold text-gray-900">Database Management</h2>
        <div className="flex gap-3">
          <button
            onClick={handleSeedMailingLists}
            disabled={seedLoading}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
          >
            {seedLoading ? 'Seeding...' : 'Seed Mailing Lists'}
          </button>
          <button
            onClick={() => setShowConfirmModal(true)}
            className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
          >
            Reset Database
          </button>
        </div>
      </div>

      {error && (
        <div className="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg text-red-800">
          {error}
        </div>
      )}

      {seedSuccess && (
        <div className="mb-4 p-4 bg-green-50 border border-green-200 rounded-lg text-green-800">
          {seedSuccess}
        </div>
      )}

      <div className="mb-4 p-4 bg-blue-50 border border-blue-200 rounded-lg text-blue-800 text-sm">
        <p className="font-medium mb-1">Setup Instructions:</p>
        <ol className="list-decimal list-inside space-y-1">
          <li>Ensure grokmirror is running and has synced repositories (see GROKMIRROR_SETUP.md)</li>
          <li>Click "Reset Database" to create fresh schema</li>
          <li>Click "Seed Mailing Lists" to populate all ~341 lore.kernel.org lists</li>
          <li>Go to Sync panel to enable and sync specific lists</li>
        </ol>
      </div>

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
            <div className="pt-4 border-t border-gray-200">
              <h3 className="text-sm font-medium text-gray-700 mb-2">Data Range</h3>
              <div className="flex items-center gap-2 text-sm text-gray-600">
                <span>{formatDate(status.date_range_start)}</span>
                <span>→</span>
                <span>{formatDate(status.date_range_end)}</span>
              </div>
            </div>
          )}

          {/* Refresh Button */}
          <div className="pt-4 border-t border-gray-200">
            <button
              onClick={loadStatus}
              className="px-4 py-2 text-sm bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors"
            >
              Refresh Statistics
            </button>
          </div>
        </div>
      )}

      {/* Confirmation Modal */}
      {showConfirmModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 max-w-md w-full mx-4">
            <h3 className="text-xl font-semibold text-gray-900 mb-4">Reset Database</h3>
            <div className="space-y-4">
              <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
                <p className="text-sm text-red-800 font-medium mb-2">Warning: This action cannot be undone!</p>
                <p className="text-sm text-red-700">
                  This will delete all data from the database and recreate all tables. You will need to run a sync to
                  repopulate the data.
                </p>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Type <span className="font-mono font-bold">RESET</span> to confirm:
                </label>
                <input
                  type="text"
                  value={confirmText}
                  onChange={(e) => setConfirmText(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-red-500 focus:border-transparent"
                  placeholder="RESET"
                />
              </div>

              <div className="flex gap-3 pt-2">
                <button
                  onClick={handleResetDatabase}
                  disabled={confirmText !== 'RESET' || loading}
                  className="flex-1 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors"
                >
                  {loading ? 'Resetting...' : 'Reset Database'}
                </button>
                <button
                  onClick={() => {
                    setShowConfirmModal(false);
                    setConfirmText('');
                  }}
                  disabled={loading}
                  className="flex-1 px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors"
                >
                  Cancel
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function StatCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="p-4 bg-gray-50 rounded-lg border border-gray-200">
      <p className="text-sm text-gray-600 mb-1">{label}</p>
      <p className="text-2xl font-semibold text-gray-900">{value}</p>
    </div>
  );
}
