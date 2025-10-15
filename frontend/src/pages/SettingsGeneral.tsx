import { TimezonePanel } from '../components/settings/TimezonePanel';
import { ThemePanel } from '../components/settings/ThemePanel';
import { APIPanel } from '../components/settings/APIPanel';
import { ScrollArea } from '../components/ui/scroll-area';

export function SettingsGeneral() {
  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <ScrollArea className="flex-1">
        <div className="px-4 py-6 lg:px-8">
          <div className="mx-auto flex max-w-4xl flex-col gap-6">
            <header className="space-y-1">
              <h1 className="text-xl font-semibold uppercase tracking-[0.08em] text-muted-foreground">
                General
              </h1>
              <p className="text-sm text-muted-foreground/80">
                Compact controls for connectivity, theming, and timestamps.
              </p>
            </header>

            <div className="grid gap-6">
              <APIPanel />
              <ThemePanel />
              <TimezonePanel />
            </div>
          </div>
        </div>
      </ScrollArea>
    </div>
  );
}
