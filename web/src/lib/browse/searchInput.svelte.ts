import { goto } from '$app/navigation';
import { page } from '$app/state';
import { recentSearches } from '#lib/stores/recentSearches.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import { untrack } from 'svelte';

const DEBOUNCE_MS = 300;

function onSearchRoute(): boolean {
  return page.url.pathname === '/search';
}

export function runSearch(query: string, session?: symbol): void {
  const q = query.trim();
  if (!q) return;
  recentSearches.record(q, session);
  const onSearch = onSearchRoute();
  const url = onSearch ? new URL(page.url.href) : new URL('/search', page.url.href);
  url.searchParams.set('q', q);
  void goto(`${url.pathname}${url.search}`, { replace: onSearch, reset: false });
}

export class SearchInput {
  #timer: ReturnType<typeof setTimeout> | undefined;
  #emptyPath: string;
  #session = Symbol('search');

  constructor(emptyPath: string) {
    this.#emptyPath = emptyPath;
    $effect(() => {
      const query = onSearchRoute() ? (page.url.searchParams.get('q') ?? '') : '';
      untrack(() => {
        if (query !== ui.searchQuery.trim()) ui.searchQuery = query;
      });
    });
    $effect(() => () => clearTimeout(this.#timer));
  }

  input = (value: string): void => {
    ui.searchQuery = value;
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => this.#go(value), DEBOUNCE_MS);
  };

  inputEvent = (event: Event): void => {
    if (event.currentTarget instanceof HTMLInputElement) this.input(event.currentTarget.value);
  };

  submit = (): void => {
    clearTimeout(this.#timer);
    this.#go(ui.searchQuery);
    this.focus();
  };

  submitEvent = (event: SubmitEvent): void => {
    event.preventDefault();
    this.submit();
  };

  focus = (): void => {
    this.#session = Symbol('search');
  };

  clear = (): void => {
    clearTimeout(this.#timer);
    ui.searchQuery = '';
    this.focus();
    if (onSearchRoute()) void goto(this.#emptyPath, { replace: true });
  };

  #go(value: string): void {
    if (value.trim()) {
      runSearch(value, this.#session);
    } else if (onSearchRoute()) {
      void goto(this.#emptyPath, { replace: this.#emptyPath === '/search', reset: false });
    }
  }
}
