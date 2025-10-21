import { useCallback, useEffect, useMemo, useState, type ReactNode } from 'react';
import { getSystemTimezone } from '../utils/timezone';
import {
  TimezoneContext,
  type TimezoneContextValue,
} from './timezone-context';

const TIMEZONE_STORAGE_KEY = 'nexus.userTimezone';

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

  const setTimezone = useCallback((tz: string) => {
    setTimezoneState(tz);
  }, []);

  const contextValue = useMemo<TimezoneContextValue>(
    () => ({
      timezone,
      setTimezone,
    }),
    [timezone, setTimezone]
  );

  return (
    <TimezoneContext.Provider value={contextValue}>
      {children}
    </TimezoneContext.Provider>
  );
}
