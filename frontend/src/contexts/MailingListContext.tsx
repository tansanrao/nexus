import { createContext, useContext, useState, useEffect } from 'react';
import type { ReactNode } from 'react';
import { api } from '../api/client';
import type { MailingList } from '../types';

const MAILING_LIST_STORAGE_KEY = 'selectedMailingList';

interface MailingListContextType {
  selectedMailingList: string | null;
  setSelectedMailingList: (slug: string) => void;
  mailingLists: MailingList[];
  loading: boolean;
}

const MailingListContext = createContext<MailingListContextType | undefined>(undefined);

export function MailingListProvider({ children }: { children: ReactNode }) {
  const [selectedMailingList, setSelectedMailingListState] = useState<string | null>(null);
  const [mailingLists, setMailingLists] = useState<MailingList[]>([]);
  const [loading, setLoading] = useState(true);

  // Load mailing lists on mount
  useEffect(() => {
    const loadMailingLists = async () => {
      try {
        const lists = await api.mailingLists.list();
        const enabledLists = lists.filter(list => list.enabled);
        setMailingLists(enabledLists);

        // Load saved mailing list from localStorage or use first enabled list
        const savedSlug = localStorage.getItem(MAILING_LIST_STORAGE_KEY);
        const savedListExists = savedSlug && enabledLists.some(list => list.slug === savedSlug);
        const defaultSlug = savedListExists ? savedSlug : (enabledLists.length > 0 ? enabledLists[0].slug : null);

        if (defaultSlug) {
          setSelectedMailingListState(defaultSlug);
        }
      } catch (error) {
        console.error('Failed to load mailing lists:', error);
      } finally {
        setLoading(false);
      }
    };

    loadMailingLists();
  }, []);

  const setSelectedMailingList = (slug: string) => {
    setSelectedMailingListState(slug);
    localStorage.setItem(MAILING_LIST_STORAGE_KEY, slug);
  };

  return (
    <MailingListContext.Provider
      value={{
        selectedMailingList,
        setSelectedMailingList,
        mailingLists,
        loading,
      }}
    >
      {children}
    </MailingListContext.Provider>
  );
}

export function useMailingList() {
  const context = useContext(MailingListContext);
  if (context === undefined) {
    throw new Error('useMailingList must be used within a MailingListProvider');
  }
  return context;
}
