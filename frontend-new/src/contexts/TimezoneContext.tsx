import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import { getSystemTimezone } from '../utils/timezone';

interface TimezoneContextValue {
  timezone: string;
  setTimezone: (tz: string) => void;
}

const TIMEZONE_STORAGE_KEY = 'nexus.userTimezone';

const TimezoneContext = createContext<TimezoneContextValue | undefined>(undefined);

export function TimezoneProvider({ children }: { children: ReactNode }) {
  const [timezone, setTimezoneState] = useState(() => {
    if (typeof window === 'undefined') {
      return 'UTC';
    }
    const stored = window.localStorage.getItem(TIMEZONE_STORAGE_KEY);
    return stored || getSystemTimezone();
  });

  useEffect(() => {
    if (typeof window === 'undefined') {
      return;
    }
    window.localStorage.setItem(TIMEZONE_STORAGE_KEY, timezone);
  }, [timezone]);

  const setTimezone = (tz: string) => {
    setTimezoneState(tz);
  };

  return (
    <TimezoneContext.Provider value={{ timezone, setTimezone }}>
      {children}
    </TimezoneContext.Provider>
  );
}

export function useTimezone() {
  const context = useContext(TimezoneContext);
  if (!context) {
    throw new Error('useTimezone must be used within a TimezoneProvider');
  }
  return context;
}
