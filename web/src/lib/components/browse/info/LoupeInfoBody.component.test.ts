import { assetAlbums } from '#lib/stores/assetAlbums.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { session } from '#lib/stores/session.svelte.ts';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { layout, layoutControls, settle, type ControlRef } from '#lib/testing/layoutParity.ts';
import { reactiveValue } from '#lib/testing/reactiveValue.svelte.ts';
import type { AlbumSummary, AssetSummary } from '#lib/types/album.ts';
import type { ExifInfo } from '#lib/types/asset.ts';
import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import LoupeInfoBody from './LoupeInfoBody.svelte';
import LoupeInfoPanel from './LoupeInfoPanel.svelte';
import LoupeInfoSheet from './LoupeInfoSheet.svelte';

const h = vi.hoisted(() => ({
  setDescription: vi.fn(() => Promise.resolve('saved')),
  albumsWithAsset: vi.fn<(id: string) => Promise<AlbumSummary[]>>()
}));

vi.mock('$app/navigation', () => ({ afterNavigate: (): void => {}, goto: vi.fn() }));

vi.mock('#lib/browse/assetMeta.ts', async (orig) => ({
  ...(await orig<typeof import('#lib/browse/assetMeta.ts')>()),
  setDescription: h.setDescription
}));

vi.mock('#lib/api/albums.ts', async (orig) => ({
  ...(await orig<typeof import('#lib/api/albums.ts')>()),
  albumsWithAsset: h.albumsWithAsset
}));

type BodyProps = ComponentProps<typeof LoupeInfoBody>;

const Host: Component<{ component: Component<BodyProps>; props: BodyProps }> = ComponentHost;

const button = (name: string): ControlRef => ({ role: 'button', name });

const photo: AssetSummary = {
  id: 'p1',
  originalFileName: 'IMG_4120.ARW',
  originalPath: '/library/2026/07/IMG_4120.ARW',
  type: 'IMAGE',
  fileCreatedAt: null,
  updatedAt: null,
  checksum: null,
  isFavorite: false,
  exifInfo: {
    make: 'SONY',
    model: 'ILCE-7M4',
    fNumber: 2.8,
    exposureTime: '0.004',
    iso: 400,
    focalLength: 35,
    city: 'Reine',
    state: 'Nordland',
    country: 'Norway',
    latitude: 67.93,
    longitude: 13.09,
    description: 'Fjord at dusk'
  } as ExifInfo,
  tags: []
};

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function render(asset: AssetSummary = photo): void {
  component = mount(Host, { target, props: { component: LoupeInfoBody, props: { asset } } });
  flushSync();
}

function field(): HTMLTextAreaElement | null {
  return target.querySelector('textarea');
}

function type(text: string): HTMLTextAreaElement {
  const input = field();
  if (!input) throw new Error('no description field');
  input.focus();
  input.value = text;
  input.dispatchEvent(new Event('input', { bubbles: true }));
  flushSync();
  return input;
}

function key(input: HTMLElement, name: string): KeyboardEvent {
  const event = new KeyboardEvent('keydown', { key: name, bubbles: true, cancelable: true });
  input.dispatchEvent(event);
  flushSync();
  return event;
}

beforeEach(() => {
  h.setDescription.mockClear();
  h.albumsWithAsset.mockReset();
  h.albumsWithAsset.mockResolvedValue([
    {
      id: 'al1',
      albumName: 'Lofoten 2026',
      assetCount: 148,
      albumThumbnailAssetId: null,
      updatedAt: null
    }
  ]);
  vi.stubGlobal(
    'fetch',
    vi.fn(() => Promise.reject(new Error('offline')))
  );
  assetAlbums.forget([photo.id]);
  session.clear();
  browsing.set([photo]);
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
  vi.unstubAllGlobals();
});

describe('LoupeInfoBody', () => {
  it('shows the exposure strip, detail rows, place, albums and path', async () => {
    render();
    const text = target.textContent ?? '';
    for (const value of ['f/2.8', '1/250s', '400', '35mm', 'SONY ILCE-7M4', 'Reine']) {
      expect(text).toContain(value);
    }
    expect(text).toContain('Nordland, Norway');
    const map = target.querySelector<HTMLAnchorElement>(
      'a[href^="https://www.openstreetmap.org/"]'
    );
    expect(map?.textContent).toContain('Open in map');
    expect(map?.target).toBe('_blank');
    expect(map?.rel).toBe('noopener noreferrer');
    expect(text).toContain('/library/2026/07/IMG_4120.ARW');
    await vi.waitFor(() => expect(target.textContent).toContain('Lofoten 2026'));
    expect(h.albumsWithAsset).toHaveBeenCalledWith('p1');
  });

  it('saves the description on Enter', () => {
    render();
    const input = type('  Reine at dusk ');
    const event = key(input, 'Enter');
    expect(event.defaultPrevented).toBe(true);
    expect(h.setDescription).toHaveBeenCalledTimes(1);
    expect(h.setDescription).toHaveBeenCalledWith(
      expect.objectContaining({ id: 'p1' }),
      'Reine at dusk'
    );
  });

  it('commits the typed draft to its own photo when the panel steps to another', () => {
    const next: AssetSummary = {
      ...photo,
      id: 'p2',
      exifInfo: { ...photo.exifInfo, description: '' } as ExifInfo
    };
    browsing.set([photo, next]);
    const shown = reactiveValue<AssetSummary>(photo);
    component = mount(Host, {
      target,
      props: {
        component: LoupeInfoBody,
        props: {
          get asset(): AssetSummary {
            return shown.value;
          }
        }
      }
    });
    flushSync();
    type('Typed before the step');

    shown.value = next;
    flushSync();

    expect(h.setDescription).toHaveBeenCalledTimes(1);
    expect(h.setDescription).toHaveBeenCalledWith(
      expect.objectContaining({ id: 'p1' }),
      'Typed before the step'
    );
    expect(field()?.value).toBe('');
  });

  it('reverts on Escape without letting it reach the loupe', () => {
    const outside = vi.fn();
    window.addEventListener('keydown', outside);
    render();
    const input = type('Something else');
    key(input, 'Escape');
    window.removeEventListener('keydown', outside);
    expect(outside).not.toHaveBeenCalled();
    expect(field()?.value).toBe('Fjord at dusk');
    expect(h.setDescription).not.toHaveBeenCalled();
  });

  it('shows the description read-only on a photo shared by someone else', () => {
    render({ ...photo, ownerId: 'someone-else' });
    expect(field()).toBeNull();
    expect(target.textContent).toContain('Fjord at dusk');
  });
});

describe('LoupeInfoBody layout parity', () => {
  it('exposes the same controls in the docked panel and the sheet', async () => {
    const found = await layoutControls([
      layout('body', LoupeInfoBody, { asset: photo }),
      layout('panel', LoupeInfoPanel, { asset: photo }, { exclude: [button('Close info')] }),
      layout(
        'sheet',
        LoupeInfoSheet,
        { asset: photo },
        { exclude: [button('Close info'), button('Resize sheet')] }
      )
    ]);
    await settle();
    expect(found.body.map((control) => control.name)).toEqual(
      expect.arrayContaining([
        'Description',
        'Open in map',
        'Add to album',
        'Add tags',
        'Copy path'
      ])
    );
    expect(found.panel).toEqual(found.body);
    expect(found.sheet).toEqual(found.body);
  });
});
