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
import { Section } from '../ui/section';
import { CompactButton } from '../ui/compact-button';

export function TimezonePanel() {
  const { timezone, setTimezone } = useTimezone();
  const systemTimezone = getSystemTimezone();
  const sampleDate = new Date().toISOString();
  const showReset = timezone !== systemTimezone;

  return (
    <Section
      title="Timezone"
      description="Choose how timestamps are rendered."
      actions={
        showReset && (
          <CompactButton onClick={() => setTimezone(systemTimezone)}>
            Use system ({systemTimezone})
          </CompactButton>
        )
      }
    >
      <div className="space-y-2">
        <label htmlFor="timezone-select" className="text-xs uppercase tracking-[0.08em] text-muted-foreground">
          Display zone
        </label>
        <Select value={timezone} onValueChange={setTimezone}>
          <SelectTrigger id="timezone-select" className="h-8 text-sm">
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

      <dl className="grid gap-2 text-[11px] uppercase tracking-[0.08em] text-muted-foreground">
        <div className="flex items-center justify-between gap-4">
          <dt>Selected</dt>
          <dd className="font-mono text-foreground">{timezone}</dd>
        </div>
        <div className="flex items-center justify-between gap-4">
          <dt>Offset</dt>
          <dd className="font-mono text-foreground">{getTimezoneOffset(timezone)}</dd>
        </div>
      </dl>

      <div className="surface-muted px-3 py-3 text-[12px] leading-relaxed">
        <div className="text-muted-foreground uppercase tracking-[0.08em] text-[11px]">
          Preview
        </div>
        <div className="font-mono text-[13px]">
          {formatDateWithTimezone(sampleDate, timezone)}
        </div>
      </div>
    </Section>
  );
}
