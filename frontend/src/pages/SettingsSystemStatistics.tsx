import { useState, useEffect } from 'react';
import { api } from '../api/client';
import type { DatabaseStatus } from '../types';
import { ScrollArea } from '../components/ui/scroll-area';
import { Card } from '../components/ui/card';
import { Button } from '../components/ui/button';
import { RefreshCw } from 'lucide-react';

export function SettingsSystemStatistics() {
  const [status, setStatus] = useState<DatabaseStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadStatus = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.admin.database.getStatus();
      setStatus(data);
    } catch (err) {
      console.error('Failed to fetch database status:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch system statistics');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadStatus();
  }, []);

  const formatNumber = (num: number): string => {
    return num.toLocaleString();
  };

  const formatDate = (dateStr: string | null): string => {
    if (!dateStr) return 'N/A';
    return new Date(dateStr).toLocaleDateString();
  };

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <ScrollArea className="flex-1">
        <div className="p-6">
          <div className="max-w-5xl mx-auto">
            <div className="mb-8">
              <div className="flex items-center justify-between">
                <div>
                  <h1 className="text-3xl font-bold mb-2">System Statistics</h1>
                  <p className="text-muted-foreground">
                    Overview of database statistics and system metrics.
                  </p>
                </div>
                <Button
                  onClick={loadStatus}
                  disabled={loading}
                  variant="outline"
                  size="sm"
                >
                  <RefreshCw className={`h-4 w-4 mr-2 ${loading ? 'animate-spin' : ''}`} />
                  Refresh
                </Button>
              </div>
            </div>

            {error && (
              <Card className="mb-6 p-4 bg-destructive/10 border-destructive">
                <div className="text-sm text-destructive">{error}</div>
              </Card>
            )}

            {loading && !status ? (
              <div className="text-center py-12 text-muted-foreground">
                Loading statistics...
              </div>
            ) : status ? (
              <div className="space-y-6">
                {/* Main Statistics */}
                <Card className="p-6">
                  <h2 className="text-lg font-semibold mb-4">Database Records</h2>
                  <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
                    <StatCard label="Authors" value={formatNumber(status.total_authors)} />
                    <StatCard label="Emails" value={formatNumber(status.total_emails)} />
                    <StatCard label="Threads" value={formatNumber(status.total_threads)} />
                    <StatCard label="Recipients" value={formatNumber(status.total_recipients)} />
                    <StatCard label="References" value={formatNumber(status.total_references)} />
                    <StatCard label="Thread Memberships" value={formatNumber(status.total_thread_memberships)} />
                  </div>
                </Card>

                {/* Date Range */}
                {status.date_range_start && status.date_range_end && (
                  <Card className="p-6">
                    <h2 className="text-lg font-semibold mb-4">Data Range</h2>
                    <div className="flex items-center gap-4 text-sm">
                      <div>
                        <span className="text-muted-foreground">Earliest Email:</span>
                        <div className="font-medium mt-1">{formatDate(status.date_range_start)}</div>
                      </div>
                      <div className="text-muted-foreground">→</div>
                      <div>
                        <span className="text-muted-foreground">Latest Email:</span>
                        <div className="font-medium mt-1">{formatDate(status.date_range_end)}</div>
                      </div>
                    </div>
                  </Card>
                )}

                {/* Information Card */}
                <Card className="p-6 bg-muted/50">
                  <h3 className="text-sm font-medium mb-2">About These Statistics</h3>
                  <ul className="text-sm text-muted-foreground space-y-1 list-disc list-inside">
                    <li>Statistics are calculated across all enabled mailing lists</li>
                    <li>Data is updated in real-time as sync operations complete</li>
                    <li>Recipients and References track email threading relationships</li>
                    <li>Thread Memberships represent email participation in threads</li>
                  </ul>
                </Card>
              </div>
            ) : null}
          </div>
        </div>
      </ScrollArea>
    </div>
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
