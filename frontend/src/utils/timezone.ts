import { formatDistanceToNow } from 'date-fns';
import { formatInTimeZone, toZonedTime } from 'date-fns-tz';

// Get the user's system timezone as default
export const getSystemTimezone = (): string => {
  return Intl.DateTimeFormat().resolvedOptions().timeZone;
};

// Format a UTC date string to a specific timezone
export const formatDateInTimezone = (
  dateString: string,
  timezone: string,
  formatString: string = 'MMM d, yyyy h:mm a'
): string => {
  try {
    if (!dateString) {
      return 'N/A';
    }
    const date = new Date(dateString);
    if (isNaN(date.getTime())) {
      console.warn('Invalid date string:', dateString);
      return 'Invalid Date';
    }
    return formatInTimeZone(date, timezone, formatString);
  } catch (error) {
    console.error('Error formatting date:', error);
    return dateString;
  }
};

// Format date with timezone abbreviation (e.g., "Jan 10, 2025 10:30 AM PST")
export const formatDateWithTimezone = (
  dateString: string,
  timezone: string
): string => {
  try {
    if (!dateString) {
      return 'N/A';
    }
    const date = new Date(dateString);
    if (isNaN(date.getTime())) {
      console.warn('Invalid date string:', dateString);
      return 'Invalid Date';
    }
    const formatted = formatInTimeZone(date, timezone, 'MMM d, yyyy h:mm a');
    const abbr = formatInTimeZone(date, timezone, 'zzz');
    return `${formatted} ${abbr}`;
  } catch (error) {
    console.error('Error formatting date with timezone:', error);
    return dateString;
  }
};

// Format for compact display (like in lists)
export const formatDateCompact = (
  dateString: string,
  timezone: string
): string => {
  try {
    if (!dateString) {
      return 'N/A';
    }
    const date = new Date(dateString);
    if (isNaN(date.getTime())) {
      console.warn('Invalid date string:', dateString);
      return 'Invalid Date';
    }
    const now = new Date();
    const diffInMs = now.getTime() - date.getTime();
    const diffInHours = diffInMs / (1000 * 60 * 60);

    // If less than 24 hours ago, show relative time
    if (diffInHours < 24) {
      return formatDistanceToNow(date, { addSuffix: true });
    }

    // If less than 7 days ago, show day of week
    if (diffInHours < 7 * 24) {
      return formatInTimeZone(date, timezone, 'EEEE');
    }

    // If within the same year, show month and day
    const zonedDate = toZonedTime(date, timezone);
    const zonedNow = toZonedTime(now, timezone);
    if (zonedDate.getFullYear() === zonedNow.getFullYear()) {
      return formatInTimeZone(date, timezone, 'MMM d');
    }

    // Otherwise show full date
    return formatInTimeZone(date, timezone, 'MMM d, yyyy');
  } catch (error) {
    console.error('Error formatting date compact:', error);
    return dateString;
  }
};

// Format for email detail view (more verbose)
export const formatDateDetailed = (
  dateString: string,
  timezone: string
): string => {
  try {
    if (!dateString) {
      return 'N/A';
    }
    const date = new Date(dateString);
    if (isNaN(date.getTime())) {
      console.warn('Invalid date string:', dateString);
      return 'Invalid Date';
    }
    return formatInTimeZone(date, timezone, 'EEEE, MMMM d, yyyy \'at\' h:mm:ss a zzz');
  } catch (error) {
    console.error('Error formatting date detailed:', error);
    return dateString;
  }
};

// Get timezone abbreviation for a given timezone
export const getTimezoneAbbreviation = (
  dateString: string,
  timezone: string
): string => {
  try {
    if (!dateString) {
      return '';
    }
    const date = new Date(dateString);
    if (isNaN(date.getTime())) {
      console.warn('Invalid date string:', dateString);
      return '';
    }
    return formatInTimeZone(date, timezone, 'zzz');
  } catch (error) {
    console.error('Error getting timezone abbreviation:', error);
    return '';
  }
};

// Get the UTC offset for a timezone (e.g., "UTC-5" or "UTC+5:30")
export const getTimezoneOffset = (timezone: string): string => {
  try {
    const now = new Date();
    const offsetStr = formatInTimeZone(now, timezone, 'XXX');
    return `UTC${offsetStr}`;
  } catch (error) {
    console.error('Error getting timezone offset:', error);
    return 'UTC';
  }
};

// List of common timezones (can be expanded as needed)
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
