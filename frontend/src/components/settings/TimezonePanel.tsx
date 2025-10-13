import { useTimezone } from '../../contexts/TimezoneContext';
import {
  COMMON_TIMEZONES,
  getTimezoneOffset,
  formatDateWithTimezone,
  getSystemTimezone,
} from '../../utils/timezone';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../ui/select';
import { Button } from '../ui/button';
import { Card } from '../ui/card';

export function TimezonePanel() {
  const { timezone, setTimezone } = useTimezone();
  const systemTimezone = getSystemTimezone();

  const handleTimezoneChange = (value: string) => {
    setTimezone(value);
  };

  const handleResetToSystem = () => {
    setTimezone(systemTimezone);
  };

  // Sample date for preview
  const sampleDate = new Date().toISOString();

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold mb-4">Timezone Settings</h2>
        <p className="text-sm text-muted-foreground mb-6">
          Choose how you want dates and times to be displayed throughout the application.
          All times are stored in UTC and converted to your selected timezone.
        </p>
      </div>

      <div className="space-y-4">
        {/* Timezone Selector */}
        <div>
          <label htmlFor="timezone-select" className="block text-sm font-medium mb-2">
            Display Timezone
          </label>
          <Select value={timezone} onValueChange={handleTimezoneChange}>
            <SelectTrigger className="w-full max-w-md">
              <SelectValue placeholder="Select timezone" />
            </SelectTrigger>
            <SelectContent>
              {COMMON_TIMEZONES.map((tz) => (
                <SelectItem key={tz.value} value={tz.value}>
                  {tz.label} ({getTimezoneOffset(tz.value)})
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        {/* Current Info */}
        <Card className="p-4 bg-primary/5 border-primary/20">
          <h3 className="text-sm font-medium mb-2">Current Settings</h3>
          <div className="text-sm text-muted-foreground space-y-1">
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
        </Card>

        {/* Reset Button */}
        {timezone !== systemTimezone && (
          <div>
            <Button onClick={handleResetToSystem} variant="outline">
              Reset to System Timezone
            </Button>
          </div>
        )}

        {/* Preview */}
        <div className="border-t pt-4">
          <h3 className="text-sm font-medium mb-3">Preview</h3>
          <Card className="p-4 bg-muted space-y-2">
            <div className="text-sm">
              <span className="text-muted-foreground">Current time in your timezone:</span>
              <div className="font-medium mt-1">
                {formatDateWithTimezone(sampleDate, timezone)}
              </div>
            </div>
            <div className="text-xs text-muted-foreground mt-2">
              This is how dates will appear throughout the application.
            </div>
          </Card>
        </div>
      </div>

      {/* Info Box */}
      <Card className="p-4 bg-muted/50">
        <h3 className="text-sm font-medium mb-2">About Timezones</h3>
        <ul className="text-sm text-muted-foreground space-y-1 list-disc list-inside">
          <li>All email timestamps preserve the original sender's timezone information</li>
          <li>Dates are automatically adjusted for Daylight Saving Time</li>
          <li>Your preference is saved in your browser</li>
          <li>Changing timezone only affects how times are displayed, not the actual data</li>
        </ul>
      </Card>
    </div>
  );
}
