import { useState, useEffect } from 'react';
import { api } from '../../api/client';
import type { AdminConfig } from '../../types';

export function ConfigPanel() {
  const [config, setConfig] = useState<AdminConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    setLoading(true);
    try {
      const data = await api.admin.config.get();
      setConfig(data);
      setError(null);
    } catch (err) {
      console.error('Failed to fetch config:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch configuration');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="bg-white rounded-lg border border-gray-200 p-6">
      <h2 className="text-xl font-semibold text-gray-900 mb-6">Configuration</h2>

      {error && (
        <div className="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg text-red-800">
          {error}
        </div>
      )}

      {loading ? (
        <div className="text-center py-8 text-gray-500">Loading configuration...</div>
      ) : config ? (
        <div className="space-y-4">
          <ConfigItem label="Repository URL" value={config.repo_url} />
          <ConfigItem label="Mirror Path" value={config.mirror_path} monospace />

          <div className="pt-4 border-t border-gray-200">
            <p className="text-sm text-gray-600">
              These settings are configured via environment variables on the server.
            </p>
          </div>
        </div>
      ) : null}
    </div>
  );
}

function ConfigItem({ label, value, monospace = false }: { label: string; value: string; monospace?: boolean }) {
  return (
    <div>
      <label className="block text-sm font-medium text-gray-700 mb-1">{label}</label>
      <div className={`px-3 py-2 bg-gray-50 border border-gray-200 rounded-lg ${monospace ? 'font-mono text-sm' : ''}`}>
        {value}
      </div>
    </div>
  );
}
