import { expect, test } from '@playwright/test';
import { bulkBar, pick } from './browse';
import { installMocks } from './helpers';
import {
  ASSETS,
  MARKERS,
  BRYGGEN,
  GEILO,
  BERGEN,
  type LocationWrite,
  chooseOnMap,
  confirmDrop,
  PLACED,
  openPlacing,
  STACK,
  openStack,
  stack
} from './mapFixtures';

test('the location picker searches a place and saves it', async ({ page }) => {
  const writes: LocationWrite[] = [];
  await installMocks(page, {
    assets: ASSETS,
    mapSearch: true,
    mapPlaces: [BRYGGEN],
    placeName: BERGEN,
    metadataPushConsented: true,
    onLocation: (body) => writes.push(body)
  });
  await page.goto('/photos');
  await pick(page, 'IMG_0005.ARW');
  await pick(page, 'IMG_0004.ARW');
  await bulkBar(page).getByRole('button', { name: 'Location', exact: true }).click();
  await chooseOnMap(page);

  const dialog = page.getByRole('dialog', { name: 'Set location' });
  await expect(dialog).toContainText('2 photos');
  await expect(dialog.getByRole('button', { name: 'Save' })).toBeDisabled();
  await dialog.getByRole('textbox', { name: 'Search for a place' }).fill('bryg');
  await dialog
    .getByRole('list', { name: 'Places' })
    .getByRole('button', { name: /Bryggen/ })
    .click();
  await expect(
    dialog.getByRole('status').filter({ hasText: 'Bergen, Vestland, Norway' })
  ).toBeVisible();
  await dialog.getByRole('button', { name: 'Save' }).click();

  await expect(dialog).toHaveCount(0);
  expect(writes).toEqual(
    [{ ids: [ASSETS[4]!.id, ASSETS[3]!.id].sort(), latitude: 60.3975, longitude: 5.324 }].map(
      (write) => ({ ...write, ids: expect.arrayContaining(write.ids) as unknown as string[] })
    )
  );
  await expect(page.getByText('Location set for 2 photos')).toBeVisible();
});

test('a selection on the map moves to its new location', async ({ page }) => {
  await installMocks(page, {
    assets: ASSETS,
    mapMarkers: [MARKERS[0]!, MARKERS[3]!],
    mapSearch: true,
    mapPlaces: [GEILO],
    placeName: { city: 'Geilo', state: 'Buskerud', country: 'Norway' },
    metadataPushConsented: true
  });
  await page.goto('/map');
  const panel = page.getByRole('complementary', { name: 'Photos in view' });
  await expect(panel).toContainText('Bergen and Oslo');
  const oslo = page.locator(`[data-map-photo="${ASSETS[3]!.id}"]`);
  await expect(oslo).toBeVisible();
  const before = (await oslo.boundingBox())!;
  const tile = panel.locator('[role="group"][data-name="IMG_0004.ARW"]');
  await tile.hover();
  await tile.getByRole('button', { name: 'Select', exact: true }).click();
  await bulkBar(page).getByRole('button', { name: 'Location', exact: true }).click();
  await chooseOnMap(page);
  const dialog = page.getByRole('dialog', { name: /^(Set|Edit) location$/ });
  await dialog.getByRole('textbox', { name: 'Search for a place' }).fill('geil');
  await dialog.getByRole('list', { name: 'Places' }).getByRole('button', { name: /Geilo/ }).click();
  await expect(dialog.getByRole('button', { name: 'Save' })).toBeEnabled();
  await dialog.getByRole('button', { name: 'Save' }).click();
  await expect(dialog).toHaveCount(0);
  await expect(panel).not.toContainText('Oslo');
  await expect(panel).toContainText('Geilo');
  await expect(oslo).toHaveCount(1);
  await expect
    .poll(async () => ((await oslo.boundingBox())?.x ?? before.x) - before.x)
    .toBeLessThan(-100);
});

test('photos without a location are dragged from the panel onto the map', async ({ page }) => {
  const writes: LocationWrite[] = [];
  await openPlacing(page, writes);
  const panel = page.getByRole('complementary', { name: 'Photos in view' });
  await panel.getByRole('radio', { name: 'No location' }).click();
  await expect(panel).toContainText('2 photos without location');
  await expect(panel).toContainText('Drag photos onto the map to place them');
  const tile = panel.locator('[role="group"][data-name="IMG_0002.ARW"]');
  const map = page.getByRole('region', { name: 'Map of your photos' });
  const box = (await map.boundingBox())!;
  await tile.hover();
  await page.mouse.down();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2, { steps: 8 });
  await expect(page.getByRole('status').filter({ hasText: 'Drop to place 1 photo' })).toBeVisible();
  await expect(page.locator('[data-drop-pin]')).toBeVisible();
  await page.mouse.up();

  await confirmDrop(page, 'Place');
  await expect.poll(() => writes.length).toBe(1);
  expect(writes[0]!.ids).toEqual([PLACED[1]!.id]);
  await expect(tile).toHaveCount(0);
  await expect(panel).toContainText('1 photo without location');
  await expect(page.locator('[data-drop-pin]')).toHaveCount(0);
});

