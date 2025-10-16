import { Sun, Moon, Monitor } from 'lucide-react';
import { useTheme } from '../../contexts/ThemeContext';
import { DEFAULT_LIGHT_THEME_ID, DEFAULT_DARK_THEME_ID } from '../../theme/registry';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../ui/select';
import { Section } from '../ui/section';
import { CompactButton } from '../ui/compact-button';

const modes = [
  { key: 'light', label: 'Light', icon: Sun },
  { key: 'dark', label: 'Dark', icon: Moon },
  { key: 'system', label: 'System', icon: Monitor },
] as const;

export function ThemePanel() {
  const {
    modePreference,
    lightSchemeId,
    darkSchemeId,
    availableLightThemes,
    availableDarkThemes,
    setModePreference,
    setLightScheme,
    setDarkScheme,
    resetDefaults,
  } = useTheme();

  const isDefault =
    modePreference === 'system' &&
    lightSchemeId === DEFAULT_LIGHT_THEME_ID &&
    darkSchemeId === DEFAULT_DARK_THEME_ID;

  return (
    <Section
      title="Theme"
      description="Switch light/dark mode and choose palettes."
      actions={
        !isDefault && (
          <CompactButton onClick={resetDefaults}>
            Reset
          </CompactButton>
        )
      }
    >
      <div className="grid grid-cols-3 gap-1">
        {modes.map(({ key, label, icon: Icon }) => (
          <CompactButton
            key={key}
            active={modePreference === key}
            onClick={() => setModePreference(key)}
            className="py-2"
          >
            <Icon className="h-3.5 w-3.5" />
            {label}
          </CompactButton>
        ))}
      </div>

      <div className="grid gap-3 sm:grid-cols-2">
        <div className="space-y-1">
          <label htmlFor="light-theme" className="text-xs uppercase tracking-[0.08em] text-muted-foreground">
            Light palette
          </label>
          <Select value={lightSchemeId} onValueChange={setLightScheme}>
            <SelectTrigger id="light-theme" className="h-8 text-sm">
              <SelectValue placeholder="Select light theme" />
            </SelectTrigger>
            <SelectContent>
              {availableLightThemes.map((theme) => (
                <SelectItem key={theme.id} value={theme.id}>
                  {theme.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-1">
          <label htmlFor="dark-theme" className="text-xs uppercase tracking-[0.08em] text-muted-foreground">
            Dark palette
          </label>
          <Select value={darkSchemeId} onValueChange={setDarkScheme}>
            <SelectTrigger id="dark-theme" className="h-8 text-sm">
              <SelectValue placeholder="Select dark theme" />
            </SelectTrigger>
            <SelectContent>
              {availableDarkThemes.map((theme) => (
                <SelectItem key={theme.id} value={theme.id}>
                  {theme.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>

      <p className="text-[11px] text-muted-foreground">
        Preferences are saved locally and follow system mode when selected.
      </p>
    </Section>
  );
}
