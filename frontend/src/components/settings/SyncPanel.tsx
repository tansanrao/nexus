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
          <h3 className="text-lg font-medium">Select Mailing Lists</h3>
          <p className="text-sm text-muted-foreground">
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
            className="flex-1 px-4 py-2 border rounded-lg focus:ring-2 focus:ring-ring focus:border-transparent"
          />
          <label className="flex items-center gap-2 px-4 py-2 border rounded-lg cursor-pointer hover:bg-accent">
            <input
              type="checkbox"
              checked={showEnabledOnly}
              onChange={(e) => setShowEnabledOnly(e.target.checked)}
              className="w-4 h-4 rounded focus:ring-ring"
            />
            <span className="text-sm">Enabled only</span>
          </label>
        </div>

        {/* Info banner */}
        <Card className="mb-3 p-3 bg-primary/5 border-primary/20 text-sm">
          <p><strong>Note:</strong> Grokmirror mirrors ALL lists automatically. The "enabled" toggle only controls which lists the API server will parse and import.</p>
        </Card>

        {/* Mailing lists */}
        <div className="space-y-2">
          {paginatedLists.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              {searchQuery || showEnabledOnly ? 'No mailing lists match your filters' : 'No mailing lists available. Click "Seed Mailing Lists" in the Database panel.'}
            </div>
          ) : (
            paginatedLists.map((list) => (
              <div
                key={list.id}
                className="flex items-center justify-between p-3 border rounded-lg hover:bg-accent"
              >
                <div className="flex items-center gap-3 flex-1 min-w-0">
                  <label className="flex items-center cursor-pointer">
                    <input
                      type="checkbox"
                      checked={list.enabled}
                      onChange={() => handleToggle(list.slug, list.enabled)}
                      className="w-5 h-5 rounded focus:ring-ring"
                      disabled={isRunning}
                    />
                    <span className="ml-3 font-medium">{list.name}</span>
                  </label>
                  <span className="text-xs text-muted-foreground font-mono">({list.slug})</span>
                  {list.description && (
                    <span className="text-sm text-muted-foreground truncate">- {list.description}</span>
                  )}
                </div>
                {list.last_synced_at && (
                  <span className="text-xs text-muted-foreground whitespace-nowrap ml-3">
                    Last synced: {new Date(list.last_synced_at).toLocaleString()}
                  </span>
                )}
              </div>
            ))
          )}
        </div>

        {/* Pagination */}
        {totalPages > 1 && (
          <div className="flex items-center justify-between mt-4 pt-4 border-t">
            <p className="text-sm text-muted-foreground">
              Showing {((currentPage - 1) * itemsPerPage) + 1} - {Math.min(currentPage * itemsPerPage, filteredLists.length)} of {filteredLists.length}
            </p>
            <div className="flex gap-2">
              <Button
                onClick={() => setCurrentPage(p => Math.max(1, p - 1))}
                disabled={currentPage === 1}
                variant="outline"
                size="sm"
              >
                Previous
              </Button>
              <span className="px-3 py-1 text-sm flex items-center">
                Page {currentPage} of {totalPages}
              </span>
              <Button
                onClick={() => setCurrentPage(p => Math.min(totalPages, p + 1))}
                disabled={currentPage === totalPages}
                variant="outline"
                size="sm"
              >
                Next
              </Button>
            </div>
          </div>
        )}
      </div>

      {/* Current Job Status */}
      {currentJob && (
        <div className="mb-6">
          <h3 className="text-lg font-medium mb-3">Current Sync Job</h3>
          <Card className="p-4 bg-primary/5 border-primary/20">
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <p className="font-medium">{currentJob.name}</p>
                <p className="text-sm text-muted-foreground mt-1">({currentJob.slug})</p>
                {currentJob.started_at && (
                  <p className="text-xs text-muted-foreground mt-2">
                    Started: {new Date(currentJob.started_at).toLocaleString()}
                  </p>
                )}
              </div>
              <PhaseBadge phase={currentJob.phase} />
            </div>
          </Card>
        </div>
      )}

      {/* Queue */}
      {syncStatus && syncStatus.queued_jobs.length > 0 && (
        <div>
          <Card className="p-4 bg-muted/50">
            <p className="text-sm">
              <span className="font-medium">{syncStatus.queued_jobs.length}</span> job{syncStatus.queued_jobs.length > 1 ? 's' : ''} in queue
            </p>
          </Card>
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

function PhaseBadge({ phase }: { phase: string }) {
  const config: Record<string, { label: string; variant: 'default' | 'secondary' | 'destructive' }> = {
    waiting: { label: 'Waiting', variant: 'secondary' },
    parsing: { label: 'Parsing', variant: 'default' },
    threading: { label: 'Threading', variant: 'default' },
    done: { label: 'Done', variant: 'default' },
    errored: { label: 'Error', variant: 'destructive' },
  };

  const { label, variant } = config[phase] || config.waiting;
  return <Badge variant={variant}>{label}</Badge>;
}

