import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { ChevronRight, Maximize2, Minimize2, Mail } from 'lucide-react';
import { api } from '../api/client';
import type { ThreadDetail } from '../types';
import { useTimezone } from '../contexts/TimezoneContext';
import { useMailingList } from '../contexts/MailingListContext';
import { formatDateInTimezone } from '../utils/timezone';
import { Button } from './ui/button';
import { Card } from './ui/card';
import { Badge } from './ui/badge';
import { Separator } from './ui/separator';
import { ScrollArea } from './ui/scroll-area';
import { Avatar, AvatarFallback } from './ui/avatar';
import { cn } from '@/lib/utils';

export function ThreadView() {
  const { threadId } = useParams<{ threadId: string }>();
  const { selectedMailingList } = useMailingList();
  const { timezone } = useTimezone();
  const [threadData, setThreadData] = useState<ThreadDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [collapsedEmails, setCollapsedEmails] = useState<Set<number>>(new Set());

  useEffect(() => {
    const loadThread = async () => {
      if (!threadId || !selectedMailingList) return;

      try {
        setLoading(true);
        const data = await api.threads.get(selectedMailingList, parseInt(threadId));
        setThreadData(data);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load thread');
      } finally {
        setLoading(false);
      }
    };

    loadThread();
  }, [threadId, selectedMailingList]);

  const formatDate = (dateStr: string) => {
    return formatDateInTimezone(dateStr, timezone, 'MMM d, yyyy h:mm a');
  };

  const toggleEmailCollapse = (emailId: number) => {
    setCollapsedEmails((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(emailId)) {
        newSet.delete(emailId);
      } else {
        newSet.add(emailId);
      }
      return newSet;
    });
  };

  const collapseAll = () => {
    if (!threadData) return;
    setCollapsedEmails(new Set(threadData.emails.map((email) => email.id)));
  };

  const expandAll = () => {
    setCollapsedEmails(new Set());
  };

  const getInitials = (name: string | null, email: string) => {
    if (name) {
      return name.split(' ').map(n => n[0]).slice(0, 2).join('').toUpperCase();
    }
    return email.substring(0, 2).toUpperCase();
  };

  if (loading) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-sm text-muted-foreground">Loading thread...</div>
      </div>
    );
  }

  if (error || !threadData) {
    return (
      <div className="h-full flex items-center justify-center">
        <Card className="p-6">
          <div className="text-sm text-destructive">Error: {error || 'Thread not found'}</div>
        </Card>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Thread header - Sticky */}
      <div className="border-b bg-card/50 backdrop-blur px-6 py-4 sticky top-0 z-10">
        {/* Subject line - larger */}
        <h1 className="text-2xl font-bold mb-3 leading-tight">
          {threadData.thread.subject}
        </h1>

        {/* Metadata bar */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4 text-sm text-muted-foreground">
            <div className="flex items-center gap-2">
              <Mail className="h-4 w-4" />
              <span>{threadData.thread.message_count || 0} messages</span>
            </div>
            <Separator orientation="vertical" className="h-4" />
            <span>Started {formatDate(threadData.thread.start_date)}</span>
            <Separator orientation="vertical" className="h-4" />
            <span>Last {formatDate(threadData.thread.last_date)}</span>
          </div>

          <div className="flex gap-2">
            <Button variant="ghost" size="sm" onClick={expandAll}>
              <Maximize2 className="h-4 w-4 mr-1.5" />
              Expand
            </Button>
            <Button variant="ghost" size="sm" onClick={collapseAll}>
              <Minimize2 className="h-4 w-4 mr-1.5" />
              Collapse
            </Button>
          </div>
        </div>
      </div>

      {/* Email messages */}
      <ScrollArea className="flex-1">
        <div className="p-6">
          <div className="space-y-4">
            {threadData.emails.map((email) => {
              const isCollapsed = collapsedEmails.has(email.id);
              return (
                <div
                  key={email.id}
                  style={{ marginLeft: `${email.depth * 24}px` }}
                >
                  <Card
                    className={cn(
                      "overflow-hidden transition-all",
                      isCollapsed ? "shadow-sm hover:shadow-md" : "shadow hover:shadow-lg"
                    )}
                  >
                    {/* Email header - always visible */}
                    <button
                      onClick={() => toggleEmailCollapse(email.id)}
                      className="w-full text-left px-4 py-3 hover:bg-accent/30 transition-colors"
                    >
                      <div className="flex items-start gap-3">
                        {/* Avatar - with ring */}
                        <Avatar className="h-10 w-10 ring-2 ring-border flex-shrink-0">
                          <AvatarFallback className="text-xs font-semibold bg-primary/10 text-primary">
                            {getInitials(email.author_name, email.author_email)}
                          </AvatarFallback>
                        </Avatar>

                        <div className="flex-1 min-w-0">
                          {/* Name & Time row */}
                          <div className="flex items-center justify-between mb-1">
                            <div className="flex items-center gap-2 flex-1 min-w-0">
                              <ChevronRight
                                className={cn(
                                  "h-4 w-4 text-muted-foreground flex-shrink-0 transition-transform duration-200",
                                  !isCollapsed && "rotate-90"
                                )}
                              />
                              <span className="font-semibold text-sm truncate">
                                {email.author_name || email.author_email}
                              </span>
                              {/* Depth indicator badge for replies */}
                              {email.depth > 0 && (
                                <Badge variant="outline" className="text-xs h-4 px-1 flex-shrink-0">
                                  ↳ {email.depth}
                                </Badge>
                              )}
                            </div>
                            <time className="text-xs text-muted-foreground whitespace-nowrap ml-2">
                              {formatDate(email.date)}
                            </time>
                          </div>

                          {/* Email - subtle */}
                          <div className="text-xs text-muted-foreground ml-6">
                            {email.author_email}
                          </div>

                          {/* Subject if different */}
                          {email.subject !== threadData.thread.subject && (
                            <div className="text-sm text-foreground ml-6 mt-2 font-medium">
                              {email.subject}
                            </div>
                          )}

                          {/* Preview when collapsed */}
                          {isCollapsed && email.body && (
                            <div className="text-sm text-muted-foreground ml-6 mt-2 line-clamp-2">
                              {email.body.substring(0, 120)}...
                            </div>
                          )}
                        </div>
                      </div>
                    </button>

                    {/* Body - only when expanded */}
                    {!isCollapsed && (
                      <>
                        <Separator />
                        <div className="px-4 py-4 bg-muted/20">
                          <pre className="whitespace-pre-wrap font-mono text-sm leading-relaxed">
                            {email.body || '(No message body)'}
                          </pre>
                        </div>
                      </>
                    )}
                  </Card>
                </div>
              );
            })}

            {threadData.emails.length === 0 && (
              <Card className="p-12">
                <div className="text-center">
                  <Mail className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
                  <p className="text-sm font-medium text-foreground">No messages in this thread</p>
                </div>
              </Card>
            )}
          </div>
        </div>
      </ScrollArea>
    </div>
  );
}
