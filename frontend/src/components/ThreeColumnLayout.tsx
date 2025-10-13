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
      {/* Left column - responsive width */}
      <div className={`${leftWidth} lg:${leftWidth} md:w-72 flex-shrink-0 border-r border-border bg-card overflow-y-auto`}>
        {left}
      </div>
      {/* Middle column - responsive width */}
      <div className={`${middleWidth} lg:${middleWidth} md:w-80 flex-shrink-0 border-r border-border bg-card overflow-y-auto`}>
        {middle}
      </div>
      {/* Right column */}
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
