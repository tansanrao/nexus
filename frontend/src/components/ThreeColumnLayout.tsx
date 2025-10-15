import { type ReactNode } from 'react';
import { cn } from '@/lib/utils';

interface ThreeColumnLayoutProps {
  left: ReactNode;
  middle: ReactNode;
  right?: ReactNode;
  leftWidth?: string;
  middleWidth?: string;
}

export function ThreeColumnLayout({
  left,
  middle,
  right,
  leftWidth = 'w-80',
  middleWidth = 'w-96',
}: ThreeColumnLayoutProps) {
  return (
    <div className="flex-1 flex overflow-hidden bg-background">
      <div className={cn(
        `${leftWidth} lg:${leftWidth} md:w-72 flex-shrink-0 border-r border-border/60 bg-background overflow-hidden flex flex-col`
      )}>
        {left}
      </div>
      <div className={cn(
        `${middleWidth} lg:${middleWidth} md:w-80 flex-shrink-0 border-r border-border/60 bg-background overflow-hidden flex flex-col`
      )}>
        {middle}
      </div>
      {right && (
        <div className="flex-1 overflow-hidden bg-background">
          {right}
        </div>
      )}
      {!right && (
        <div className="flex-1 flex items-center justify-center">
          <p className="text-[12px] uppercase tracking-[0.08em] text-muted-foreground">
            Select a thread to view its content
          </p>
        </div>
      )}
    </div>
  );
}
