import { createContext, useContext } from 'react';

export interface TimezoneContextValue {
  timezone: string;
  setTimezone: (tz: string) => void;
}

export const TimezoneContext = createContext<TimezoneContextValue | undefined>(undefined);

export function useTimezone(): TimezoneContextValue {
  const context = useContext(TimezoneContext);
  if (!context) {
    throw new Error('useTimezone must be used within a TimezoneProvider');
  }
  return context;
}
