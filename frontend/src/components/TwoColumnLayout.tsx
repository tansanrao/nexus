import { type ReactNode } from 'react';
import { cn } from '@/lib/utils';

interface TwoColumnLayoutProps {
  left: ReactNode;
  right: ReactNode;
  leftWidth?: string;
}

export function TwoColumnLayout({ left, right, leftWidth = 'w-96' }: TwoColumnLayoutProps) {
  return (
    <div className="flex-1 flex overflow-hidden bg-background">
      <div className={cn(
        `${leftWidth} lg:${leftWidth} md:w-80 sm:w-72 flex-shrink-0 border-r border-border/60 bg-background overflow-hidden flex flex-col`
      )}>
        {left}
      </div>
      <div className="flex-1 overflow-hidden bg-background">
        {right}
      </div>
    </div>
  );
}
