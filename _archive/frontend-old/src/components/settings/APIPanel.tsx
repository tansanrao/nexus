import { useState } from 'react';
import { Server } from 'lucide-react';
import { useApiConfig } from '../../contexts/ApiConfigContext';
import { Input } from '../ui/input';
import { Section } from '../ui/section';
import { CompactButton } from '../ui/compact-button';

type TestStatus = 'idle' | 'testing' | 'success' | 'error';

const resolveApiProbeUrl = (baseUrl: string) => {
  const trimmed = baseUrl.trim().replace(/\/+$/, '');
  if (/\/api(?:\/v1)?$/i.test(trimmed)) {
    return `${trimmed}/admin/mailing-lists`;
  }
  return `${trimmed}/api/v1/admin/mailing-lists`;
};

const DEFAULT_FALLBACK = import.meta.env.VITE_API_URL || 'http://localhost:8000';

export function APIPanel() {
  const { apiBaseUrl, setApiBaseUrl, resetToDefault, isDefault } = useApiConfig();
  const [inputValue, setInputValue] = useState(apiBaseUrl);
  const [testStatus, setTestStatus] = useState<TestStatus>('idle');
  const [testMessage, setTestMessage] = useState('');
  const [isSaving, setIsSaving] = useState(false);

  const hasChanges = inputValue.trim() !== apiBaseUrl.trim();
  const trimmedInput = inputValue.trim();

  const save = async () => {
    if (!trimmedInput) {
      return;
    }
    setIsSaving(true);
    try {
      setApiBaseUrl(trimmedInput);
      setTestStatus('idle');
      setTestMessage('');
    } finally {
      setIsSaving(false);
    }
  };

  const cancel = () => {
    setInputValue(apiBaseUrl);
    setTestStatus('idle');
    setTestMessage('');
  };

  const reset = () => {
    resetToDefault();
    setInputValue(DEFAULT_FALLBACK);
    setTestStatus('idle');
    setTestMessage('');
  };

  const testConnection = async () => {
    if (!trimmedInput || testStatus === 'testing') {
      return;
    }

    setTestStatus('testing');
    setTestMessage('Testing connection…');

    try {
      const probeUrl = resolveApiProbeUrl(trimmedInput);
      const response = await fetch(probeUrl, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
      });

      if (response.ok) {
        setTestStatus('success');
        setTestMessage('Connection OK');
      } else {
        setTestStatus('error');
        setTestMessage(`HTTP ${response.status} ${response.statusText}`);
      }
    } catch (err) {
      setTestStatus('error');
      setTestMessage(err instanceof Error ? err.message : 'Unknown error');
    }
  };

  return (
    <Section
      title="API endpoint"
      description="Point the UI at your backend service."
      actions={
        !isDefault && (
          <CompactButton onClick={reset}>
            Reset default
          </CompactButton>
        )
      }
    >
      <div className="space-y-2">
        <label htmlFor="api-base-url" className="text-xs uppercase tracking-[0.08em] text-muted-foreground">
          Base URL
        </label>
        <div className="flex gap-2">
          <Input
            id="api-base-url"
            type="text"
            value={inputValue}
            onChange={(event) => setInputValue(event.target.value)}
            placeholder="http://localhost:8000"
            className="h-8 font-mono text-sm"
          />
          <CompactButton
            onClick={testConnection}
            disabled={!trimmedInput || testStatus === 'testing'}
            className="min-w-[80px]"
          >
            <Server className="h-3.5 w-3.5" />
            Test
          </CompactButton>
        </div>
        {testStatus !== 'idle' && (
          <div className="text-[11px] uppercase tracking-[0.08em]">
            <span
              className={
                testStatus === 'success'
                  ? 'text-green-600'
                  : testStatus === 'error'
                    ? 'text-destructive'
                    : 'text-muted-foreground'
              }
            >
              {testMessage}
            </span>
          </div>
        )}
      </div>

      <div className="flex flex-wrap items-center gap-2 text-[11px] uppercase tracking-[0.08em]">
        <span className="text-muted-foreground">
          Current:&nbsp;
          <code className="font-mono text-foreground">{apiBaseUrl}</code>
        </span>
        <div className="ml-auto flex gap-2">
          <CompactButton
            onClick={save}
            disabled={!hasChanges || !trimmedInput || isSaving}
            className="px-3"
          >
            {isSaving ? 'Saving…' : 'Save'}
          </CompactButton>
          {hasChanges && (
            <CompactButton onClick={cancel} className="px-3">
              Cancel
            </CompactButton>
          )}
        </div>
      </div>
    </Section>
  );
}
