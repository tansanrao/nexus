import { formatDistanceToNow } from 'date-fns';

const DEFAULT_LOCALE = 'en-US';

const formatter = (timezone: string, options: Intl.DateTimeFormatOptions) =>
  new Intl.DateTimeFormat(DEFAULT_LOCALE, { timeZone: timezone, ...options });

const safeFormat = (date: Date, timezone: string, options: Intl.DateTimeFormatOptions) => {
  try {
    return formatter(timezone, options).format(date);
  } catch (error) {
    console.error('Error formatting date', error);
    return 'Invalid Date';
  }
};

export const getSystemTimezone = (): string => {
  return Intl.DateTimeFormat().resolvedOptions().timeZone;
};

export const formatDateInTimezone = (
  dateString: string,
  timezone: string,
  options: Intl.DateTimeFormatOptions = {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
    hour12: true,
  }
): string => {
  if (!dateString) return 'N/A';
  const date = new Date(dateString);
  if (Number.isNaN(date.getTime())) return 'Invalid Date';
  return safeFormat(date, timezone, options);
};

export const formatDateWithTimezone = (dateString: string, timezone: string): string => {
  if (!dateString) return 'N/A';
  const date = new Date(dateString);
  if (Number.isNaN(date.getTime())) return 'Invalid Date';
  return safeFormat(date, timezone, {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
    hour12: true,
    timeZoneName: 'short',
  });
};

export const formatDateCompact = (dateString: string, timezone: string): string => {
  if (!dateString) return 'N/A';
  const date = new Date(dateString);
  if (Number.isNaN(date.getTime())) return 'Invalid Date';

  const now = new Date();
  const diffInMs = now.getTime() - date.getTime();
  const diffInDays = diffInMs / (1000 * 60 * 60 * 24);

  if (diffInDays <= 7) {
    return formatDistanceToNow(date, { addSuffix: true });
  }

  const day = safeFormat(date, timezone, { day: '2-digit' });
  const month = safeFormat(date, timezone, { month: '2-digit' });
  const year = safeFormat(date, timezone, { year: 'numeric' });

  if ([day, month, year].includes('Invalid Date')) {
    return 'Invalid Date';
  }

  return `${day}-${month}-${year}`;
};

export const formatDateDetailed = (dateString: string, timezone: string): string => {
  if (!dateString) return 'N/A';
  const date = new Date(dateString);
  if (Number.isNaN(date.getTime())) return 'Invalid Date';
  return safeFormat(date, timezone, {
    weekday: 'long',
    month: 'long',
    day: 'numeric',
    year: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
    second: '2-digit',
    hour12: true,
    timeZoneName: 'short',
  });
};

export const getTimezoneAbbreviation = (dateString: string, timezone: string): string => {
  if (!dateString) return '';
  const date = new Date(dateString);
  if (Number.isNaN(date.getTime())) return '';

  try {
    const parts = formatter(timezone, {
      timeZoneName: 'short',
      hour: 'numeric',
    }).formatToParts(date);
    return parts.find((part) => part.type === 'timeZoneName')?.value ?? '';
  } catch (error) {
    console.error('Error getting timezone abbreviation', error);
    return '';
  }
};

export const getTimezoneOffset = (timezone: string): string => {
  const now = new Date();

  try {
    const parts = formatter(timezone, {
      timeZoneName: 'shortOffset',
      hour: 'numeric',
    }).formatToParts(now);
    const offset = parts.find((part) => part.type === 'timeZoneName')?.value;
    if (offset) {
      return offset.replace('GMT', 'UTC');
    }
  } catch (error) {
    console.error('Error getting timezone offset via Intl API', error);
  }

  // Fallback manual calculation
  const utc = now.getTime();
  const localTimezoneDate = new Date(now.toLocaleString('en-US', { timeZone: timezone }));
  const diffMinutes = Math.round((localTimezoneDate.getTime() - utc) / 60000);
  const sign = diffMinutes >= 0 ? '+' : '-';
  const absolute = Math.abs(diffMinutes);
  const hours = String(Math.floor(absolute / 60)).padStart(2, '0');
  const minutes = String(absolute % 60).padStart(2, '0');
  return `UTC${sign}${hours}${minutes === '00' ? '' : `:${minutes}`}`;
};

export const COMMON_TIMEZONES = [
  { value: 'America/New_York', label: 'Eastern Time (US)' },
  { value: 'America/Chicago', label: 'Central Time (US)' },
  { value: 'America/Denver', label: 'Mountain Time (US)' },
  { value: 'America/Los_Angeles', label: 'Pacific Time (US)' },
  { value: 'America/Anchorage', label: 'Alaska Time' },
  { value: 'Pacific/Honolulu', label: 'Hawaii Time' },
  { value: 'Europe/London', label: 'London (GMT/BST)' },
  { value: 'Europe/Paris', label: 'Central European Time' },
  { value: 'Europe/Berlin', label: 'Berlin Time' },
  { value: 'Europe/Moscow', label: 'Moscow Time' },
  { value: 'Asia/Dubai', label: 'Dubai Time' },
  { value: 'Asia/Kolkata', label: 'India Standard Time' },
  { value: 'Asia/Bangkok', label: 'Bangkok Time' },
  { value: 'Asia/Shanghai', label: 'China Standard Time' },
  { value: 'Asia/Tokyo', label: 'Japan Standard Time' },
  { value: 'Asia/Seoul', label: 'Korea Standard Time' },
  { value: 'Australia/Sydney', label: 'Australian Eastern Time' },
  { value: 'Australia/Melbourne', label: 'Melbourne Time' },
  { value: 'Pacific/Auckland', label: 'New Zealand Time' },
  { value: 'UTC', label: 'UTC (Coordinated Universal Time)' },
];
