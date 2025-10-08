import { useTimezone } from '../../contexts/TimezoneContext';
import {
  COMMON_TIMEZONES,
  getTimezoneOffset,
  formatDateWithTimezone,
  getSystemTimezone,
} from '../../utils/timezone';

export function TimezonePanel() {
  const { timezone, setTimezone } = useTimezone();
  const systemTimezone = getSystemTimezone();

  const handleTimezoneChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    setTimezone(event.target.value);
  };

  const handleResetToSystem = () => {
    setTimezone(systemTimezone);
  };

  // Sample date for preview
  const sampleDate = new Date().toISOString();

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Timezone Settings</h2>
        <p className="text-sm text-gray-600 mb-6">
          Choose how you want dates and times to be displayed throughout the application.
          All times are stored in UTC and converted to your selected timezone.
        </p>
      </div>

      <div className="space-y-4">
        {/* Timezone Selector */}
        <div>
          <label htmlFor="timezone-select" className="block text-sm font-medium text-gray-700 mb-2">
            Display Timezone
          </label>
          <select
            id="timezone-select"
            value={timezone}
            onChange={handleTimezoneChange}
            className="w-full max-w-md px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
          >
            {COMMON_TIMEZONES.map((tz) => (
              <option key={tz.value} value={tz.value}>
                {tz.label} ({getTimezoneOffset(tz.value)})
              </option>
            ))}
          </select>
        </div>

        {/* Current Info */}
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <h3 className="text-sm font-medium text-blue-900 mb-2">Current Settings</h3>
          <div className="text-sm text-blue-700 space-y-1">
            <p>
              <span className="font-medium">Selected Timezone:</span> {timezone}
            </p>
            <p>
              <span className="font-medium">Offset:</span> {getTimezoneOffset(timezone)}
            </p>
            <p>
              <span className="font-medium">System Timezone:</span> {systemTimezone}
            </p>
          </div>
        </div>

        {/* Reset Button */}
        {timezone !== systemTimezone && (
          <div>
            <button
              onClick={handleResetToSystem}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            >
              Reset to System Timezone
            </button>
          </div>
        )}

        {/* Preview */}
        <div className="border-t border-gray-200 pt-4">
          <h3 className="text-sm font-medium text-gray-700 mb-3">Preview</h3>
          <div className="bg-gray-50 rounded-lg p-4 space-y-2">
            <div className="text-sm">
              <span className="text-gray-600">Current time in your timezone:</span>
              <div className="font-medium text-gray-900 mt-1">
                {formatDateWithTimezone(sampleDate, timezone)}
              </div>
            </div>
            <div className="text-xs text-gray-500 mt-2">
              This is how dates will appear throughout the application.
            </div>
          </div>
        </div>
      </div>

      {/* Info Box */}
      <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
        <h3 className="text-sm font-medium text-gray-900 mb-2">About Timezones</h3>
        <ul className="text-sm text-gray-600 space-y-1 list-disc list-inside">
          <li>All email timestamps preserve the original sender's timezone information</li>
          <li>Dates are automatically adjusted for Daylight Saving Time</li>
          <li>Your preference is saved in your browser</li>
          <li>Changing timezone only affects how times are displayed, not the actual data</li>
        </ul>
      </div>
    </div>
  );
}
