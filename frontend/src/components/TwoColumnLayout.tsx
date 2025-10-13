import { type ReactNode } from 'react';

interface TwoColumnLayoutProps {
  left: ReactNode;
  right: ReactNode;
  leftWidth?: string;
}

export function TwoColumnLayout({ left, right, leftWidth = 'w-96' }: TwoColumnLayoutProps) {
  return (
    <div className="flex-1 flex overflow-hidden">
      {/* Left column - responsive width */}
      <div className={`${leftWidth} lg:${leftWidth} md:w-80 sm:w-72 flex-shrink-0 border-r border-border bg-card overflow-y-auto`}>
        {left}
      </div>
      {/* Right column */}
      <div className="flex-1 overflow-y-auto bg-background">
        {right}
      </div>
    </div>
  );
}
