import { ScrollArea } from '../components/ui/scroll-area';
import { SyncPanel } from '../components/settings/SyncPanel';
import { DatabasePanel } from '../components/settings/DatabasePanel';
import { SearchMaintenancePanel } from '../components/settings/SearchMaintenancePanel';

export function SettingsDatabase() {
  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <ScrollArea className="flex-1">
        <div className="px-4 py-6 lg:px-8">
          <div className="mx-auto flex max-w-4xl flex-col gap-6">
            <header className="space-y-1">
              <h1 className="text-xl font-semibold uppercase tracking-[0.08em] text-muted-foreground">
                Database
              </h1>
              <p className="text-sm text-muted-foreground/80">
                Manage synchronization and low-level maintenance.
              </p>
            </header>

            <SearchMaintenancePanel />
            <SyncPanel />
            <DatabasePanel />
          </div>
        </div>
      </ScrollArea>
    </div>
  );
}
