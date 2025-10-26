import { ChevronRight } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import type { EmailHierarchy } from '../types';
import { formatRelativeTime } from '../utils/date';
import { cn } from '../lib/utils';
import { GitDiffViewer } from './GitDiffViewer';
import { EmailBody } from './EmailBody';

interface EmailItemProps {
  email: EmailHierarchy;
  isCollapsed: boolean;
  onCollapsedChange: (collapsed: boolean) => void;
  hiddenReplyCount?: number;
  isHidden?: boolean;
}

export function EmailItem({
  email,
  isCollapsed,
  onCollapsedChange,
  hiddenReplyCount = 0,
  isHidden = false,
}: EmailItemProps) {
  const navigate = useNavigate();

  const authorName = email.author_name || email.author_email.split('@')[0];
  
  // Parse out metadata from email body and exclude diff sections
  const parseEmailBody = (
    body: string | null,
    patchMetadata: EmailHierarchy['patch_metadata'],
  ) => {
    if (!body) return { cleanBody: '', parsedSubject: null };
    
    const lines = body.split('\n');
    const cleanLines: string[] = [];
    let parsedSubject: string | null = null;
    
    // Create a set of line indices to exclude (diff sections)
    const excludeLines = new Set<number>();
    if (patchMetadata && patchMetadata.diff_sections) {
      for (const section of patchMetadata.diff_sections) {
        for (let i = section.start_line; i <= section.end_line; i++) {
          excludeLines.add(i);
        }
      }
    }
    
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      
      // Skip From: line
      if (line.trim().startsWith('From:')) {
        continue;
      }
      
      // Extract and skip Subject: line
      if (line.trim().startsWith('Subject:')) {
        parsedSubject = line.replace(/^Subject:\s*/i, '').trim();
        continue;
      }
      
      // Skip diff sections based on patch metadata
      if (excludeLines.has(i)) {
        continue;
      }
      
      cleanLines.push(line);
    }
    
    // Remove leading empty lines
    while (cleanLines.length > 0 && cleanLines[0].trim() === '') {
      cleanLines.shift();
    }
    
    return {
      cleanBody: cleanLines.join('\n'),
      parsedSubject
    };
  };
  
  const { cleanBody, parsedSubject } = parseEmailBody(email.body, email.patch_metadata);
  const displaySubject = parsedSubject || email.subject;

  const handleAuthorActivate = (event: React.MouseEvent | React.KeyboardEvent) => {
    if ('key' in event) {
      if (event.key !== 'Enter' && event.key !== ' ') {
        return;
      }
      event.preventDefault();
    }

    event.stopPropagation();
    const searchParams = new URLSearchParams(window.location.search);
    searchParams.set('author', email.author_id.toString());
    navigate(`/?${searchParams.toString()}`);
  };

  const indentPx = Math.min((email.depth || 0) * 16, 8 * 16);
  const indentationStyle = indentPx
    ? { marginLeft: `${indentPx}px`, maxWidth: `calc(100% - ${indentPx}px)` }
    : undefined;

  if (isHidden) {
    return null;
  }

  return (
    <div style={indentationStyle}>
      <div className="px-3 py-2 rounded-md transition-colors">
        {/* Header - always visible */}
        <button
          type="button"
          onClick={() => onCollapsedChange(!isCollapsed)}
          className="w-full text-left cursor-pointer select-none focus:outline-none focus-visible:outline-none"
        >
          <div className="flex items-start gap-2 min-w-0">
            <ChevronRight
              className={cn(
                "h-4 w-4 text-muted-foreground flex-shrink-0 transition-transform mt-0.5",
                !isCollapsed && "rotate-90"
              )}
            />
            <div className="flex-1 min-w-0">
              <div className="flex items-start justify-between gap-3 min-w-0">
                <div className="flex flex-1 items-center gap-2 min-w-0 text-sm text-foreground">
                  <span
                    onClick={handleAuthorActivate}
                    role="button"
                    tabIndex={0}
                    className="font-semibold text-foreground hover:underline cursor-pointer select-none"
                    onKeyDown={handleAuthorActivate}
                  >
                    {authorName}
                  </span>
                  {displaySubject && (
                    <span
                      className="flex-1 min-w-0 truncate text-sm"
                      title={displaySubject}
                    >
                      {displaySubject}
                    </span>
                  )}
                </div>
                <div className="flex shrink-0 items-center gap-2 text-xs text-muted-foreground whitespace-nowrap">
                  {isCollapsed && hiddenReplyCount > 0 && (
                    <span>[{hiddenReplyCount} more]</span>
                  )}
                  <span>{formatRelativeTime(email.date)}</span>
                </div>
              </div>
            </div>
          </div>
        </button>

        {/* Body - only when expanded */}
        {!isCollapsed && (
          <div className="ml-6 mt-2 space-y-2">
            {cleanBody && <EmailBody body={cleanBody} />}
            
            {/* Git Diff Viewer - show if there's patch content or potential diff content */}
            {email.body && (
              <div className="max-w-full overflow-hidden">
                <GitDiffViewer
                  emailBody={email.body}
                  patchMetadata={email.patch_metadata}
                  gitCommitHash={email.git_commit_hash}
                />
              </div>
            )}
            
            <div className="text-xs text-muted-foreground pt-2">
              {email.message_id}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
