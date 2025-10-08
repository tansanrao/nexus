import { SyncPanel } from '../components/settings/SyncPanel';
import { DatabasePanel } from '../components/settings/DatabasePanel';
import { ConfigPanel } from '../components/settings/ConfigPanel';
import { TimezonePanel } from '../components/settings/TimezonePanel';
import { ScrollArea } from '../components/ui/scroll-area';
import { Card } from '../components/ui/card';

export function Settings() {
  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <ScrollArea className="flex-1">
        <div className="p-6">
          <div className="max-w-5xl mx-auto">
            <div className="mb-8">
              <h1 className="text-3xl font-bold mb-2">Settings</h1>
              <p className="text-muted-foreground">
                Manage data synchronization, database operations, timezone preferences, and system configuration.
              </p>
            </div>

            <div className="space-y-6">
              <Card className="p-6">
                <TimezonePanel />
              </Card>
              <SyncPanel />
              <DatabasePanel />
              <ConfigPanel />
            </div>
          </div>
        </div>
      </ScrollArea>
    </div>
  );
}
