import { useState, useEffect } from 'react';
import { ChevronRight } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import type { EmailHierarchy } from '../types';
import { formatRelativeTime } from '../utils/date';
import { cn } from '../lib/utils';
import { GitDiffViewer } from './GitDiffViewer';

interface EmailItemProps {
  email: EmailHierarchy;
  forceCollapsed?: boolean | null;
  hiddenReplyCount?: number;
}

export function EmailItem({ email, forceCollapsed = null, hiddenReplyCount = 0 }: EmailItemProps) {
  const [isCollapsed, setIsCollapsed] = useState(false);
  const navigate = useNavigate();

  // Sync with global expand/collapse control
  // When forceCollapsed is provided, mirror that state locally
  useEffect(() => {
    if (forceCollapsed !== null && forceCollapsed !== undefined) {
      setIsCollapsed(forceCollapsed);
    }
  }, [forceCollapsed]);

  const authorName = email.author_name || email.author_email.split('@')[0];
  
  // Parse out metadata from email body and exclude diff sections
  const parseEmailBody = (body: string | null, patchMetadata: any) => {
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

  return (
    <div style={indentationStyle}>
      <div className="px-3 py-2 rounded-md transition-colors">
        {/* Header - always visible */}
        <button
          type="button"
          onClick={() => setIsCollapsed(!isCollapsed)}
          className="w-full text-left cursor-pointer select-none focus:outline-none focus-visible:outline-none"
        >
          <div className="flex items-start gap-2">
            <ChevronRight
              className={cn(
                "h-4 w-4 text-muted-foreground flex-shrink-0 transition-transform mt-0.5",
                !isCollapsed && "rotate-90"
              )}
            />
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2 text-sm overflow-hidden">
                <span
                  onClick={handleAuthorActivate}
                  role="button"
                  tabIndex={0}
                  className="font-semibold text-foreground hover:underline cursor-pointer select-none shrink-0"
                  onKeyDown={handleAuthorActivate}
                >
                  {authorName}
                </span>
                {displaySubject && (
                  <span
                    className="text-sm text-foreground flex-1 min-w-0 block max-w-[min(32rem,100%)] truncate"
                    title={displaySubject}
                  >
                    {displaySubject}
                  </span>
                )}
                {isCollapsed && hiddenReplyCount > 0 && (
                  <span className="text-xs text-muted-foreground shrink-0">[{hiddenReplyCount} more]</span>
                )}
                <span className="text-xs text-muted-foreground shrink-0 ml-2 whitespace-nowrap">
                  {formatRelativeTime(email.date)}
                </span>
              </div>
            </div>
          </div>
        </button>

        {/* Body - only when expanded */}
        {!isCollapsed && (
          <div className="ml-6 mt-2 space-y-2">
            {cleanBody && (
              <pre className="text-sm whitespace-pre-wrap font-mono text-foreground leading-relaxed overflow-x-auto bg-surface-inset/70 p-3">
                {cleanBody}
              </pre>
            )}
            
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
