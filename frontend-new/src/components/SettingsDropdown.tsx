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
  DropdownMenuSub,
  DropdownMenuSubTrigger,
  DropdownMenuSubContent,
} from './ui/dropdown-menu';
import { Input } from './ui/input';
import { useApiConfig } from '../contexts/ApiConfigContext';
import { apiClient } from '../lib/api';
import type { MailingList } from '../types';

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
        <Button variant="ghost" size="icon">
          <Settings className="h-5 w-5" />
          <span className="sr-only">Settings</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-80">
        <DropdownMenuLabel>Settings</DropdownMenuLabel>
        <DropdownMenuSeparator />

        {/* Mailing List Selector */}
        <DropdownMenuSub>
          <DropdownMenuSubTrigger>
            <span>Mailing List</span>
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent className="max-h-96 overflow-y-auto">
            {loading ? (
              <DropdownMenuItem disabled>Loading...</DropdownMenuItem>
            ) : mailingLists.filter((list) => list.enabled).length === 0 ? (
              <DropdownMenuItem disabled>No mailing lists available</DropdownMenuItem>
            ) : (
              mailingLists
                .filter((list) => list.enabled)
                .map((list) => (
                  <DropdownMenuItem
                    key={list.slug}
                    onClick={() => setSelectedMailingList(list.slug)}
                  >
                    {selectedMailingList === list.slug && <Check className="mr-2 h-4 w-4" />}
                    <span className={selectedMailingList !== list.slug ? 'ml-6' : ''}>
                      {list.name}
                    </span>
                  </DropdownMenuItem>
                ))
            )}
          </DropdownMenuSubContent>
        </DropdownMenuSub>

        <DropdownMenuSeparator />

        {/* API Configuration */}
        <DropdownMenuLabel className="text-xs text-muted-foreground">
          API Endpoint
        </DropdownMenuLabel>
        <div className="px-2 py-2 space-y-2">
          <Input
            type="url"
            placeholder="http://localhost:8000"
            value={tempApiUrl}
            onChange={(e) => setTempApiUrl(e.target.value)}
            className="h-8 text-xs"
          />
          <div className="flex gap-1">
            <Button 
              onClick={handleSaveApiUrl} 
              size="sm" 
              className="flex-1 h-7 text-xs min-w-0"
              disabled={savingUrl}
            >
              {savingUrl && <Loader2 className="h-3 w-3 animate-spin" />}
              {!savingUrl && "Save"}
            </Button>
            <Button
              onClick={handleTestConnection}
              size="sm"
              variant="outline"
              className="flex-1 h-7 text-xs min-w-0"
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
              className="w-full h-7 text-xs"
            >
              Reset to Default
            </Button>
          )}
        </div>

        {error && (
          <>
            <DropdownMenuSeparator />
            <div className="px-2 py-1">
              <p className="text-xs text-destructive">{error}</p>
            </div>
          </>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

