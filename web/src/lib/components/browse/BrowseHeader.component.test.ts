import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { resetLayout } from '#lib/testing/layoutMock.ts';
import { browseControls, FILTER_DEFAULTS } from '#lib/stores/browseControls.svelte.ts';
import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import BrowseHeader from './BrowseHeader.svelte';

vi.mock('#lib/device/layout.svelte.ts', async () => ({
  layout: (await import('#lib/testing/layoutMock.ts')).fakeLayout
}));

vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/photos'), params: {} } }));

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));

type HeaderProps = ComponentProps<typeof BrowseHeader>;

const Host: Component<{ component: Component<HeaderProps>; props: HeaderProps }> = ComponentHost;

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function render(mode: 'desktop' | 'phone' | 'wide'): void {
  resetLayout(
    mode === 'desktop' ? {} : { mode: 'touch', wide: mode === 'wide', landscape: mode === 'wide' }
  );
  component = mount(Host, {
    target,
    props: { component: BrowseHeader, props: { title: 'Photos', totalCount: 12 } }
  });
  flushSync();
}

function labelled(name: string): HTMLElement | null {
  return target.querySelector<HTMLElement>(`[aria-label="${name}"]`);
}

beforeEach(() => {
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
  browseControls.applyFilters({ ...FILTER_DEFAULTS });
  flushSync();
});

describe('BrowseHeader', () => {
  it('shows the desktop toolbar inline', () => {
    render('desktop');

    expect(target.querySelector('h1')?.textContent).toBe('Photos');
    expect(target.textContent).toContain('12 photos');
    expect(labelled('Thumbnail size S')).not.toBeNull();
    expect(labelled('Thumbnail info: badges')).not.toBeNull();
    expect(labelled('Filters')).not.toBeNull();
    expect(labelled('Account')).toBeNull();
    expect(labelled('Select photos')).toBeNull();
  });

  it('shows the touch app bar with the avatar, one View & filters button and Select', () => {
    render('phone');

    expect(target.querySelector('h1')?.textContent).toBe('Photos');
    expect(target.textContent).toContain('12 photos');
    expect(labelled('Account')).not.toBeNull();
    expect(labelled('View & filters')).not.toBeNull();
    expect(labelled('Select photos')).not.toBeNull();
    expect(labelled('View options')).toBeNull();
    expect(labelled('Thumbnail size S')).toBeNull();
  });

  it('trades the inline view controls for the touch actions on a wide touch screen', () => {
    render('wide');

    expect(labelled('View & filters')).not.toBeNull();
    expect(labelled('Select photos')).not.toBeNull();
    expect(labelled('Thumbnail size S')).toBeNull();
    expect(labelled('Account')).toBeNull();
  });

  it('counts the active filters on the touch Filter badge and lists their chips', () => {
    render('phone');
    browseControls.applyFilters({ ...FILTER_DEFAULTS, favoriteOnly: true, make: ['SONY'] });
    flushSync();

    expect(labelled('View & filters (2 active)')).not.toBeNull();
    expect(target.querySelectorAll('[aria-label="Active filters"] [role="listitem"]')).toHaveLength(
      2
    );
  });
});
