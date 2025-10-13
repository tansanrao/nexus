import { useTheme } from '../../contexts/ThemeContext';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../ui/select';
import { Button } from '../ui/button';
import { Card } from '../ui/card';
import { Sun, Moon, Monitor } from 'lucide-react';

export function ThemePanel() {
  const { themeMode, lightTheme, darkTheme, setThemeMode, setLightTheme, setDarkTheme } = useTheme();

  const handleResetToDefaults = () => {
    setThemeMode('system');
    setLightTheme('catppuccin-latte');
    setDarkTheme('catppuccin-mocha');
  };

  const isDefault =
    themeMode === 'system' &&
    lightTheme === 'catppuccin-latte' &&
    darkTheme === 'catppuccin-mocha';

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold mb-4">Theme Settings</h2>
        <p className="text-sm text-muted-foreground mb-6">
          Customize the appearance of the application with your preferred color themes.
          Choose between light and dark modes, and select your favorite theme for each.
        </p>
      </div>

      <div className="space-y-4">
        {/* Theme Mode Selector */}
        <div>
          <label htmlFor="theme-mode" className="block text-sm font-medium mb-2">
            Theme Mode
          </label>
          <div className="grid grid-cols-3 gap-3">
            <Button
              variant={themeMode === 'light' ? 'default' : 'outline'}
              className="w-full justify-start gap-2"
              onClick={() => setThemeMode('light')}
            >
              <Sun className="h-4 w-4" />
              Light
            </Button>
            <Button
              variant={themeMode === 'dark' ? 'default' : 'outline'}
              className="w-full justify-start gap-2"
              onClick={() => setThemeMode('dark')}
            >
              <Moon className="h-4 w-4" />
              Dark
            </Button>
            <Button
              variant={themeMode === 'system' ? 'default' : 'outline'}
              className="w-full justify-start gap-2"
              onClick={() => setThemeMode('system')}
            >
              <Monitor className="h-4 w-4" />
              System
            </Button>
          </div>
        </div>

        {/* Light Theme Selector */}
        <div>
          <label htmlFor="light-theme" className="block text-sm font-medium mb-2">
            Light Theme
          </label>
          <Select value={lightTheme} onValueChange={setLightTheme}>
            <SelectTrigger className="w-full max-w-md">
              <SelectValue placeholder="Select light theme" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="catppuccin-latte">Catppuccin Latte</SelectItem>
              <SelectItem value="solarized-light">Solarized Light</SelectItem>
            </SelectContent>
          </Select>
        </div>

        {/* Dark Theme Selector */}
        <div>
          <label htmlFor="dark-theme" className="block text-sm font-medium mb-2">
            Dark Theme
          </label>
          <Select value={darkTheme} onValueChange={setDarkTheme}>
            <SelectTrigger className="w-full max-w-md">
              <SelectValue placeholder="Select dark theme" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="catppuccin-mocha">Catppuccin Mocha</SelectItem>
              <SelectItem value="solarized-dark">Solarized Dark</SelectItem>
            </SelectContent>
          </Select>
        </div>

        {/* Reset Button */}
        {!isDefault && (
          <div>
            <Button onClick={handleResetToDefaults} variant="outline">
              Reset to Defaults
            </Button>
          </div>
        )}
      </div>

      {/* Info Box */}
      <Card className="p-4 bg-muted/50">
        <h3 className="text-sm font-medium mb-2">About Themes</h3>
        <ul className="text-sm text-muted-foreground space-y-1 list-disc list-inside">
          <li>Catppuccin: Modern pastel color palette with smooth gradients</li>
          <li>Solarized: Precision color scheme optimized for readability</li>
          <li>System mode automatically switches between light and dark based on your OS settings</li>
          <li>Your preferences are saved in your browser</li>
        </ul>
      </Card>
    </div>
  );
}
