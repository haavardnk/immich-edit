import {
  copyLocationOf,
  locationName,
  NO_LOCATION_COPIED,
  NO_LOCATION_TO_COPY,
  pasteLocationOnto,
  pasteLocationTo
} from '#lib/browse/locationCopy.ts';
import { locationClipboard } from '#lib/stores/locationClipboard.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { asset } from '#lib/testing/assets.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const h = vi.hoisted(() => ({
  applyLocation: vi.fn(() => Promise.resolve(true)),
  resolveAssets: vi.fn((ids: readonly string[]) => Promise.resolve(ids.map((id) => ({ id }))))
}));

vi.mock('#lib/browse/locationOps.ts', async (original) => ({
  ...(await original<typeof import('#lib/browse/locationOps.ts')>()),
  applyLocation: h.applyLocation,
  resolveAssets: h.resolveAssets
}));

const BERGEN = {
  lat: 60.39,
  lon: 5.32,
  place: { city: 'Bergen', state: 'Vestland', country: 'Norway' }
};

beforeEach(() => {
  h.applyLocation.mockClear();
  h.resolveAssets.mockClear();
  locationClipboard.copied = null;
  toasts.items = [];
});

describe('copy and paste a location', () => {
  it('names a location by its most specific place, else its coordinates', () => {
    expect(locationName(BERGEN)).toBe('Bergen');
    expect(
      locationName({ ...BERGEN, place: { city: null, state: 'Vestland', country: null } })
    ).toBe('Vestland');
    expect(locationName({ lat: -1, lon: -2, place: null })).toBe('1.0000° S, 2.0000° W');
  });

  it('copies a located photo and explains a photo without one', () => {
    copyLocationOf(asset('a'));
    expect(locationClipboard.copied).toBeNull();
    expect(toasts.items.at(-1)?.message).toBe(NO_LOCATION_TO_COPY);

    const exifInfo = {
      latitude: 60.39,
      longitude: 5.32,
      city: 'Bergen'
    } as AssetSummary['exifInfo'];
    copyLocationOf(asset('a', { exifInfo }));
    expect(locationClipboard.copied).toMatchObject({ lat: 60.39, lon: 5.32 });
    expect(toasts.items.at(-1)?.message).toBe('Copied Bergen');
  });

  it('refuses to paste before anything is copied, without fetching photos', async () => {
    expect(await pasteLocationTo(['a', 'b'])).toBe(false);
    expect(h.resolveAssets).not.toHaveBeenCalled();
    expect(h.applyLocation).not.toHaveBeenCalled();
    expect(toasts.items.at(-1)?.message).toBe(NO_LOCATION_COPIED);
  });

  it('pastes the copied location onto every photo', async () => {
    locationClipboard.copied = BERGEN;
    expect(await pasteLocationTo(['a', 'b'])).toBe(true);
    expect(h.resolveAssets).toHaveBeenCalledWith(['a', 'b']);
    expect(h.applyLocation).toHaveBeenCalledWith([{ id: 'a' }, { id: 'b' }], BERGEN);
    expect(await pasteLocationOnto([])).toBe(true);
    expect(await pasteLocationTo([])).toBe(false);
  });
});
