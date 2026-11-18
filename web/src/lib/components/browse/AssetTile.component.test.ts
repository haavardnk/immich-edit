import ComponentHost from '#lib/testing/ComponentHost.svelte';
import type { AssetSummary } from '#lib/types/album.ts';
import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import AssetTile from './AssetTile.svelte';

vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/photos?rating=3'), params: {} }
}));

type TileProps = ComponentProps<typeof AssetTile>;

const Host: Component<{ component: Component<TileProps>; props: TileProps }> = ComponentHost;

const ASSET: AssetSummary = {
  id: 'a1',
  originalFileName: 'IMG_0001.ARW',
  type: 'IMAGE',
  fileCreatedAt: '2024-01-01T00:00:00Z',
  updatedAt: '2024-01-01T00:00:00Z',
  checksum: 'c',
  isFavorite: false,
  exifInfo: null,
  tags: []
};

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function render(props: Partial<TileProps> = {}): void {
  component = mount(Host, {
    target,
    props: { component: AssetTile, props: { asset: ASSET, ...props } }
  });
  flushSync();
}

function tileLink(): HTMLAnchorElement {
  const link = target.querySelector<HTMLAnchorElement>('a[aria-label="IMG_0001.ARW"]');
  if (!link) throw new Error('no tile link');
  return link;
}

beforeEach(() => {
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
});

describe('AssetTile', () => {
  it('opens the loupe on a click and offers Edit on hover', () => {
    const onLoupe = vi.fn();
    const onActivate = vi.fn();
    render({ onLoupe, onActivate });
    expect(tileLink().getAttribute('href')).toBe('/assets/a1?from=%2Fphotos%3Frating%3D3');
    const edit = target.querySelector<HTMLAnchorElement>('a[aria-label="Edit"]');
    expect(edit?.getAttribute('href')).toBe(tileLink().getAttribute('href'));

    const click = new MouseEvent('click', { bubbles: true, cancelable: true });
    tileLink().dispatchEvent(click);
    expect(click.defaultPrevented).toBe(true);
    expect(onLoupe).toHaveBeenCalledOnce();
    expect(onActivate).toHaveBeenCalledOnce();
  });
});
