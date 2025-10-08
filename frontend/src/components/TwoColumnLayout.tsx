import { type ReactNode } from 'react';

interface TwoColumnLayoutProps {
  left: ReactNode;
  right: ReactNode;
  leftWidth?: string;
}

export function TwoColumnLayout({ left, right, leftWidth = 'w-96' }: TwoColumnLayoutProps) {
  return (
    <div className="flex-1 flex overflow-hidden">
      <div className={`${leftWidth} flex-shrink-0 border-r border-border bg-card overflow-y-auto`}>
        {left}
      </div>
      <div className="flex-1 overflow-y-auto bg-background">
        {right}
      </div>
    </div>
  );
}
