import { useState, useEffect } from 'react';
import { api } from '../api/client';
import type { DatabaseStatus } from '../types';
import { ScrollArea } from '../components/ui/scroll-area';
import { Section } from '../components/ui/section';
import { CompactButton } from '../components/ui/compact-button';

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
      setError(err instanceof Error ? err.message : 'Failed to fetch system statistics');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void loadStatus();
  }, []);

  const formatNumber = (num: number) => num.toLocaleString();
  const formatDate = (date: string | null) =>
    date ? new Date(date).toLocaleDateString() : 'N/A';

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <ScrollArea className="flex-1">
        <div className="px-4 py-6 lg:px-8">
          <div className="mx-auto flex max-w-4xl flex-col gap-6">
            <header className="space-y-1">
              <h1 className="text-xl font-semibold uppercase tracking-[0.08em] text-muted-foreground">
                System stats
              </h1>
              <p className="text-sm text-muted-foreground/80">
                Overview of database totals and data ranges.
              </p>
            </header>

            <Section
              title="Refresh"
              description="Pull the latest snapshot from the API."
              actions={
                <CompactButton onClick={() => void loadStatus()} disabled={loading}>
                  {loading ? 'Refreshing…' : 'Refresh'}
                </CompactButton>
              }
            >
              {error && (
                <p className="text-[12px] uppercase tracking-[0.08em] text-destructive">
                  {error}
                </p>
              )}
              {loading && !status && (
                <p className="text-[12px] uppercase tracking-[0.08em] text-muted-foreground">
                  Loading statistics…
                </p>
              )}
              {status && (
                <p className="text-[12px] uppercase tracking-[0.08em] text-muted-foreground">
                  Last checked {new Date().toLocaleString()}
                </p>
              )}
            </Section>

            {status && (
              <>
                <Section title="Database records" description="Total counts across indexed data.">
                  <div className="grid grid-cols-2 md:grid-cols-3 gap-3 text-sm">
                    <Stat label="Authors" value={formatNumber(status.total_authors)} />
                    <Stat label="Emails" value={formatNumber(status.total_emails)} />
                    <Stat label="Threads" value={formatNumber(status.total_threads)} />
                    <Stat label="Recipients" value={formatNumber(status.total_recipients)} />
                    <Stat label="References" value={formatNumber(status.total_references)} />
                    <Stat label="Thread memberships" value={formatNumber(status.total_thread_memberships)} />
                  </div>
                </Section>

                {status.date_range_start && status.date_range_end && (
                  <Section title="Data range" description="Time span covered by the indexed dataset.">
                    <div className="surface-muted px-3 py-3 text-sm">
                      <div className="flex flex-wrap items-center gap-3">
                        <span className="text-muted-foreground">Earliest:</span>
                        <span className="font-medium">{formatDate(status.date_range_start)}</span>
                        <span className="text-muted-foreground" aria-hidden="true">→</span>
                        <span className="text-muted-foreground">Latest:</span>
                        <span className="font-medium">{formatDate(status.date_range_end)}</span>
                      </div>
                    </div>
                  </Section>
                )}

                <Section title="Notes" description="How these measurements are produced.">
                  <ul className="list-disc list-inside space-y-1 text-sm text-muted-foreground">
                    <li>Counts include all mailing lists that are currently enabled.</li>
                    <li>Numbers update as sync jobs finish importing data.</li>
                    <li>Recipients and references reflect threading relationships.</li>
                    <li>Thread memberships represent participation within threads.</li>
                  </ul>
                </Section>
              </>
            )}
          </div>
        </div>
      </ScrollArea>
    </div>
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
