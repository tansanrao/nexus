import { type ReactNode } from 'react';
import { TopNavigation } from './TopNavigation';
import { MailingListSubHeader } from './MailingListSubHeader';

interface AppLayoutProps {
  children: ReactNode;
}

export function AppLayout({ children }: AppLayoutProps) {
  return (
    <div className="h-screen flex flex-col bg-background">
      <TopNavigation />
      <MailingListSubHeader />
      <div className="flex-1 flex flex-col overflow-hidden">
        {children}
      </div>
    </div>
  );
}
