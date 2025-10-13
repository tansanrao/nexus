import { Outlet } from 'react-router-dom';
import { SettingsSubHeader } from '../components/SettingsSubHeader';

export function Settings() {
  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <SettingsSubHeader />
      <Outlet />
    </div>
  );
}
