import { useState, useEffect } from 'react';
import { Link, useParams } from 'react-router-dom';
import { Search, SortAsc, SortDesc, Users } from 'lucide-react';
import { api } from '../api/client';
import type { AuthorWithStats, AuthorSortBy, SortOrder } from '../types';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Badge } from './ui/badge';
import { ScrollArea } from './ui/scroll-area';
import { Avatar, AvatarFallback } from './ui/avatar';
import { cn } from '@/lib/utils';

export function AuthorListSidebar() {
  const { mailingList, authorId } = useParams<{ mailingList: string; authorId: string }>();
  const [authors, setAuthors] = useState<AuthorWithStats[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [page, setPage] = useState(1);
  const [sortBy, setSortBy] = useState<AuthorSortBy>('email_count');
  const [order, setOrder] = useState<SortOrder>('desc');
  const limit = 50;

  useEffect(() => {
    const loadAuthors = async () => {
      if (!mailingList) return;

      try {
        setLoading(true);
        const data = await api.authors.search(mailingList, {
          search: searchQuery || undefined,
          page,
          limit,
          sort_by: sortBy,
          order,
        });
        setAuthors(data);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load authors');
      } finally {
        setLoading(false);
      }
    };

    const debounce = setTimeout(() => {
      loadAuthors();
    }, 300);

    return () => clearTimeout(debounce);
  }, [searchQuery, page, sortBy, order, mailingList]);

  const handleSortChange = (newSortBy: AuthorSortBy) => {
    if (newSortBy === sortBy) {
      setOrder(order === 'desc' ? 'asc' : 'desc');
    } else {
      setSortBy(newSortBy);
      setOrder(newSortBy === 'canonical_name' || newSortBy === 'email' ? 'asc' : 'desc');
    }
    setPage(1);
  };

  const getInitials = (name: string | null | undefined, email: string) => {
    if (name) {
      return name.split(' ').map(n => n[0]).slice(0, 2).join('').toUpperCase();
    }
    return email.substring(0, 2).toUpperCase();
  };

  const getDisplayName = (author: AuthorWithStats) => {
    return author.canonical_name || author.name_variations?.[0] || author.email;
  };

  if (loading && authors.length === 0) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-sm text-muted-foreground">Loading authors...</div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Search and filters */}
      <div className="border-b p-4 space-y-3">
        {/* Search bar */}
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            type="text"
            placeholder="Search authors..."
            value={searchQuery}
            onChange={(e) => {
              setSearchQuery(e.target.value);
              setPage(1);
            }}
            className="pl-9 h-9"
          />
        </div>

        {/* Sort controls */}
        <div className="flex flex-col gap-1">
          <span className="text-xs font-medium text-muted-foreground mb-1">Sort by:</span>
          <div className="grid grid-cols-2 gap-1">
            <Button
              variant={sortBy === 'email_count' ? 'secondary' : 'ghost'}
              size="sm"
              onClick={() => handleSortChange('email_count')}
              className="h-8 text-xs justify-start"
            >
              Messages
              {sortBy === 'email_count' && (
                order === 'desc' ? <SortDesc className="ml-auto h-3 w-3" /> : <SortAsc className="ml-auto h-3 w-3" />
              )}
            </Button>
            <Button
              variant={sortBy === 'thread_count' ? 'secondary' : 'ghost'}
              size="sm"
              onClick={() => handleSortChange('thread_count')}
              className="h-8 text-xs justify-start"
            >
              Threads
              {sortBy === 'thread_count' && (
                order === 'desc' ? <SortDesc className="ml-auto h-3 w-3" /> : <SortAsc className="ml-auto h-3 w-3" />
              )}
            </Button>
            <Button
              variant={sortBy === 'canonical_name' ? 'secondary' : 'ghost'}
              size="sm"
              onClick={() => handleSortChange('canonical_name')}
              className="h-8 text-xs justify-start"
            >
              Name
              {sortBy === 'canonical_name' && (
                order === 'desc' ? <SortDesc className="ml-auto h-3 w-3" /> : <SortAsc className="ml-auto h-3 w-3" />
              )}
            </Button>
            <Button
              variant={sortBy === 'email' ? 'secondary' : 'ghost'}
              size="sm"
              onClick={() => handleSortChange('email')}
              className="h-8 text-xs justify-start"
            >
              Email
              {sortBy === 'email' && (
                order === 'desc' ? <SortDesc className="ml-auto h-3 w-3" /> : <SortAsc className="ml-auto h-3 w-3" />
              )}
            </Button>
          </div>
        </div>
      </div>

      {error && (
        <div className="mx-4 mt-4 p-3 bg-destructive/10 border border-destructive rounded-md">
          <div className="text-xs text-destructive">Error: {error}</div>
        </div>
      )}

      {/* Author list */}
      <ScrollArea className="flex-1">
        <div className="p-2">
          {authors.length === 0 ? (
            <div className="p-8 text-center">
              <Users className="h-8 w-8 mx-auto text-muted-foreground mb-2" />
              <p className="text-xs text-muted-foreground">No authors found</p>
            </div>
          ) : (
            <div className="space-y-0">
              {authors.map((author) => {
                const isSelected = authorId === String(author.id);
                return (
                  <Link
                    key={author.id}
                    to={`/${mailingList}/authors/${author.id}`}
                    className="block border-b last:border-b-0"
                  >
                    <div
                      className={cn(
                        "p-3 transition-colors hover:bg-accent",
                        isSelected && "bg-accent"
                      )}
                    >
                      <div className="flex items-start gap-3">
                        <Avatar className="h-8 w-8 flex-shrink-0">
                          <AvatarFallback className="text-xs font-medium">
                            {getInitials(author.canonical_name, author.email)}
                          </AvatarFallback>
                        </Avatar>
                        <div className="flex-1 min-w-0">
                          <div className="font-medium text-sm truncate">
                            {getDisplayName(author)}
                          </div>
                          <div className="text-xs text-muted-foreground truncate mb-2">
                            {author.email}
                          </div>
                          <div className="space-y-1 text-xs">
                            <div className="flex items-center gap-1.5">
                              <span className="text-muted-foreground font-medium">Messages:</span>
                              <Badge variant="secondary" className="text-xs px-1.5 py-0 h-5">
                                {author.email_count}
                              </Badge>
                            </div>
                            <div className="flex items-center gap-1.5">
                              <span className="text-muted-foreground font-medium">Threads:</span>
                              <Badge variant="outline" className="text-xs px-1.5 py-0 h-5">
                                {author.thread_count}
                              </Badge>
                            </div>
                            {author.mailing_lists && author.mailing_lists.length > 1 && (
                              <div className="flex items-center gap-1 flex-wrap">
                                {author.mailing_lists.slice(0, 2).map((ml) => (
                                  <Badge key={ml} variant="outline" className="text-xs px-1 py-0 h-4">
                                    {ml}
                                  </Badge>
                                ))}
                                {author.mailing_lists.length > 2 && (
                                  <span className="text-xs text-muted-foreground">+{author.mailing_lists.length - 2}</span>
                                )}
                              </div>
                            )}
                          </div>
                        </div>
                      </div>
                    </div>
                  </Link>
                );
              })}
            </div>
          )}
        </div>
      </ScrollArea>

      {/* Pagination */}
      <div className="border-t p-2">
        <div className="flex justify-between items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setPage((p) => Math.max(1, p - 1))}
            disabled={page === 1}
          >
            Prev
          </Button>
          <span className="text-xs text-muted-foreground">Page {page}</span>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setPage((p) => p + 1)}
            disabled={authors.length < limit}
          >
            Next
          </Button>
        </div>
      </div>
    </div>
  );
}
