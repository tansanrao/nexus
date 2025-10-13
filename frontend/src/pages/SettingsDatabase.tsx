import { SyncPanel } from '../components/settings/SyncPanel';
import { DatabasePanel } from '../components/settings/DatabasePanel';
import { ScrollArea } from '../components/ui/scroll-area';

export function SettingsDatabase() {
  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <ScrollArea className="flex-1">
        <div className="p-6">
          <div className="max-w-5xl mx-auto">
            <div className="mb-8">
              <h1 className="text-3xl font-bold mb-2">Database Management</h1>
              <p className="text-muted-foreground">
                Manage data synchronization and database operations.
              </p>
            </div>

            <div className="space-y-6">
              <SyncPanel />
              <DatabasePanel />
            </div>
          </div>
        </div>
      </ScrollArea>
    </div>
  );
}
