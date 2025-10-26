import { Outlet } from 'react-router-dom';
import { SettingsSubHeader } from '../components/SettingsSubHeader';

export function SettingsPage() {
  return (
    <div className="h-screen flex flex-col bg-background">
      <SettingsSubHeader />
      <div className="flex-1 flex flex-col overflow-hidden">
        <Outlet />
      </div>
    </div>
  );
}
