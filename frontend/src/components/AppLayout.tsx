import { type ReactNode } from 'react';
import { AppHeader } from './layout/AppHeader';

interface AppLayoutProps {
  children: ReactNode;
}

export function AppLayout({ children }: AppLayoutProps) {
  return (
    <div className="h-svh flex flex-col bg-background text-foreground text-sm">
      <AppHeader />
      <div className="flex-1 flex flex-col overflow-hidden bg-background">
        {children}
      </div>
    </div>
  );
}