test('dragging a selected photo places the whole selection', async ({ page }) => {
  const writes: LocationWrite[] = [];
  await openPlacing(page, writes);
  const panel = page.getByRole('complementary', { name: 'Photos in view' });
  await panel.getByRole('radio', { name: 'No location' }).click();
  for (const name of ['IMG_0002.ARW', 'IMG_0004.ARW']) {
    const tile = panel.locator(`[role="group"][data-name="${name}"]`);
    await tile.hover();
    await tile.getByRole('button', { name: 'Select', exact: true }).click();
    await expect(tile).toHaveAttribute('data-selected', 'true');
  }
  const map = page.getByRole('region', { name: 'Map of your photos' });
  const box = (await map.boundingBox())!;
  await panel.locator('[role="group"][data-name="IMG_0004.ARW"]').hover();
  await page.mouse.down();
  await page.mouse.move(box.x + box.width / 3, box.y + box.height / 2, { steps: 8 });
  await expect(
    page.getByRole('status').filter({ hasText: 'Drop to place 2 photos' })
  ).toBeVisible();
  await page.mouse.up();
  await confirmDrop(page, 'Place');
  await expect.poll(() => writes.length).toBe(1);
  expect([...writes[0]!.ids].sort()).toEqual([PLACED[1]!.id, PLACED[3]!.id]);
  await expect(panel).toContainText('Every photo matching these filters has a location.');
});

test('a photo marker is dragged to a new spot', async ({ page }) => {
  const writes: LocationWrite[] = [];
  await openPlacing(page, writes);
  const oslo = page.locator(`[data-map-photo="${PLACED[2]!.id}"]`);
  const start = (await oslo.boundingBox())!;
  const from = { x: start.x + start.width / 2, y: start.y + start.height / 2 };
  await page.mouse.move(from.x, from.y);
  await page.mouse.down();
  await page.waitForTimeout(350);
  await page.mouse.move(from.x - 200, from.y + 40, { steps: 10 });
  const pin = page.locator('[data-drag-pin]');
  await expect(pin).toBeVisible();
  const tip = (await pin.boundingBox())!;
  expect(Math.abs(tip.x + tip.width / 2 - (from.x - 200))).toBeLessThan(3);
  expect(Math.abs(tip.y + tip.height - (from.y + 40))).toBeLessThan(3);
  await page.mouse.up();
  await expect(pin).toHaveCount(0);

  await confirmDrop(page, 'Move');
  await expect.poll(() => writes.length).toBe(1);
  expect(writes[0]!.ids).toEqual([PLACED[2]!.id]);
  expect(writes[0]!.longitude).toBeLessThan(10.7522);
  await expect
    .poll(async () => ((await oslo.boundingBox())?.x ?? start.x) - start.x)
    .toBeLessThan(-150);
  await expect(page.getByRole('complementary', { name: 'Photos in view' })).toContainText(
    '2 photos in view'
  );
});

test('Escape puts a dragged marker back and a quick grab pans instead', async ({ page }) => {
  const writes: LocationWrite[] = [];
  await openStack(page, writes);
  const box = (await stack(page).boundingBox())!;
  const from = { x: box.x + box.width / 2, y: box.y + box.height / 2 };
  await page.mouse.move(from.x, from.y);
  await page.mouse.down();
  await page.waitForTimeout(350);
  await page.mouse.move(from.x + 200, from.y + 40, { steps: 8 });
  await expect(page.locator('[data-drag-pin]')).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(page.locator('[data-drag-pin]')).toHaveCount(0);
  await page.mouse.up();
  await page.waitForTimeout(600);
  expect(writes).toEqual([]);
  const back = (await stack(page).boundingBox())!;
  expect(Math.abs(back.x - box.x)).toBeLessThan(2);

  const at = new URL(page.url()).searchParams.get('at');
  await page.mouse.move(from.x, from.y);
  await page.mouse.down();
  await page.mouse.move(from.x + 150, from.y, { steps: 8 });
  await page.mouse.up();
  await page.waitForTimeout(800);
  expect(writes).toEqual([]);
  expect(new URL(page.url()).searchParams.get('at')).not.toBe(at);
});

test('cancelling the drop card leaves the photos where they were', async ({ page }) => {
  const writes: LocationWrite[] = [];
  await openStack(page, writes);
  const box = (await stack(page).boundingBox())!;
  const from = { x: box.x + box.width / 2, y: box.y + box.height / 2 };
  await page.mouse.move(from.x, from.y);
  await page.mouse.down();
  await page.waitForTimeout(350);
  await page.mouse.move(from.x + 200, from.y + 40, { steps: 8 });
  await page.mouse.up();
  const card = page.locator('[data-drop-confirm]');
  await expect(card).toContainText('Move 2 photos here?');
  await expect(card).toContainText('Bergen');
  await page.keyboard.press('Escape');
  await expect(card).toHaveCount(0);
  await page.waitForTimeout(600);
  expect(writes).toEqual([]);
  const back = (await stack(page).boundingBox())!;
  expect(Math.abs(back.x - box.x)).toBeLessThan(2);
});

test('a group is dragged to a new spot as one', async ({ page }) => {
  const writes: LocationWrite[] = [];
  await openStack(page, writes);
  const box = (await stack(page).boundingBox())!;
  const from = { x: box.x + box.width / 2, y: box.y + box.height / 2 };
  await page.mouse.move(from.x, from.y);
  await page.mouse.down();
  await page.waitForTimeout(350);
  await page.mouse.move(from.x + 220, from.y + 30, { steps: 10 });
  const pin = page.locator('[data-drag-pin]');
  await expect(pin).toBeVisible();
  const tip = (await pin.boundingBox())!;
  expect(Math.abs(tip.y + tip.height - (from.y + 30))).toBeLessThan(3);
  await page.mouse.up();
  await confirmDrop(page, 'Move');
  await expect.poll(() => writes.length).toBe(1);
  expect([...writes[0]!.ids].sort()).toEqual([STACK[0]!.id, STACK[1]!.id]);
});
