import { useParams } from 'react-router-dom';
import { TwoColumnLayout } from '../components/TwoColumnLayout';
import { ThreadListSidebar } from '../components/ThreadListSidebar';
import { ThreadView } from '../components/ThreadView';

export function ThreadBrowser() {
  const { threadId } = useParams<{ threadId: string }>();

  return (
    <TwoColumnLayout
      left={<ThreadListSidebar />}
      right={
        threadId ? (
          <ThreadView />
        ) : (
          <div className="h-full flex items-center justify-center">
            <p className="text-sm text-muted-foreground">Select a thread to view its content</p>
          </div>
        )
      }
    />
  );
}
