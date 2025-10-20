import { useState, useEffect } from 'react';
import { Settings, Check, Loader2, XCircle, CheckCircle } from 'lucide-react';
import { Button } from './ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';
import { Input } from './ui/input';
import { useApiConfig } from '../contexts/ApiConfigContext';
import { apiClient } from '../lib/api';
import type { MailingList } from '../types';
import { cn } from '../lib/utils';

export function SettingsDropdown() {
  const { apiBaseUrl, setApiBaseUrl, resetToDefault, isDefault, selectedMailingList, setSelectedMailingList } =
    useApiConfig();
  const [mailingLists, setMailingLists] = useState<MailingList[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [tempApiUrl, setTempApiUrl] = useState(apiBaseUrl);
  const [testingConnection, setTestingConnection] = useState(false);
  const [testResult, setTestResult] = useState<'success' | 'error' | null>(null);
  const [savingUrl, setSavingUrl] = useState(false);

  useEffect(() => {
    loadMailingLists();
  }, [apiBaseUrl]);

  useEffect(() => {
    setTempApiUrl(apiBaseUrl);
  }, [apiBaseUrl]);

  const loadMailingLists = async () => {
    setLoading(true);
    setError(null);
    try {
      const lists = await apiClient.getMailingLists();
      setMailingLists(lists);

      // If no mailing list is selected, select the first enabled one
      if (!selectedMailingList && lists.length > 0) {
        const firstEnabled = lists.find((list) => list.enabled);
        if (firstEnabled) {
          setSelectedMailingList(firstEnabled.slug);
        }
      }
    } catch (err) {
      setError('Failed to load mailing lists');
      console.error('Error loading mailing lists:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleSaveApiUrl = async () => {
    setSavingUrl(true);
    setTestResult(null);
    try {
      const cleanUrl = tempApiUrl.trim();
      setApiBaseUrl(cleanUrl);
      // Small delay for visual feedback
      await new Promise(resolve => setTimeout(resolve, 300));
    } finally {
      setSavingUrl(false);
    }
  };

  const handleTestConnection = async () => {
    setTestingConnection(true);
    setTestResult(null);
    try {
      // Save the URL first
      const cleanUrl = tempApiUrl.trim();
      setApiBaseUrl(cleanUrl);
      
      // Wait a moment for the URL to update
      await new Promise(resolve => setTimeout(resolve, 100));
      
      const result = await apiClient.testConnection();
      setTestResult(result ? 'success' : 'error');
    } catch {
      setTestResult('error');
    } finally {
      setTestingConnection(false);
    }
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          className="data-[state=open]:bg-muted/80 data-[state=open]:text-foreground bg-transparent"
        >
          <Settings className="h-5 w-5" />
          <span className="sr-only">Settings</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-72 space-y-1">
        <div className="flex items-start justify-between gap-3 px-2 py-1.5">
          <div>
            <DropdownMenuLabel className="px-0 text-sm">Settings</DropdownMenuLabel>
            <p className="text-xs text-muted-foreground">Configure data sources</p>
          </div>
          <div className="rounded-md bg-muted/30 px-2 py-0.5 text-xs font-medium uppercase tracking-wide text-muted-foreground">
            {selectedMailingList || 'No list'}
          </div>
        </div>
        <DropdownMenuSeparator />

        <DropdownMenuLabel className="text-xs text-muted-foreground px-2 py-1">
          Mailing Lists
        </DropdownMenuLabel>
        <div className="rounded-md border border-surface-border/60 bg-muted/20 p-1">
          <div className="max-h-52 overflow-y-auto space-y-1">
            {loading ? (
              <DropdownMenuItem disabled>Loading…</DropdownMenuItem>
            ) : mailingLists.filter((list) => list.enabled).length === 0 ? (
              <DropdownMenuItem disabled>No mailing lists available</DropdownMenuItem>
            ) : (
              mailingLists
                .filter((list) => list.enabled)
                .map((list) => (
                  <DropdownMenuItem
                    key={list.slug}
                    className="flex-col items-start gap-1.5 rounded-md px-2.5 py-1.5 data-[highlighted]:bg-primary/10"
                    onClick={() => setSelectedMailingList(list.slug)}
                  >
                    <div className="flex w-full items-center gap-2">
                      <Check
                        className={cn(
                          "h-3.5 w-3.5 shrink-0 transition-opacity",
                          selectedMailingList === list.slug ? "opacity-100" : "opacity-0"
                        )}
                      />
                      <div className="flex flex-col">
                        <span className="font-medium leading-none">{list.name}</span>
                        <span className="text-[11px] text-muted-foreground">
                          {list.description || 'No description provided'}
                        </span>
                      </div>
                      <span className="ml-auto shrink-0 rounded-full bg-muted/60 px-2 py-0.5 text-[10px] uppercase tracking-wide text-muted-foreground">
                        {list.slug}
                      </span>
                    </div>
                  </DropdownMenuItem>
                ))
            )}
          </div>
        </div>

        <DropdownMenuSeparator />

        {/* API Configuration */}
        <DropdownMenuLabel className="text-xs text-muted-foreground px-2 py-1">
          API Endpoint
        </DropdownMenuLabel>
        <div className="rounded-md bg-muted/30 px-2.5 py-2 space-y-2">
          <Input
            type="url"
            placeholder="http://localhost:8000"
            value={tempApiUrl}
            onChange={(e) => setTempApiUrl(e.target.value)}
            className="h-8 text-xs"
          />
          <div className="text-[11px] text-muted-foreground">
            Active endpoint:
            <span className="ml-1 font-medium text-foreground">{apiBaseUrl}</span>
          </div>
          <div className="flex gap-1">
            <Button 
              onClick={handleSaveApiUrl} 
              size="sm" 
              className="flex-1 h-7 text-xs min-w-0 hover:underline"
              disabled={savingUrl}
            >
              {savingUrl && <Loader2 className="h-3 w-3 animate-spin" />}
              {!savingUrl && "Save"}
            </Button>
            <Button
              onClick={handleTestConnection}
              size="sm"
              variant="outline"
              className="flex-1 h-7 text-xs min-w-0 hover:underline"
              disabled={testingConnection}
            >
              {testingConnection && <Loader2 className="h-3 w-3 animate-spin" />}
              {!testingConnection && testResult === 'success' && (
                <>
                  <CheckCircle className="h-3 w-3 text-green-500" />
                  <span className="ml-1">Test</span>
                </>
              )}
              {!testingConnection && testResult === 'error' && (
                <>
                  <XCircle className="h-3 w-3 text-red-500" />
                  <span className="ml-1">Test</span>
                </>
              )}
              {!testingConnection && testResult === null && "Test"}
            </Button>
          </div>
          {!isDefault && (
            <Button
              onClick={resetToDefault}
              size="sm"
              variant="ghost"
              className="w-full h-7 text-xs hover:underline"
            >
              Reset to Default
            </Button>
          )}
        </div>
        <DropdownMenuSeparator />

        {error && (
          <div className="px-2 py-1">
            <p className="text-xs text-destructive">{error}</p>
          </div>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
