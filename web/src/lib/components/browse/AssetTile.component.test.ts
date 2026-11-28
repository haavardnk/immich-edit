import type { TileFacts } from '#lib/browse/tileContent.ts';
import {
  DEFAULT_TILE_OVERLAY,
  withIcon,
  type TileInfo,
  type TileOverlay
} from '#lib/browse/tileOverlay.ts';
import { album } from '#lib/testing/albums.ts';
import { asset } from '#lib/testing/assets.ts';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import type { AssetSummary } from '#lib/types/album.ts';
import type { ExifInfo } from '#lib/types/asset.ts';
import { captureTime, fmtDay } from '#lib/utils/exif.ts';
import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import AssetTile from './AssetTile.svelte';

vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/photos'), params: {} } }));

type TileProps = ComponentProps<typeof AssetTile>;

const Host: Component<{ component: Component<TileProps>; props: TileProps }> = ComponentHost;

const NO_FACTS: TileFacts = { edited: false, albums: null, tags: null };

const RATED = asset('a', {
  originalFileName: 'DSCF4120.RAF',
  fileCreatedAt: '2026-10-31T09:00:00.000Z',
  exifInfo: { rating: 3 } as ExifInfo
});

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function render(
  info: TileInfo,
  options: {
    item?: AssetSummary;
    overlay?: TileOverlay;
    facts?: TileFacts;
    width?: number;
    height?: number;
    rate?: boolean;
  } = {}
): void {
  if (component) void unmount(component);
  component = mount(Host, {
    target,
    props: {
      component: AssetTile,
      props: {
        asset: options.item ?? RATED,
        info,
        overlay: options.overlay ?? DEFAULT_TILE_OVERLAY,
        facts: options.facts ?? NO_FACTS,
        width: options.width ?? 220,
        height: options.height ?? 150,
        onRate: options.rate ? () => {} : undefined
      }
    }
  });
  flushSync();
}

function band(): HTMLElement | null {
  return target.querySelector<HTMLElement>('[data-testid="tile-info"]');
}

function cluster(): HTMLElement | null {
  return target.querySelector<HTMLElement>('[data-testid="tile-cluster"]');
}

function named(name: string): HTMLElement | null {
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
});

describe('AssetTile overlay', () => {
  it('pins the icon cluster in icons mode and reveals the name on hover', () => {
    render('icons');

    expect(band()?.className).toContain('opacity-100');
    expect(cluster()?.querySelector('[aria-label="3 stars"]')).not.toBeNull();
    const name = target.querySelector('[data-testid="tile-name"]');
    expect(name?.textContent).toBe('DSCF4120.RAF');
    expect(name?.className).toContain('hidden group-hover:block');
  });

  it('keeps an empty icons band hover-only', () => {
    render('icons', { item: asset('b') });

    expect(band()?.className).toContain('opacity-0 group-hover:opacity-100');
    expect(cluster()).toBeNull();
  });

  it('shows as many detail lines as the height fits in full mode', () => {
    render('full', { height: 150 });

    const lines = [...(band()?.querySelectorAll('.truncate') ?? [])].map(
      (line) => line.textContent
    );
    expect(lines).toEqual(['DSCF4120.RAF', fmtDay(captureTime(RATED))]);
    expect(band()?.className).toContain('opacity-100');
  });

  it('drops every detail line on low tiles', () => {
    render('full', { height: 72, width: 100 });

    expect(target.querySelector('[data-testid="tile-name"]')).toBeNull();
    expect(band()?.className).toContain('pb-1');
  });

  it('pins the stars in full mode and takes the rating out of the cluster', () => {
    render('full', { height: 150, rate: true });

    const stars = target.querySelector('[role="radiogroup"]')?.closest('span.pointer-events-auto');
    expect(stars?.className).toMatch(/(^|\s)flex(\s|$)/);
    expect(cluster()?.querySelector('[aria-label="3 stars"]') ?? null).toBeNull();
  });

  it('swaps the rating badge for hover stars outside full mode', () => {
    render('icons', { rate: true });

    expect(cluster()?.querySelector('[aria-label="3 stars"]')?.className).toContain(
      'group-hover:hidden'
    );
  });

  it('keeps the top marks hover-only in hover only mode', () => {
    render('none', { facts: { ...NO_FACTS, edited: true } });

    expect(named('Edited')?.parentElement?.className).toContain('opacity-0');
    render('icons', { facts: { ...NO_FACTS, edited: true } });
    expect(named('Edited')?.parentElement?.className).not.toContain('opacity-0');
  });

  it('counts albums only when the tile has room', () => {
    const overlay = withIcon(DEFAULT_TILE_OVERLAY, 'albums', true);
    const facts = { ...NO_FACTS, albums: [album('x'), album('y')] };
    render('icons', { overlay, facts });

    expect(named('2 albums')?.textContent?.trim()).toBe('2');
    render('icons', { overlay, facts, width: 80, height: 72 });
    expect(named('2 albums')?.textContent?.trim()).toBe('');
  });

  it('stacks the last line over the cluster on narrow tiles', () => {
    render('full', { width: 100, height: 150 });

    expect(cluster()?.parentElement?.className).toContain('flex-col');
    expect(cluster()?.previousElementSibling?.className).not.toContain('max-w-[62%]');
  });

  it('gives the last line the whole row when no icon shares it', () => {
    render('full', { item: { ...RATED, exifInfo: null }, width: 200, height: 150 });

    expect(cluster()).toBeNull();
    const lines = band()?.querySelectorAll('.truncate') ?? [];
    expect(lines[lines.length - 1]?.className).not.toContain('max-w-[62%]');
  });

  it('keeps the last line whole up to 62% and lets the cluster drop icons', () => {
    render('full', { width: 200, height: 150 });

    expect(cluster()?.previousElementSibling?.className).toContain('max-w-[62%] shrink-0');
    expect(cluster()?.className).not.toContain('max-w-[62%]');
  });
});
