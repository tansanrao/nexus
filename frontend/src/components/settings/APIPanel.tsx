import { useState } from 'react';
import { Server } from 'lucide-react';
import { useApiConfig } from '../../contexts/ApiConfigContext';
import { Input } from '../ui/input';
import { Section } from '../ui/section';
import { CompactButton } from '../ui/compact-button';

type TestStatus = 'idle' | 'testing' | 'success' | 'error';

export function APIPanel() {
  const { apiBaseUrl, setApiBaseUrl, resetToDefault, isDefault } = useApiConfig();
  const [inputValue, setInputValue] = useState(apiBaseUrl);
  const [testStatus, setTestStatus] = useState<TestStatus>('idle');
  const [testMessage, setTestMessage] = useState('');

  const hasChanges = inputValue !== apiBaseUrl;
  const trimmedInput = inputValue.trim();

  const save = () => {
    if (!trimmedInput) {
      return;
    }
    setApiBaseUrl(trimmedInput);
    setTestStatus('idle');
    setTestMessage('');
  };

  const cancel = () => {
    setInputValue(apiBaseUrl);
    setTestStatus('idle');
    setTestMessage('');
  };

  const reset = () => {
    resetToDefault();
    setInputValue(import.meta.env.VITE_API_URL || 'http://localhost:8000/api/v1');
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
      const base = trimmedInput.endsWith('/') ? trimmedInput.slice(0, -1) : trimmedInput;
      const response = await fetch(`${base}/admin/mailing-lists`, {
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
            placeholder="http://localhost:8000/api/v1"
            className="h-8 font-mono text-sm"
          />
          <CompactButton
            onClick={testConnection}
            disabled={!trimmedInput || testStatus === 'testing'}
            className="min-w-[72px]"
          >
            <Server className="h-3.5 w-3.5" />
            Test
          </CompactButton>
        </div>
        {testStatus !== 'idle' && (
          <div
            className="text-[11px] uppercase tracking-[0.08em]"
            data-status={testStatus}
          >
            <span className={
              testStatus === 'success'
                ? 'text-green-600'
                : testStatus === 'error'
                  ? 'text-destructive'
                  : 'text-muted-foreground'
            }>
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
            disabled={!hasChanges || !trimmedInput}
            className="px-3"
          >
            Save
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
