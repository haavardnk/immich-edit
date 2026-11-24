import { expect, type Page } from '@playwright/test';
import { installMocks, numberedAssets, type MockMapMarker } from './helpers';
import type { PlacesMock } from './placesMocks';

export const ASSETS = numberedAssets(5).map((asset, index) => ({
  ...asset,
  isFavorite: index === 0
}));

export const MARKERS: MockMapMarker[] = [
  { id: ASSETS[0]!.id, lat: 60.3975, lon: 5.324, city: 'Bergen' },
  { id: ASSETS[1]!.id, lat: 60.3913, lon: 5.3221, city: 'Bergen' },
  { id: ASSETS[2]!.id, lat: 60.6289, lon: 6.4153, city: 'Voss' },
  { id: ASSETS[3]!.id, lat: 59.9139, lon: 10.7522, city: 'Oslo' }
];

export const BRYGGEN = {
  name: 'Bryggen',
  detail: 'Bergen, Vestland, Norway',
  lat: 60.3975,
  lon: 5.324
};

export const GEILO = { name: 'Geilo', detail: 'Hol, Buskerud, Norway', lat: 60.534, lon: 8.206 };

export const BERGEN = { city: 'Bergen', state: 'Vestland', country: 'Norway' };

export type LocationWrite = { ids: string[]; latitude: number; longitude: number };

export async function openMap(page: Page): Promise<void> {
  await installMocks(page, { assets: ASSETS, mapMarkers: MARKERS });
  await page.goto('/map');
  await expect(page.locator('[data-map-cluster], [data-map-photo]').first()).toBeVisible();
}

export const PHOTOS = '[data-map-sheet], aside[aria-label="Photos in view"]:not([aria-hidden])';

export async function openTray(page: Page): Promise<void> {
  const pill = page.locator('[data-map-pill]');
  const toggle = page.getByRole('button', { name: 'Photos panel' });
  await expect(page.locator(`[data-map-pill], ${PHOTOS}`).or(toggle).first()).toBeVisible();
  if (await pill.isVisible()) await pill.tap();
  else if ((await toggle.isVisible()) && (await toggle.getAttribute('aria-pressed')) === 'false') {
    await toggle.tap();
    await page.waitForTimeout(600);
  }
  await expect(page.locator(PHOTOS).first()).toBeVisible();
}

export async function chooseOnMap(page: Page): Promise<void> {
  await page
    .getByRole('group', { name: 'Location' })
    .getByRole('button', { name: 'Choose on map…' })
    .click();
}

export async function confirmDrop(page: Page, verb: 'Move' | 'Place'): Promise<void> {
  const card = page.locator('[data-drop-confirm]');
  await expect(card).toBeVisible();
  await expect(card.getByRole('button', { name: verb, exact: true })).toBeFocused();
  await card.getByRole('button', { name: verb, exact: true }).click();
  await expect(card).toHaveCount(0);
}

export function markers(page: Page) {
  return page.locator('[data-map-cluster], [data-map-photo]');
}

export const PLACED = numberedAssets(4).map((asset, index) => {
  const spot = index < 2 ? [60.3975, 5.324, 'Bergen'] : [59.9139, 10.7522, 'Oslo'];
  return index % 2 === 0
    ? {
        ...asset,
        exifInfo: { latitude: spot[0], longitude: spot[1], city: spot[2], country: 'Norway' }
      }
    : asset;
}) as ReturnType<typeof numberedAssets>;

export const PLACED_MARKERS: MockMapMarker[] = [
  { id: PLACED[0]!.id, lat: 60.3975, lon: 5.324, city: 'Bergen' },
  { id: PLACED[2]!.id, lat: 59.9139, lon: 10.7522, city: 'Oslo' }
];

export async function openPlacing(
  page: Page,
  writes: LocationWrite[],
  places?: PlacesMock
): Promise<void> {
  await installMocks(page, {
    assets: PLACED,
    mapMarkers: PLACED_MARKERS,
    places,
    placeName: BERGEN,
    metadataPushConsented: true,
    onLocation: (body) => writes.push(body)
  });
  await page.goto('/map');
  await expect(markers(page)).toHaveCount(2);
}

export const STACK = numberedAssets(3);

export const STACK_MARKERS: MockMapMarker[] = [
  { id: STACK[0]!.id, lat: 60.3975, lon: 5.324, city: 'Bergen' },
  { id: STACK[1]!.id, lat: 60.3975, lon: 5.324, city: 'Bergen' },
  { id: STACK[2]!.id, lat: 59.9139, lon: 10.7522, city: 'Oslo' }
];

export async function openStack(page: Page, writes: LocationWrite[] = []): Promise<void> {
  await installMocks(page, {
    assets: STACK,
    mapMarkers: STACK_MARKERS,
    placeName: BERGEN,
    metadataPushConsented: true,
    onLocation: (body) => writes.push(body)
  });
  await page.goto('/map');
  await expect(markers(page)).toHaveCount(2);
}

export function stack(page: Page) {
  return page.locator(`[data-map-cluster="2"], [data-map-photo="${STACK[0]!.id}"]`);
}
