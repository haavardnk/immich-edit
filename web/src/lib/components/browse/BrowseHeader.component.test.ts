import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { resetLayout } from '#lib/testing/layoutMock.ts';
import { browseControls, FILTER_DEFAULTS } from '#lib/stores/browseControls.svelte.ts';
import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import BrowseHeader from './BrowseHeader.svelte';

vi.mock('#lib/device/layout.svelte.ts', async () => ({
  layout: (await import('#lib/testing/layoutMock.ts')).fakeLayout
}));

type HeaderProps = ComponentProps<typeof BrowseHeader>;

const Host: Component<{ component: Component<HeaderProps>; props: HeaderProps }> = ComponentHost;

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function render(compact: boolean): void {
  resetLayout({ mode: compact ? 'touch' : 'desktop' });
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
    render(false);

    expect(target.querySelector('h1')?.textContent).toBe('Photos');
    expect(target.textContent).toContain('12 photos');
    expect(labelled('Thumbnail size S')).not.toBeNull();
    expect(labelled('Thumbnail info: badges')).not.toBeNull();
    expect(labelled('Account')).toBeNull();
    expect(labelled('View options')).toBeNull();
  });

  it('shows the touch app bar with the avatar, Filter and the view options', () => {
    render(true);

    expect(target.querySelector('h1')?.textContent).toBe('Photos');
    expect(target.textContent).toContain('12 photos');
    expect(labelled('Account')).not.toBeNull();
    expect(labelled('Filters')).not.toBeNull();
    expect(labelled('View options')).not.toBeNull();
    expect(labelled('Thumbnail size S')).toBeNull();
  });

  it('counts the active filters on the touch Filter badge and lists their chips', () => {
    render(true);
    browseControls.applyFilters({ ...FILTER_DEFAULTS, favoriteOnly: true, make: ['SONY'] });
    flushSync();

    expect(labelled('Filters (2 active)')).not.toBeNull();
    expect(target.querySelectorAll('[aria-label="Active filters"] [role="listitem"]')).toHaveLength(
      2
    );
  });
});
