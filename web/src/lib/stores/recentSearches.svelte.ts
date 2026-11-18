import {
  sameRecents,
  storedRecents,
  withoutRecent,
  withRecent
} from '#lib/browse/recentSearches.ts';
import { readStored, writeStored } from '#lib/utils/storage.ts';

const STORAGE_KEY = 'immich-edit:recentSearches';

type Persisted = { items: string[] };

class RecentSearchesStore {
  items = $state<string[]>(storedRecents(readStored<Persisted>(STORAGE_KEY)?.items));
  #newestSession: symbol | undefined;

  record = (query: string, session?: symbol): void => {
    const continues = session !== undefined && session === this.#newestSession;
    this.#newestSession = session;
    this.#set(withRecent(this.items, query, continues));
  };

  remove = (query: string): void => {
    this.#set(withoutRecent(this.items, query));
  };

  clear = (): void => {
    this.#newestSession = undefined;
    this.#set([]);
  };

  #set(items: string[]): void {
    if (sameRecents(items, this.items)) return;
    this.items = items;
    writeStored(STORAGE_KEY, { items } satisfies Persisted);
  }
}

export const recentSearches = new RecentSearchesStore();
