import { useState, useEffect } from 'react';
import { api } from '../../api/client';
import type { GlobalSyncStatus, MailingList } from '../../types';
import { Button } from '../ui/button';
import { Card } from '../ui/card';
import { Badge } from '../ui/badge';

export function SyncPanel() {
  const [lists, setLists] = useState<MailingList[]>([]);
  const [syncStatus, setSyncStatus] = useState<GlobalSyncStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [currentPage, setCurrentPage] = useState(1);
  const [showEnabledOnly, setShowEnabledOnly] = useState(false);
  const itemsPerPage = 20;

  // Load mailing lists on mount
  useEffect(() => {
    loadMailingLists();
  }, []);

  // Poll for sync status
  useEffect(() => {
    const pollStatus = async () => {
      try {
        const status = await api.admin.sync.getStatus();
        setSyncStatus(status);
      } catch (err) {
        console.error('Failed to fetch sync status:', err);
      }
    };

    pollStatus();

    const isActive = syncStatus?.is_running;
    const pollInterval = isActive ? 1000 : 5000; // 1s when active, 5s when idle
    const interval = setInterval(pollStatus, pollInterval);

    return () => clearInterval(interval);
  }, [syncStatus?.is_running]);

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
      // Reload lists to reflect the change
      await loadMailingLists();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to toggle mailing list');
    }
  };

  const handleStartSync = async () => {
    const enabledSlugs = lists.filter(l => l.enabled).map(l => l.slug);

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
      setError(err instanceof Error ? err.message : 'Failed to cancel sync');
    } finally {
      setLoading(false);
    }
  };

  const currentJob = syncStatus?.current_job;
  const isRunning = syncStatus?.is_running || false;
  const hasQueue = (syncStatus?.queued_jobs.length || 0) > 0;

  // Filter and paginate lists
  const filteredLists = lists.filter(list => {
    const matchesSearch = searchQuery === '' ||
      list.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      list.slug.toLowerCase().includes(searchQuery.toLowerCase()) ||
      (list.description?.toLowerCase().includes(searchQuery.toLowerCase()) ?? false);

    const matchesEnabledFilter = !showEnabledOnly || list.enabled;

    return matchesSearch && matchesEnabledFilter;
  });

  const totalPages = Math.ceil(filteredLists.length / itemsPerPage);
  const paginatedLists = filteredLists.slice(
    (currentPage - 1) * itemsPerPage,
    currentPage * itemsPerPage
  );

  // Reset to page 1 when filters change
  useEffect(() => {
    setCurrentPage(1);
  }, [searchQuery, showEnabledOnly]);

  return (
    <Card className="p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-semibold">Data Synchronization</h2>
        <div className="flex gap-3">
          <Button
            onClick={handleStartSync}
            disabled={loading || isRunning}
          >
            {isRunning ? 'Syncing...' : 'Sync Now'}
          </Button>
          {(isRunning || hasQueue) && (
            <Button
              onClick={handleCancelSync}
              disabled={loading}
              variant="destructive"
            >
              Cancel Queue
            </Button>
          )}
        </div>
      </div>

      {error && (
        <Card className="mb-4 p-4 bg-destructive/10 border-destructive">
          <div className="text-sm text-destructive">{error}</div>
        </Card>
      )}

      {/* Mailing Lists Selection */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-lg font-medium text-gray-900">Select Mailing Lists</h3>
          <p className="text-sm text-gray-600">
            {lists.filter(l => l.enabled).length} of {lists.length} enabled
          </p>
        </div>

        {/* Search and Filters */}
        <div className="flex gap-3 mb-4">
          <input
            type="text"
            placeholder="Search by name, slug, or description..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
          <label className="flex items-center gap-2 px-4 py-2 border border-gray-300 rounded-lg cursor-pointer hover:bg-gray-50">
            <input
              type="checkbox"
              checked={showEnabledOnly}
              onChange={(e) => setShowEnabledOnly(e.target.checked)}
              className="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
            />
            <span className="text-sm text-gray-700">Enabled only</span>
          </label>
        </div>

        {/* Info banner */}
        <div className="mb-3 p-3 bg-blue-50 border border-blue-200 rounded-lg text-sm text-blue-800">
          <p><strong>Note:</strong> Grokmirror mirrors ALL lists automatically. The "enabled" toggle only controls which lists the API server will parse and import.</p>
        </div>

        {/* Mailing lists */}
        <div className="space-y-2">
          {paginatedLists.length === 0 ? (
            <div className="text-center py-8 text-gray-500">
              {searchQuery || showEnabledOnly ? 'No mailing lists match your filters' : 'No mailing lists available. Click "Seed Mailing Lists" in the Database panel.'}
            </div>
          ) : (
            paginatedLists.map((list) => (
              <div
                key={list.id}
                className="flex items-center justify-between p-3 border border-gray-200 rounded-lg hover:bg-gray-50"
              >
                <div className="flex items-center gap-3 flex-1 min-w-0">
                  <label className="flex items-center cursor-pointer">
                    <input
                      type="checkbox"
                      checked={list.enabled}
                      onChange={() => handleToggle(list.slug, list.enabled)}
                      className="w-5 h-5 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                      disabled={isRunning}
                    />
                    <span className="ml-3 font-medium text-gray-900">{list.name}</span>
                  </label>
                  <span className="text-xs text-gray-400 font-mono">({list.slug})</span>
                  {list.description && (
                    <span className="text-sm text-gray-500 truncate">- {list.description}</span>
                  )}
                </div>
                {list.last_synced_at && (
                  <span className="text-xs text-gray-400 whitespace-nowrap ml-3">
                    Last synced: {new Date(list.last_synced_at).toLocaleString()}
                  </span>
                )}
              </div>
            ))
          )}
        </div>

        {/* Pagination */}
        {totalPages > 1 && (
          <div className="flex items-center justify-between mt-4 pt-4 border-t border-gray-200">
            <p className="text-sm text-gray-600">
              Showing {((currentPage - 1) * itemsPerPage) + 1} - {Math.min(currentPage * itemsPerPage, filteredLists.length)} of {filteredLists.length}
            </p>
            <div className="flex gap-2">
              <button
                onClick={() => setCurrentPage(p => Math.max(1, p - 1))}
                disabled={currentPage === 1}
                className="px-3 py-1 text-sm border border-gray-300 rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Previous
              </button>
              <span className="px-3 py-1 text-sm text-gray-700">
                Page {currentPage} of {totalPages}
              </span>
              <button
                onClick={() => setCurrentPage(p => Math.min(totalPages, p + 1))}
                disabled={currentPage === totalPages}
                className="px-3 py-1 text-sm border border-gray-300 rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Next
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Current Job Status */}
      {currentJob && (
        <div className="mb-6">
          <h3 className="text-lg font-medium text-gray-900 mb-3">Current Sync Job</h3>
          <div className="bg-blue-50 rounded-lg p-4 border border-blue-200">
            <div className="flex items-start justify-between mb-3">
              <div>
                <p className="font-medium text-blue-900">
                  Syncing: {currentJob.job_data.mailing_list_slug}
                </p>
                <p className="text-sm text-blue-700 mt-1">
                  {currentJob.job_data.progress.current_step}
                </p>
                {currentJob.job_data.progress.phase_details && (
                  <p className="text-xs text-blue-600 mt-1">
                    {currentJob.job_data.progress.phase_details}
                  </p>
                )}
              </div>
              <StatusBadge status={currentJob.status} />
            </div>

            {/* Progress Bar */}
            {currentJob.job_data.progress.total && currentJob.job_data.progress.total > 0 && (
              <div className="mt-3">
                <div className="flex justify-between text-xs text-blue-700 mb-1">
                  <span>Progress</span>
                  <span>
                    {currentJob.job_data.progress.processed.toLocaleString()} / {currentJob.job_data.progress.total.toLocaleString()}
                    {' '}({getProgressPercentage(currentJob.job_data.progress).toFixed(1)}%)
                  </span>
                </div>
                <div className="w-full bg-blue-200 rounded-full h-2 overflow-hidden">
                  <div
                    className="bg-blue-600 h-full transition-all duration-300 rounded-full"
                    style={{ width: `${getProgressPercentage(currentJob.job_data.progress)}%` }}
                  />
                </div>
              </div>
            )}

            {/* Metrics */}
            <div className="grid grid-cols-5 gap-2 mt-4 pt-3 border-t border-blue-200">
              <MetricItem label="Parsed" value={currentJob.job_data.metrics.emails_parsed} />
              <MetricItem label="Errors" value={currentJob.job_data.metrics.parse_errors} warning={currentJob.job_data.metrics.parse_errors > 0} />
              <MetricItem label="Authors" value={currentJob.job_data.metrics.authors_imported} />
              <MetricItem label="Emails" value={currentJob.job_data.metrics.emails_imported} />
              <MetricItem label="Threads" value={currentJob.job_data.metrics.threads_created} />
            </div>
          </div>
        </div>
      )}

      {/* Queue */}
      {syncStatus && syncStatus.queued_jobs.length > 0 && (
        <div>
          <h3 className="text-lg font-medium text-gray-900 mb-3">
            Queue ({syncStatus.queued_jobs.length} jobs)
          </h3>
          <div className="space-y-2">
            {syncStatus.queued_jobs.map((job) => (
              <div
                key={job.id}
                className="flex items-center justify-between p-3 bg-gray-50 border border-gray-200 rounded-lg"
              >
                <div className="flex items-center gap-3">
                  <span className="flex items-center justify-center w-6 h-6 bg-gray-300 text-gray-700 rounded-full text-xs font-bold">
                    {job.position}
                  </span>
                  <span className="font-medium text-gray-900">{job.mailing_list_name}</span>
                  <span className="text-sm text-gray-500">({job.mailing_list_slug})</span>
                </div>
                <span className="text-xs text-gray-400 px-2 py-1 bg-gray-200 rounded">Queued</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Idle state */}
      {!isRunning && !hasQueue && (
        <div className="text-center py-8 text-muted-foreground">
          <p>No sync jobs running or queued</p>
          <p className="text-sm mt-1">Select mailing lists above and click "Sync Now" to start</p>
        </div>
      )}
    </Card>
  );
}

function StatusBadge({ status }: { status: string }) {
  const variant = status === 'Failed' ? 'destructive' :
                   status === 'Completed' ? 'default' :
                   'secondary';

  return <Badge variant={variant}>{status}</Badge>;
}

function MetricItem({ label, value, warning = false }: { label: string; value: number; warning?: boolean }) {
  return (
    <div className="text-center">
      <p className={`text-xs ${warning ? 'text-yellow-700' : 'text-blue-700'} font-medium`}>{label}</p>
      <p className={`text-lg font-bold ${warning ? 'text-yellow-900' : 'text-blue-900'}`}>
        {value.toLocaleString()}
      </p>
    </div>
  );
}

function getProgressPercentage(progress: { processed: number; total: number | null }): number {
  if (!progress.total || progress.total === 0) return 0;
  return Math.min(100, (progress.processed / progress.total) * 100);
}
