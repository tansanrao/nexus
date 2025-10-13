import { createContext, useContext, useState, useEffect, type ReactNode } from 'react';

interface ApiConfigContextType {
  apiBaseUrl: string;
  setApiBaseUrl: (url: string) => void;
  resetToDefault: () => void;
  isDefault: boolean;
}

const ApiConfigContext = createContext<ApiConfigContextType | undefined>(undefined);

const API_BASE_URL_KEY = 'apiBaseUrl';
const DEFAULT_API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8000/api';

// Global variable to store the current API base URL
// This allows the API client to access it without using hooks
let currentApiBaseUrl = DEFAULT_API_BASE_URL;

export function getApiBaseUrl(): string {
  return currentApiBaseUrl;
}

export function ApiConfigProvider({ children }: { children: ReactNode }) {
  // Initialize from localStorage or defaults
  const [apiBaseUrl, setApiBaseUrlState] = useState<string>(() => {
    const saved = localStorage.getItem(API_BASE_URL_KEY);
    const url = saved || DEFAULT_API_BASE_URL;
    currentApiBaseUrl = url;
    return url;
  });

  const isDefault = apiBaseUrl === DEFAULT_API_BASE_URL;

  // Update global variable and localStorage when URL changes
  const setApiBaseUrl = (url: string) => {
    const trimmedUrl = url.trim();
    setApiBaseUrlState(trimmedUrl);
    localStorage.setItem(API_BASE_URL_KEY, trimmedUrl);
    currentApiBaseUrl = trimmedUrl;
  };

  const resetToDefault = () => {
    setApiBaseUrlState(DEFAULT_API_BASE_URL);
    localStorage.setItem(API_BASE_URL_KEY, DEFAULT_API_BASE_URL);
    currentApiBaseUrl = DEFAULT_API_BASE_URL;
  };

  // Sync with localStorage changes from other tabs
  useEffect(() => {
    const handleStorageChange = (e: StorageEvent) => {
      if (e.key === API_BASE_URL_KEY && e.newValue) {
        setApiBaseUrlState(e.newValue);
        currentApiBaseUrl = e.newValue;
      }
    };

    window.addEventListener('storage', handleStorageChange);
    return () => window.removeEventListener('storage', handleStorageChange);
  }, []);

  return (
    <ApiConfigContext.Provider
      value={{
        apiBaseUrl,
        setApiBaseUrl,
        resetToDefault,
        isDefault,
      }}
    >
      {children}
    </ApiConfigContext.Provider>
  );
}

export function useApiConfig() {
  const context = useContext(ApiConfigContext);
  if (context === undefined) {
    throw new Error('useApiConfig must be used within an ApiConfigProvider');
  }
  return context;
}
