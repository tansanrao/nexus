import { useState } from 'react';
import { useApiConfig } from '../../contexts/ApiConfigContext';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Card } from '../ui/card';
import { Server, Check, X } from 'lucide-react';

export function APIPanel() {
  const { apiBaseUrl, setApiBaseUrl, resetToDefault, isDefault } = useApiConfig();
  const [inputValue, setInputValue] = useState(apiBaseUrl);
  const [testStatus, setTestStatus] = useState<'idle' | 'testing' | 'success' | 'error'>('idle');
  const [testMessage, setTestMessage] = useState('');

  const hasChanges = inputValue !== apiBaseUrl;

  const handleSave = () => {
    setApiBaseUrl(inputValue);
    setTestStatus('idle');
    setTestMessage('');
  };

  const handleReset = () => {
    resetToDefault();
    setInputValue(import.meta.env.VITE_API_URL || 'http://localhost:8000/api');
    setTestStatus('idle');
    setTestMessage('');
  };

  const handleCancel = () => {
    setInputValue(apiBaseUrl);
  };

  const handleTest = async () => {
    setTestStatus('testing');
    setTestMessage('Testing connection...');

    try {
      const testUrl = inputValue.endsWith('/') ? inputValue.slice(0, -1) : inputValue;
      const response = await fetch(`${testUrl}/admin/config`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
      });

      if (response.ok) {
        setTestStatus('success');
        setTestMessage('Connection successful!');
      } else {
        setTestStatus('error');
        setTestMessage(`Connection failed: ${response.status} ${response.statusText}`);
      }
    } catch (error) {
      setTestStatus('error');
      setTestMessage(`Connection failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold mb-4">API Configuration</h2>
        <p className="text-sm text-muted-foreground mb-6">
          Configure the backend API endpoint. Change this if your backend server is running on a different host or port.
        </p>
      </div>

      <div className="space-y-4">
        {/* API Base URL Input */}
        <div>
          <label htmlFor="api-base-url" className="block text-sm font-medium mb-2">
            API Base URL
          </label>
          <div className="flex gap-2">
            <div className="flex-1">
              <Input
                id="api-base-url"
                type="text"
                value={inputValue}
                onChange={(e) => setInputValue(e.target.value)}
                placeholder="http://localhost:8000/api"
                className="font-mono text-sm"
              />
            </div>
            <Button
              onClick={handleTest}
              variant="outline"
              disabled={!inputValue.trim() || testStatus === 'testing'}
            >
              <Server className="h-4 w-4 mr-2" />
              Test
            </Button>
          </div>

          {/* Test Status Message */}
          {testStatus !== 'idle' && (
            <div className="mt-2 flex items-center gap-2">
              {testStatus === 'testing' && (
                <div className="text-xs text-muted-foreground">{testMessage}</div>
              )}
              {testStatus === 'success' && (
                <>
                  <Check className="h-4 w-4 text-green-600" />
                  <div className="text-xs text-green-600">{testMessage}</div>
                </>
              )}
              {testStatus === 'error' && (
                <>
                  <X className="h-4 w-4 text-destructive" />
                  <div className="text-xs text-destructive">{testMessage}</div>
                </>
              )}
            </div>
          )}
        </div>

        {/* Action Buttons */}
        <div className="flex items-center gap-2">
          <Button
            onClick={handleSave}
            disabled={!hasChanges || !inputValue.trim()}
          >
            Save Changes
          </Button>
          {hasChanges && (
            <Button onClick={handleCancel} variant="outline">
              Cancel
            </Button>
          )}
          {!isDefault && (
            <Button onClick={handleReset} variant="outline">
              Reset to Default
            </Button>
          )}
        </div>

        {/* Current Configuration Display */}
        <div className="pt-2">
          <div className="text-xs text-muted-foreground">
            Current API endpoint: <code className="font-mono bg-muted px-1.5 py-0.5 rounded">{apiBaseUrl}</code>
          </div>
        </div>
      </div>

      {/* Info Box */}
      <Card className="p-4 bg-muted/50">
        <h3 className="text-sm font-medium mb-2">About API Configuration</h3>
        <ul className="text-sm text-muted-foreground space-y-1 list-disc list-inside">
          <li>The default endpoint is configured via the VITE_API_URL environment variable</li>
          <li>Changes are saved to your browser's local storage</li>
          <li>Use the "Test" button to verify connectivity before saving</li>
          <li>The API URL should not include a trailing slash</li>
          <li>You may need to refresh the page after changing the endpoint for all components to update</li>
        </ul>
      </Card>
    </div>
  );
}
