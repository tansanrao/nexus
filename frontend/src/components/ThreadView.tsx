import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { ChevronRight, Maximize2, Minimize2, Mail } from 'lucide-react';
import { api } from '../api/client';
import type { ThreadDetail } from '../types';
import { useTimezone } from '../contexts/TimezoneContext';
import { formatDateInTimezone } from '../utils/timezone';
import { Button } from './ui/button';
import { Card } from './ui/card';
import { Badge } from './ui/badge';
import { Separator } from './ui/separator';
import { ScrollArea } from './ui/scroll-area';
import { Avatar, AvatarFallback } from './ui/avatar';
import { cn } from '@/lib/utils';

export function ThreadView() {
  const { threadId, mailingList } = useParams<{ threadId: string; mailingList: string }>();
  const { timezone } = useTimezone();
  const [threadData, setThreadData] = useState<ThreadDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [collapsedEmails, setCollapsedEmails] = useState<Set<number>>(new Set());

  useEffect(() => {
    const loadThread = async () => {
      if (!threadId || !mailingList) return;

      try {
        setLoading(true);
        const data = await api.threads.get(mailingList, parseInt(threadId));
        setThreadData(data);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load thread');
      } finally {
        setLoading(false);
      }
    };

    loadThread();
  }, [threadId, mailingList]);

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
      {/* Thread header */}
      <div className="border-b bg-card px-6 py-4">
        <div className="flex items-start justify-between gap-4 mb-4">
          <h1 className="text-xl font-semibold">{threadData.thread.subject}</h1>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={expandAll}
            >
              <Maximize2 className="h-3 w-3 mr-1" />
              Expand All
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={collapseAll}
            >
              <Minimize2 className="h-3 w-3 mr-1" />
              Collapse All
            </Button>
          </div>
        </div>
        <div className="flex items-center gap-3 text-xs text-muted-foreground flex-wrap">
          <Badge variant="secondary">
            {threadData.thread.message_count || 0} messages
          </Badge>
          <span>Started {formatDate(threadData.thread.start_date)}</span>
          <span>•</span>
          <span>Last activity {formatDate(threadData.thread.last_date)}</span>
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
                  <Card className="overflow-hidden">
                    {/* Email header */}
                    <button
                      onClick={() => toggleEmailCollapse(email.id)}
                      className="w-full text-left hover:bg-accent transition-colors"
                    >
                      <div className="p-4 flex items-start gap-3">
                        <Avatar className="h-10 w-10 flex-shrink-0">
                          <AvatarFallback className="text-xs font-medium">
                            {getInitials(email.author_name, email.author_email)}
                          </AvatarFallback>
                        </Avatar>

                        <div className="flex-1 min-w-0">
                          <div className="flex items-start justify-between gap-4 mb-1">
                            <div className="flex-1 min-w-0">
                              <div className="flex items-center gap-2">
                                <ChevronRight
                                  className={cn(
                                    "h-4 w-4 text-muted-foreground flex-shrink-0 transition-transform",
                                    !isCollapsed && "rotate-90"
                                  )}
                                />
                                <span className="font-semibold text-sm truncate">
                                  {email.author_name || email.author_email}
                                </span>
                              </div>
                              <div className="text-xs text-muted-foreground ml-6 truncate">
                                {email.author_email}
                              </div>
                            </div>
                            <span className="text-xs text-muted-foreground whitespace-nowrap">
                              {formatDate(email.date)}
                            </span>
                          </div>

                          {email.subject !== threadData.thread.subject && (
                            <div className="text-sm text-foreground ml-6 mt-2">
                              {email.subject}
                            </div>
                          )}

                          {isCollapsed && email.body && (
                            <div className="text-xs text-muted-foreground ml-6 mt-2 line-clamp-2">
                              {email.body.substring(0, 150)}...
                            </div>
                          )}
                        </div>
                      </div>
                    </button>

                    {/* Email body */}
                    {!isCollapsed && (
                      <>
                        <Separator />
                        <div className="p-4 bg-muted/30">
                          <pre className="whitespace-pre-wrap font-mono text-xs leading-relaxed text-foreground">
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
