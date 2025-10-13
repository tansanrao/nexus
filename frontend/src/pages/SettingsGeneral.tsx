import { TimezonePanel } from '../components/settings/TimezonePanel';
import { ThemePanel } from '../components/settings/ThemePanel';
import { APIPanel } from '../components/settings/APIPanel';
import { ScrollArea } from '../components/ui/scroll-area';
import { Card } from '../components/ui/card';

export function SettingsGeneral() {
  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <ScrollArea className="flex-1">
        <div className="p-6">
          <div className="max-w-5xl mx-auto">
            <div className="mb-8">
              <h1 className="text-3xl font-bold mb-2">General Settings</h1>
              <p className="text-muted-foreground">
                Manage your general preferences and display settings.
              </p>
            </div>

            <div className="space-y-6">
              <Card className="p-6">
                <APIPanel />
              </Card>

              <Card className="p-6">
                <ThemePanel />
              </Card>

              <Card className="p-6">
                <TimezonePanel />
              </Card>
            </div>
          </div>
        </div>
      </ScrollArea>
    </div>
  );
}
