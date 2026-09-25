import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import Loupe from './Loupe.svelte';
import ComponentHost from '$lib/testing/ComponentHost.svelte';
import { browsing } from '$lib/stores/browsing.svelte';
import { browseView } from '$lib/stores/browseView.svelte';
import { compare } from '$lib/stores/compare.svelte';
import { selection } from '$lib/stores/selection.svelte';
import { ui } from '$lib/stores/ui.svelte';
import type { AssetSummary } from '$lib/types/album';
import type { ExifInfo } from '$lib/types/asset';

vi.mock('$app/navigation', () => ({
  afterNavigate: (): void => {},
  goto: vi.fn()
}));

vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/albums/a1'), params: {} }
}));

type LoupeProps = ComponentProps<typeof Loupe>;

const Host: Component<{ component: Component<LoupeProps>; props: LoupeProps }> = ComponentHost;

function asset(id: string): AssetSummary {
  return {
    id,
    originalFileName: `${id}.jpg`,
    type: 'IMAGE',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: null,
    tags: []
  };
}

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function render(): void {
  component = mount(Host, { target, props: { component: Loupe, props: {} } });
  flushSync();
}

function press(key: string): void {
  document.body.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true }));
  flushSync();
}

function query(selector: string): HTMLElement | null {
  return target.querySelector<HTMLElement>(selector);
}

beforeEach(() => {
  vi.stubGlobal(
    'fetch',
    vi.fn(() => Promise.reject(new Error('offline')))
  );
  browsing.set([asset('a'), asset('b'), asset('c')]);
  browseView.openLoupe('b');
  compare.exit();
  selection.clear();
  ui.fullscreen = false;
  ui.loupeFilmstripCollapsed = false;
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
  browseView.closeLoupe();
});

describe('Loupe', () => {
  it('shows the focused filename', () => {
    render();
    expect(query('nav[aria-label="Loupe toolbar"] h2')?.textContent).toBe('b.jpg');
  });

  it('toggles selection of the focused photo with the keyboard', () => {
    render();
    press('s');
    expect([...selection.selected]).toEqual(['b']);
    expect(query('[aria-label^="Deselect photo"]')).not.toBeNull();
    press('s');
    expect(selection.count).toBe(0);
    expect(query('[aria-label^="Select photo"]')).not.toBeNull();
  });

  it('shows the selection count and clears it from the toolbar', () => {
    render();
    press('s');
    const counter = query('nav[aria-label="Loupe toolbar"] [aria-live="polite"]');
    expect(counter?.textContent?.trim()).toBe('1 selected');
    counter?.closest('button')?.click();
    flushSync();
    expect(selection.count).toBe(0);
    expect(query('nav[aria-label="Loupe toolbar"] [aria-live="polite"]')).toBeNull();
  });

  it('marks selected thumbnails in the filmstrip', () => {
    render();
    press('s');
    expect(query('[data-testid="filmstrip-scroll"] [aria-label="Selected"]')).not.toBeNull();
  });

  it('moves to the next photo with the arrow keys', () => {
    render();
    press('ArrowRight');
    expect(browseView.loupeId).toBe('c');
    expect(query('nav[aria-label="Loupe toolbar"] h2')?.textContent).toBe('c.jpg');
  });

  it('describes the photo with the editor EXIF rows and closes the info panel', () => {
    const taken = '2024-05-06T07:08:09Z';
    browsing.patch('b', {
      exifInfo: { dateTimeOriginal: taken, fileSizeInByte: 2 * 1024 * 1024 } as ExifInfo
    });
    if (!browseView.loupeInfoOpen) browseView.toggleLoupeInfo();
    render();

    const info = query('section[aria-label="Photo info"]');
    expect(info?.textContent).toContain('b.jpg');
    expect(info?.textContent).toContain(new Date(taken).toLocaleString());
    expect(info?.textContent).toContain('2.0 MB');

    query('button[aria-label="Close info"]')?.click();
    flushSync();
    expect(query('section[aria-label="Photo info"]')).toBeNull();
  });
});
