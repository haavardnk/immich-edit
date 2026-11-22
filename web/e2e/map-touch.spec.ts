import { expect, test } from '@playwright/test';
import { bulkBar } from './browse';
import { installMocks } from './helpers';
import { drag, longPressUntil, touchDown, touchMoveTo, touchUp } from './touch';
import { selectTwo } from './touchShell';
import {
  ASSETS,
  MARKERS,
  BERGEN,
  type LocationWrite,
  openMap,
  openTray,
  confirmDrop,
  markers,
  PLACED,
  openPlacing,
  STACK,
  STACK_MARKERS,
  openStack,
  stack
} from './mapFixtures';

test.describe('map on touch', { tag: '@touch' }, () => {
  test('the map keeps its photos behind a pill on phones and in a panel on tablets', async ({
    page
  }) => {
    await openMap(page);
    const area = (await page.getByRole('region', { name: 'Map of your photos' }).boundingBox())!;
    expect(area.height).toBeGreaterThan(300);
    const compact = (page.viewportSize()?.width ?? 0) < 700;
    if (compact) {
      const pill = page.locator('[data-map-pill]');
      await expect(pill).toContainText('4 photos in view');
      await expect(page.locator('[data-map-sheet]')).toHaveCount(0);
      await pill.tap();
      await expect(page.locator('[data-map-sheet]')).toContainText('4 photos');
      await page.getByRole('button', { name: 'Close photos' }).tap();
      await expect(pill).toBeVisible();
    } else {
      const panel = page.getByRole('complementary', { name: 'Photos in view' });
      await expect(panel).toHaveCount(0);
      await page.getByRole('button', { name: 'Photos panel' }).tap();
      await expect(panel).toContainText(/photos? in view/);
    }
  });

  test('a long press on the map selects photos and offers Location', async ({ page }) => {
    await openMap(page);
    await openTray(page);
    const tile = page.locator('[role="group"][data-name="IMG_0001.ARW"]');
    await expect(tile).toBeVisible();
    await longPressUntil(page, tile, page.getByText('1 selected'));
    await bulkBar(page).getByRole('button', { name: 'More', exact: true }).tap();
    await expect(
      page.getByRole('dialog', { name: 'More actions' }).getByRole('button', { name: 'Location' })
    ).toBeVisible();
  });

  test('a long press on a photo marker drags it to a new spot', async ({ page }) => {
    const writes: LocationWrite[] = [];
    await openPlacing(page, writes);
    const oslo = page.locator(`[data-map-photo="${PLACED[2]!.id}"]`);
    const box = (await oslo.boundingBox())!;
    const from = { x: box.x + box.width / 2, y: box.y + box.height / 2 };
    await touchDown(page, from);
    await page.waitForTimeout(650);
    for (let step = 1; step <= 8; step += 1) {
      await touchMoveTo(page, { x: from.x - step * 12, y: from.y - step * 6 });
    }
    await touchUp(page);
    await confirmDrop(page, 'Move');
    await expect.poll(() => writes.length).toBe(1);
    expect(writes[0]!.ids).toEqual([PLACED[2]!.id]);
  });

  test('a double tap on a photo marks it without zooming the map', async ({ page }) => {
    await openStack(page);
    await stack(page).tap();
    await expect(page.locator(`[data-map-photo="${STACK[0]!.id}"]`)).toBeVisible();
    await page.waitForTimeout(900);
    const photo = page.locator(`[data-map-photo="${STACK[0]!.id}"]`);
    const before = (await photo.boundingBox())!;
    await photo.tap();
    await photo.tap();
    await expect(photo).toHaveAttribute('data-marked', 'all');
    await expect(page.getByText('2 selected')).toBeVisible();
    await page.waitForTimeout(600);
    const after = (await photo.boundingBox())!;
    expect(Math.abs(after.x - before.x)).toBeLessThan(1);
    expect(Math.abs(after.y - before.y)).toBeLessThan(1);
    expect(Math.abs(after.width - before.width)).toBeLessThan(1);
  });

  test('starting a selection from a shared map link keeps the view', async ({ page }) => {
    await installMocks(page, { assets: STACK, mapMarkers: STACK_MARKERS });
    await page.goto('/map?at=60.39750,5.32400,6.00');
    await stack(page).tap();
    await page.waitForTimeout(1500);
    const photo = page.locator(`[data-map-photo="${STACK[0]!.id}"]`);
    await expect(photo).toBeVisible();
    const before = (await photo.boundingBox())!;
    await photo.tap();
    await photo.tap();
    await expect(page.getByText('2 selected')).toBeVisible();
    await page.waitForTimeout(1500);
    const after = (await photo.boundingBox())!;
    expect(Math.abs(after.x - before.x)).toBeLessThan(1);
    expect(Math.abs(after.y - before.y)).toBeLessThan(1);
    expect(new URL(page.url()).searchParams.get('at')).toMatch(/,12\.00$/);
  });

  test('opening a photo from the sheet hides the sheet until the loupe closes', async ({
    page
  }) => {
    test.skip((page.viewportSize()?.width ?? 0) >= 700, 'wide layouts use the side panel');
    await openStack(page);
    const sheet = page.locator('[data-map-sheet]');
    await openTray(page);
    const tile = sheet.locator('[role="group"][data-name] > a').first();
    await expect(tile).toBeVisible();
    await tile.tap({ position: { x: 8, y: 8 } });
    await expect(page.getByRole('button', { name: 'Back (Esc)' })).toBeVisible();
    await expect(sheet).toHaveCount(0);
    await page.getByRole('button', { name: 'Back (Esc)' }).tap();
    await expect(sheet).toBeVisible();
  });

  test('a long press in the tray drags a photo out onto the map', async ({ page }) => {
    test.skip((page.viewportSize()?.width ?? 0) >= 700, 'wide layouts drag from the side panel');
    const writes: LocationWrite[] = [];
    await openPlacing(page, writes);
    await openTray(page);
    await page.getByRole('radio', { name: 'No location' }).tap();
    const sheet = page.locator('[data-map-sheet]');
    const tile = sheet.locator('[role="group"][data-name="IMG_0002.ARW"]');
    await expect(tile).toBeVisible();
    await page.waitForTimeout(400);
    const box = (await tile.boundingBox())!;
    const from = { x: box.x + box.width / 2, y: box.y + box.height / 2 };
    await touchDown(page, from);
    await page.waitForTimeout(700);
    const top = (await sheet.boundingBox())!.y;
    for (let y = from.y; y > top + 20; y -= 30) await touchMoveTo(page, { x: from.x, y });
    await touchMoveTo(page, { x: from.x, y: top + 20 });
    const area = (await page.getByRole('region', { name: 'Map of your photos' }).boundingBox())!;
    for (let step = 1; step <= 6; step += 1) {
      await touchMoveTo(page, { x: from.x, y: top + 20 + ((area.y + 260 - top - 20) * step) / 6 });
    }
    await expect(
      page.getByRole('status').filter({ hasText: 'Drop to place 1 photo' })
    ).toBeVisible();
    await expect(page.locator('[data-drop-pin]')).toBeVisible();
    await expect(page.locator('[data-map-pill]')).toHaveCount(0);
    await touchUp(page);
    await confirmDrop(page, 'Place');
    await expect.poll(() => writes.length).toBe(1);
    expect(writes[0]!.ids).toEqual([PLACED[1]!.id]);
    await expect(page.locator('[data-drop-pin]')).toHaveCount(0);
    await expect(page.locator('[data-map-pill]')).toBeVisible();
  });

  test('the pill and the open sheet sit above the bottom bars', async ({ page }) => {
    test.skip((page.viewportSize()?.width ?? 0) >= 700, 'wide layouts use the side panel');
    await openStack(page);
    const pill = page.locator('[data-map-pill]');
    const area = (await page.getByRole('region', { name: 'Map of your photos' }).boundingBox())!;
    const idle = (await pill.boundingBox())!;
    expect(idle.y + idle.height).toBeLessThan(area.y + area.height);
    const photo = page.locator(`[data-map-photo="${STACK[2]!.id}"]`);
    await photo.tap();
    await expect(page.locator('[data-map-sheet]')).toBeVisible();
    await page.waitForTimeout(600);
    await photo.tap();
    await photo.tap();
    const bar = bulkBar(page);
    await expect(bar).toBeVisible();
    await expect(page.getByRole('radio', { name: 'In view' })).toBeVisible();
    const tray = (await page.locator('[data-map-sheet]').boundingBox())!;
    const dock = (await bar.boundingBox())!;
    expect(tray.y + tray.height).toBeLessThanOrEqual(dock.y + 1);
    await page.getByRole('button', { name: 'Show all', exact: true }).tap();
    await expect(page.locator('[data-map-sheet]')).toContainText('in view');
  });

  test('a long press on a group drags all its photos', async ({ page }) => {
    const writes: LocationWrite[] = [];
    await openStack(page, writes);
    const box = (await stack(page).boundingBox())!;
    const from = { x: box.x + box.width / 2, y: box.y + box.height / 2 };
    await touchDown(page, from);
    await page.waitForTimeout(650);
    for (let step = 1; step <= 8; step += 1) {
      await touchMoveTo(page, { x: from.x + step * 12, y: from.y - step * 6 });
    }
    await touchUp(page);
    await confirmDrop(page, 'Move');
    await expect.poll(() => writes.length).toBe(1);
    expect(writes[0]!.ids).toHaveLength(2);
  });

  test('a quick swipe from a photo marker pans the map instead', async ({ page }) => {
    const writes: LocationWrite[] = [];
    await openPlacing(page, writes);
    const oslo = page.locator(`[data-map-photo="${PLACED[2]!.id}"]`);
    const box = (await oslo.boundingBox())!;
    const from = { x: box.x + box.width / 2, y: box.y + box.height / 2 };
    await drag(page, from, { x: from.x - 80, y: from.y }, { settleMs: 50 });
    await page.waitForTimeout(800);
    expect(writes).toEqual([]);
  });

  test('Back on the map returns to Library past earlier map views', async ({ page }) => {
    test.skip((page.viewportSize()?.width ?? 0) >= 700, 'wide layouts have the sidebar, not Back');
    await installMocks(page, { assets: STACK, mapMarkers: STACK_MARKERS });
    await page.goto('/library');
    await page.getByRole('link', { name: /^Map/ }).filter({ visible: true }).first().tap();
    await expect(page).toHaveURL(/\/map/);
    await page.goto('/map?at=60.39750,5.32400,8.00');
    await expect(markers(page).first()).toBeVisible();
    await page.getByRole('button', { name: 'Back', exact: true }).tap();
    await expect(page).toHaveURL(/\/library$/);
  });

  test('leaving the map and coming back keeps its view and tab', async ({ page }) => {
    await openStack(page);
    await stack(page).tap();
    await page.waitForTimeout(1500);
    const at = new URL(page.url()).searchParams.get('at');
    await page.getByRole('radio', { name: 'No location' }).tap();
    const wide = (page.viewportSize()?.width ?? 0) >= 700;
    await page
      .getByRole('link', { name: /^Photos/ })
      .filter({ visible: true })
      .first()
      .tap();
    await expect(page).toHaveURL(/\/photos/);
    if (!wide) {
      await page
        .getByRole('link', { name: /^Library/ })
        .filter({ visible: true })
        .first()
        .tap();
      await page.waitForURL(/\/(library|map)/);
    }
    if (!new URL(page.url()).pathname.startsWith('/map')) {
      await page.getByRole('link', { name: /^Map/ }).filter({ visible: true }).first().tap();
    }
    await expect(page).toHaveURL(/\/map/);
    await openTray(page);
    await expect(page.getByRole('radio', { name: 'No location' })).toBeChecked();
    await expect.poll(() => new URL(page.url()).searchParams.get('at')).toBe(at);
  });

  test('Back to Library and into the map again starts fresh', async ({ page }) => {
    test.skip((page.viewportSize()?.width ?? 0) >= 700, 'wide layouts have the sidebar, not Back');
    await installMocks(page, { assets: STACK, mapMarkers: STACK_MARKERS });
    await page.goto('/library');
    await page.getByRole('link', { name: /^Map/ }).filter({ visible: true }).first().tap();
    await expect(stack(page)).toBeVisible();
    await page.waitForTimeout(800);
    const fitted = new URL(page.url()).searchParams.get('at');
    await stack(page).tap();
    await page.waitForTimeout(1500);
    expect(new URL(page.url()).searchParams.get('at')).not.toBe(fitted);
    await page.getByRole('button', { name: 'Back', exact: true }).tap();
    await expect(page).toHaveURL(/\/library$/);
    await page.getByRole('link', { name: /^Map/ }).filter({ visible: true }).first().tap();
    await expect(page.locator('[data-map-pill]')).toContainText('3 photos in view');
    await expect.poll(() => new URL(page.url()).searchParams.get('at')).toBe(fitted);
  });

  test('Library lists the map', async ({ page }) => {
    await installMocks(page, { assets: ASSETS, mapMarkers: MARKERS });
    await page.goto('/library');
    await page.getByRole('link', { name: /^Map/ }).filter({ visible: true }).first().tap();
    await expect(page).toHaveURL(/\/map/);
    await expect(markers(page).first()).toBeVisible();
  });

  test('the More sheet sets the location of a selection', async ({ page }) => {
    const writes: LocationWrite[] = [];
    await installMocks(page, {
      assets: ASSETS,
      placeName: BERGEN,
      metadataPushConsented: true,
      onLocation: (body) => writes.push(body)
    });
    await page.goto('/photos');
    await selectTwo(page);
    await bulkBar(page).getByRole('button', { name: 'More', exact: true }).tap();
    await page
      .getByRole('dialog', { name: 'More actions' })
      .getByRole('button', { name: 'Location' })
      .tap();
    const dialog = page.getByRole('dialog', { name: 'Set location' });
    await expect(dialog).toContainText('Move the map under the pin');
    const map = dialog.getByRole('region', { name: 'Location map' });
    await expect(map.locator('canvas')).toBeVisible();
    const box = (await map.boundingBox())!;
    const centre = { x: box.x + box.width / 2, y: box.y + box.height / 2 };
    await drag(page, centre, { x: centre.x - 60, y: centre.y - 40 }, { settleMs: 50 });
    await expect(dialog.getByRole('button', { name: 'Save' })).toBeEnabled();
    await dialog.getByRole('button', { name: 'Save' }).tap();
    await expect(dialog).toHaveCount(0);
    expect(writes).toHaveLength(1);
    expect(writes[0]!.ids).toHaveLength(2);
  });
});
