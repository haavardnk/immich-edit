import ComponentHost from '#lib/testing/ComponentHost.svelte';
import {
  createRawSnippet,
  flushSync,
  mount,
  unmount,
  type Component,
  type ComponentProps
} from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { browseControls, FILTER_DEFAULTS } from '#lib/stores/browseControls.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import BrowsePage from './BrowsePage.svelte';

vi.mock('$app/navigation', () => ({
  afterNavigate: (): void => {},
  goto: vi.fn()
}));

vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/search?q=red'), params: {} }
}));

type PageProps = ComponentProps<typeof BrowsePage>;

const Host: Component<{ component: Component<PageProps>; props: PageProps }> = ComponentHost;

const toolbar = createRawSnippet(() => ({
  render: () => '<div data-testid="toolbar"><button type="button">File name</button></div>'
}));

const emptyAction = createRawSnippet(() => ({
  render: () => '<button type="button">Search filenames instead</button>'
}));

const search: PageProps = {
  title: 'Search: red',
  assets: [],
  loading: false,
  loadingLabel: 'Searching…',
  headerWhileLoading: true,
  sortBasis: null,
  trashRejects: false,
  toolbar,
  emptyTitle: 'No results',
  emptyMessage: 'Nothing matched “red”.',
  emptyAction
};

const photos: PageProps = {
  title: 'Photos',
  assets: [],
  loading: false,
  loadingLabel: 'Loading photos…',
  emptyTitle: 'No photos'
};

const rejected: AssetSummary = {
  id: 'a1',
  originalFileName: 'a1.jpg',
  type: 'IMAGE',
  fileCreatedAt: '2024-01-01T00:00:00.000Z',
  updatedAt: '2024-01-01T00:00:00.000Z',
  checksum: null,
  isFavorite: false,
  exifInfo: null,
  tags: [{ id: 't1', name: 'reject', value: 'immich-edit/reject' }]
};

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function render(props: PageProps): void {
  component = mount(Host, { target, props: { component: BrowsePage, props } });
  flushSync();
}

function heading(level: string): string | undefined {
  return target.querySelector(level)?.textContent?.trim();
}

function button(name: string): HTMLButtonElement | undefined {
  return [...target.querySelectorAll('button')].find((b) => b.textContent?.trim() === name);
}

function status(): HTMLElement | null {
  return target.querySelector<HTMLElement>('[role="status"]');
}

beforeEach(() => {
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  browseControls.applyFilters(FILTER_DEFAULTS);
  if (component) void unmount(component);
  component = null;
  target.remove();
});

describe('BrowsePage', () => {
  it('renders the search toolbar under the header and the empty action', () => {
    render(search);

    expect(heading('h1')).toBe('Search: red');
    const header = target.querySelector('header');
    const bar = target.querySelector('[data-testid="toolbar"]');
    expect(header?.compareDocumentPosition(bar as Node)).toBe(Node.DOCUMENT_POSITION_FOLLOWING);
    expect(button('File name')).toBeDefined();
    expect(heading('h2')).toBe('No results');
    expect(target.textContent).toContain('Nothing matched “red”.');
    expect(button('Search filenames instead')).toBeDefined();
    expect(target.querySelector('[aria-label^="Sort:"]')).toBeNull();
  });

  it('keeps the header and toolbar mounted while a search loads', () => {
    render({ ...search, loading: true });

    expect(heading('h1')).toBe('Search: red');
    expect(button('File name')).toBeDefined();
    expect(status()?.textContent?.trim()).toBe('Searching…');
    expect(heading('h2')).toBeUndefined();
  });

  it('shows only the loading line by default', () => {
    render({ ...photos, loading: true });

    expect(status()?.textContent?.trim()).toBe('Loading photos…');
    expect(target.querySelector('header')).toBeNull();
  });

  it('shows the sort menu and no extras by default', () => {
    render(photos);

    expect(target.querySelector('[aria-label^="Sort:"]')).not.toBeNull();
    expect(target.querySelector('[data-testid="toolbar"]')).toBeNull();
    expect(button('Search filenames instead')).toBeUndefined();
  });

  it.each([
    ['without', photos],
    ['with', search]
  ])('shows an error before any loading line %s the header kept', (_case, props) => {
    render({ ...props, loading: true, error: 'Load failed' });

    expect(target.textContent).toContain('Load failed');
    expect(target.querySelector('header')).toBeNull();
    expect(status()).toBeNull();
  });

  it('offers to trash rejected photos when the page allows it', () => {
    browseControls.applyFilters({ ...FILTER_DEFAULTS, rejected: 'only' });
    render({ ...photos, assets: [rejected] });

    expect(button('Move rejected to trash')).toBeDefined();
  });

  it('hides the trash rejected action when the page turns it off', () => {
    browseControls.applyFilters({ ...FILTER_DEFAULTS, rejected: 'only' });
    render({ ...photos, assets: [rejected], trashRejects: false });

    expect(target.querySelector('[aria-label="Active filters"]')).not.toBeNull();
    expect(button('Move rejected to trash')).toBeUndefined();
  });
});
