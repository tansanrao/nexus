import { type ReactNode } from 'react';

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
    <div className="flex-1 flex overflow-hidden">
      <div className={`${leftWidth} flex-shrink-0 border-r border-border bg-card overflow-y-auto`}>
        {left}
      </div>
      <div className={`${middleWidth} flex-shrink-0 border-r border-border bg-card overflow-y-auto`}>
        {middle}
      </div>
      {right && (
        <div className="flex-1 overflow-y-auto bg-background">
          {right}
        </div>
      )}
      {!right && (
        <div className="flex-1 flex items-center justify-center bg-muted/20">
          <p className="text-sm text-muted-foreground">Select a thread to view its content</p>
        </div>
      )}
    </div>
  );
}
