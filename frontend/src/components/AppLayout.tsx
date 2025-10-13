import { type ReactNode } from 'react';
import { AppSidebar } from './layout/AppSidebar';

interface AppLayoutProps {
  children: ReactNode;
}

export function AppLayout({ children }: AppLayoutProps) {
  return (
    <div className="h-screen flex bg-background">
      {/* Left: App Sidebar */}
      <AppSidebar />

      {/* Right: Sub-app Container */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {children}
      </div>
    </div>
  );
}
