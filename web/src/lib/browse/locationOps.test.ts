import { applyLocation, locationOf, planLocation } from '#lib/browse/locationOps.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { session } from '#lib/stores/session.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { asset } from '#lib/testing/assets.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const h = vi.hoisted(() => ({
  setAssetLocation: vi.fn((_ids: string[], _lat: number, _lon: number) => Promise.resolve()),
  consent: { value: true }
}));

vi.mock('#lib/api/map.ts', () => ({ setAssetLocation: h.setAssetLocation }));
vi.mock('#lib/stores/metadataConsent.svelte.ts', () => ({
  metadataConsent: { gate: () => Promise.resolve(h.consent.value) }
}));

const ME = 'user-me';
const A = '00000000-0000-4000-8000-00000000000a';
const BERGEN = {
  lat: 60.39,
  lon: 5.32,
  place: { city: 'Bergen', state: 'Vestland', country: 'Norway' }
};

function mine(
  id: string,
  exif: Partial<NonNullable<ReturnType<typeof asset>['exifInfo']>> | null = null
) {
  const base = asset(id);
  return {
    ...base,
    ownerId: ME,
    exifInfo: exif ? ({ ...base.exifInfo, ...exif } as typeof base.exifInfo) : base.exifInfo
  };
}

beforeEach(() => {
  h.setAssetLocation.mockReset();
  h.setAssetLocation.mockResolvedValue();
  h.consent.value = true;
  session.set({ id: ME, email: 'me@x', name: 'Me', is_admin: false, auth_kind: 'password' });
  toasts.items = [];
});

describe('location operations', () => {
  it('reads a location and splits owned photos from others', () => {
    expect(locationOf(null)).toBeNull();
    expect(locationOf(mine('a', { latitude: 1, longitude: 2, city: 'X' }).exifInfo)).toEqual({
      lat: 1,
      lon: 2,
      place: { city: 'X', state: null, country: null }
    });
    const other = { ...asset('b'), ownerId: 'someone' };
    expect(planLocation([mine('a'), other])).toEqual({ owned: [mine('a')], refused: 1 });
  });

  it('writes owned photos once, patches them and offers Undo', async () => {
    const a = mine(`${A}_1`);
    const b = mine('b', { latitude: 59.9, longitude: 10.7, city: 'Oslo' });
    browsing.set([a, b]);
    expect(await applyLocation([a, b], BERGEN)).toBe(true);
    expect(h.setAssetLocation).toHaveBeenCalledWith([A, 'b'], 60.39, 5.32);
    expect(browsing.find('b')?.exifInfo?.city).toBe('Bergen');
    const toast = toasts.items.at(-1);
    expect(toast?.message).toBe('Location set for 2 photos');
    toast?.action?.run();
    await vi.waitFor(() => expect(browsing.find('b')?.exifInfo?.city).toBe('Oslo'));
    expect(h.setAssetLocation).toHaveBeenLastCalledWith(['b'], 59.9, 10.7);
  });

  it('puts the old values back when Immich refuses', async () => {
    const a = mine('a', { latitude: 1, longitude: 1, city: 'Old' });
    browsing.set([a]);
    h.setAssetLocation.mockRejectedValueOnce(new Error('boom'));
    expect(await applyLocation([a], BERGEN)).toBe(false);
    expect(browsing.find('a')?.exifInfo?.city).toBe('Old');
  });

  it('writes nothing without consent or without owned photos', async () => {
    h.consent.value = false;
    expect(await applyLocation([mine('a')], BERGEN)).toBe(false);
    h.consent.value = true;
    expect(await applyLocation([{ ...asset('b'), ownerId: 'someone' }], BERGEN)).toBe(false);
    expect(h.setAssetLocation).not.toHaveBeenCalled();
  });
});
