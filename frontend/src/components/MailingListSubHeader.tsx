import { useEffect, useState } from 'react';
import { Link, useLocation, useNavigate } from 'react-router-dom';
import { api } from '../api/client';
import type { MailingList } from '../types';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './ui/select';
import { cn } from '@/lib/utils';

export function MailingListSubHeader() {
  const location = useLocation();
  const navigate = useNavigate();

  // Extract mailingList from pathname since we're outside Routes context
  const pathParts = location.pathname.split('/').filter(Boolean);
  const currentSlug = pathParts.length > 0 && !['settings'].includes(pathParts[0]) ? pathParts[0] : undefined;

  const [mailingLists, setMailingLists] = useState<MailingList[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const loadMailingLists = async () => {
      try {
        const lists = await api.mailingLists.list();
        setMailingLists(lists);
      } catch (error) {
        console.error('Failed to load mailing lists:', error);
      } finally {
        setLoading(false);
      }
    };

    loadMailingLists();
  }, []);

  const handleMailingListChange = (slug: string) => {
    // Maintain the current view type (threads or authors)
    if (location.pathname.includes('/authors')) {
      navigate(`/${slug}/authors`);
    } else {
      navigate(`/${slug}/threads`);
    }
  };

  // Determine page title
  const getPageTitle = () => {
    if (location.pathname.startsWith('/settings')) {
      return 'Settings';
    }
    if (location.pathname.includes('/authors')) {
      return 'Authors';
    }
    if (location.pathname.includes('/threads')) {
      return 'Threads';
    }
    return 'Home';
  };

  const pageTitle = getPageTitle();

  // Check if we're on a mailing list page by checking both the URL pattern and the slug
  const isMailingListPage = !!currentSlug && !location.pathname.startsWith('/settings');

  return (
    <div className="h-12 px-6 flex items-center justify-between border-b bg-card">
      {/* Left: Page Title */}
      <h2 className="text-base font-semibold">{pageTitle}</h2>

      {/* Right: Mailing List Controls (only for mailing list pages) */}
      {isMailingListPage && (
        <div className="flex items-center gap-4">
          {/* Mailing List Selector */}
          <div className="w-56">
            <Select
              value={currentSlug}
              onValueChange={handleMailingListChange}
              disabled={loading}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select mailing list" />
              </SelectTrigger>
              <SelectContent>
                {mailingLists.map((list) => (
                  <SelectItem key={list.id} value={list.slug}>
                    {list.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Secondary Navigation */}
          <nav className="flex gap-1">
            <Link
              to={`/${currentSlug}/threads`}
              className={cn(
                "px-3 py-1.5 rounded-md text-sm font-medium transition-colors",
                location.pathname.includes('/threads')
                  ? "bg-primary text-primary-foreground"
                  : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
              )}
            >
              Threads
            </Link>
            <Link
              to={`/${currentSlug}/authors`}
              className={cn(
                "px-3 py-1.5 rounded-md text-sm font-medium transition-colors",
                location.pathname.includes('/authors')
                  ? "bg-primary text-primary-foreground"
                  : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
              )}
            >
              Authors
            </Link>
          </nav>
        </div>
      )}
    </div>
  );
}
