import type { SavedView } from '#lib/api/savedViews.ts';
import { FILTER_DEFAULTS, browseControls } from '#lib/stores/browseControls.svelte.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { savedViews } from '#lib/stores/savedViews.svelte.ts';
import { layout, renderLayout, settle } from '#lib/testing/layoutParity.ts';
import { goto } from '$app/navigation';
import { afterEach, describe, expect, it, vi } from 'vitest';
import SavedViewList from './SavedViewList.svelte';

vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/photos?rating=3'), params: {} }
}));

vi.mock('$app/navigation', () => ({ goto: vi.fn().mockResolvedValue(undefined) }));

function view(id: string, name: string, url: string): SavedView {
  const at = '2026-01-01T00:00:00Z';
  return { id, name, url, created_at: at, updated_at: at };
}

let cleanup: (() => Promise<void>) | null = null;

async function render(
  views: SavedView[],
  variant: 'sidebar' | 'list' = 'sidebar'
): Promise<HTMLAnchorElement[]> {
  savedViews.views = views;
  cleanup = renderLayout(layout(variant, SavedViewList, { variant }));
  await settle();
  return [...document.querySelectorAll<HTMLAnchorElement>('ul[aria-label="Saved views"] a')];
}

afterEach(async () => {
  await cleanup?.();
  cleanup = null;
  savedViews.views = [];
  vi.mocked(goto).mockClear();
});

describe('SavedViewList', () => {
  it('lists views as links to their path and marks the current one', async () => {
    const links = await render([
      view('a', 'Norway · X-T5', '/photos?country=Norway&model=X-T5&sort=taken-desc&size=md'),
      view('b', 'To edit · ≥ 3 stars', '/photos?rating=3&sort=added-asc&size=xl')
    ]);
    expect(links.map((link) => link.textContent.trim())).toEqual([
      'Norway · X-T5',
      'To edit · ≥ 3 stars'
    ]);
    expect(links.map((link) => link.getAttribute('href'))).toEqual([
      '/photos?country=Norway&model=X-T5',
      '/photos?rating=3'
    ]);
    expect(links.map((link) => link.getAttribute('aria-current'))).toEqual([null, 'page']);
  });

  it('renders library rows in the list variant', async () => {
    const links = await render(
      [view('a', 'Norway', '/photos?country=Norway&size=sm'), view('b', 'Zoo', '/albums')],
      'list'
    );
    expect(links.map((link) => link.getAttribute('href'))).toEqual([
      '/photos?country=Norway',
      '/albums'
    ]);
    expect(links.every((link) => !link.hasAttribute('type'))).toBe(true);
    expect(links[0]?.querySelector('.border-b')).not.toBeNull();
    expect(links[1]?.querySelector('.border-b')).toBeNull();
  });

  it('opens a view with its sort and size', async () => {
    browseControls.enter('timeline', FILTER_DEFAULTS);
    const [link] = await render([
      view('a', 'Norway', '/photos?country=Norway&sort=added-asc&size=xl')
    ]);
    link?.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true, button: 0 }));
    await settle();
    expect(goto).toHaveBeenCalledWith('/photos?country=Norway');
    expect(browseView.gridSize).toBe('xl');
    expect(browseControls.sort).toEqual({ field: 'added', direction: 'asc' });
  });

  it('leaves modified clicks to the browser', async () => {
    const [link] = await render([view('a', 'Norway', '/photos?country=Norway&size=sm')]);
    let handled = true;
    const browser = (event: MouseEvent): void => {
      handled = event.defaultPrevented;
      event.preventDefault();
    };
    document.addEventListener('click', browser);
    const click = new MouseEvent('click', {
      bubbles: true,
      cancelable: true,
      button: 0,
      metaKey: true
    });
    link?.dispatchEvent(click);
    document.removeEventListener('click', browser);
    await settle();
    expect(handled).toBe(false);
    expect(goto).not.toHaveBeenCalled();
  });
});
