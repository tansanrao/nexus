import { createContext, useContext, useState, useEffect, type ReactNode } from 'react';
import { getSystemTimezone } from '../utils/timezone';

interface TimezoneContextType {
  timezone: string;
  setTimezone: (tz: string) => void;
}

const TimezoneContext = createContext<TimezoneContextType | undefined>(undefined);

const TIMEZONE_STORAGE_KEY = 'userTimezone';

export function TimezoneProvider({ children }: { children: ReactNode }) {
  const [timezone, setTimezoneState] = useState<string>(() => {
    // Try to load from localStorage first
    const saved = localStorage.getItem(TIMEZONE_STORAGE_KEY);
    if (saved) {
      return saved;
    }
    // Fall back to system timezone
    return getSystemTimezone();
  });

  const setTimezone = (tz: string) => {
    setTimezoneState(tz);
    localStorage.setItem(TIMEZONE_STORAGE_KEY, tz);
  };

  // Save to localStorage whenever timezone changes
  useEffect(() => {
    localStorage.setItem(TIMEZONE_STORAGE_KEY, timezone);
  }, [timezone]);

  return (
    <TimezoneContext.Provider value={{ timezone, setTimezone }}>
      {children}
    </TimezoneContext.Provider>
  );
}

export function useTimezone() {
  const context = useContext(TimezoneContext);
  if (context === undefined) {
    throw new Error('useTimezone must be used within a TimezoneProvider');
  }
  return context;
}
