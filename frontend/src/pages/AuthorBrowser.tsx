import { useParams } from 'react-router-dom';
import { ThreeColumnLayout } from '../components/ThreeColumnLayout';
import { AuthorListSidebar } from '../components/AuthorListSidebar';
import { AuthorDetailMiddle } from '../components/AuthorDetailMiddle';
import { ThreadView } from '../components/ThreadView';
import { MailingListHeader } from '../components/mailinglist/MailingListHeader';

export function AuthorBrowser() {
  const { authorId, threadId } = useParams<{ authorId: string; threadId: string }>();

  if (!authorId) {
    // Only show author list
    return (
      <>
        <MailingListHeader />
        <div className="flex-1 flex overflow-hidden">
          <div className="w-80 flex-shrink-0 border-r border-border bg-card overflow-y-auto">
            <AuthorListSidebar />
          </div>
          <div className="flex-1 flex items-center justify-center bg-muted/20">
            <p className="text-sm text-muted-foreground">Select an author to view their profile</p>
          </div>
        </div>
      </>
    );
  }

  return (
    <>
      <MailingListHeader />
      <ThreeColumnLayout
        left={<AuthorListSidebar />}
        middle={<AuthorDetailMiddle />}
        right={threadId ? <ThreadView /> : undefined}
      />
    </>
  );
}
