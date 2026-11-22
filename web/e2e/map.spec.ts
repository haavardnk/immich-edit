import { expect, test } from '@playwright/test';
import { bulkBar } from './browse';
import { installMocks } from './helpers';
import { ASSETS, MARKERS, openMap, markers, openStack, stack } from './mapFixtures';

test('the map lists the photos in view beside clustered markers', async ({ page }) => {
  await openMap(page);
  await expect(page.getByRole('link', { name: 'Map' })).toHaveAttribute('aria-current', 'page');
  const panel = page.getByRole('complementary', { name: 'Photos in view' });
  await expect(panel).toContainText('4 photos in view');
  await expect(panel.locator('[role="group"][data-name]')).toHaveCount(4);
  await expect(markers(page)).not.toHaveCount(0);

  await page.getByRole('button', { name: 'Favorites' }).click();
  await expect(panel).toContainText('1 photo');
  await expect(page).toHaveURL(/favorite=1/);
  await expect(panel.locator('[role="group"][data-name]')).toHaveCount(1);
});

test('the date filter stacks its fields and clears them', async ({ page }) => {
  await openMap(page);
  await page.getByRole('button', { name: 'Any date' }).click();
  const from = page.getByRole('group', { name: 'From' });
  const to = page.getByRole('group', { name: 'To' });
  const fromBox = (await from.boundingBox())!;
  const toBox = (await to.boundingBox())!;
  expect(toBox.y).toBeGreaterThan(fromBox.y + fromBox.height - 1);
  await from.locator('[data-segment="month"]').click();
  await page.keyboard.type('01012024');
  await expect(page).toHaveURL(/after=2024-01-01/);
  await expect(page.getByRole('button', { name: /^From / })).toBeVisible();
  await page.getByRole('button', { name: 'Clear dates' }).click();
  await expect(page).not.toHaveURL(/after=/);
  await expect(page.getByRole('button', { name: 'Any date' })).toBeVisible();
});

test('a photo in view opens the loupe and Back returns to the map', async ({ page }) => {
  await openMap(page);
  const panel = page.getByRole('complementary', { name: 'Photos in view' });
  await panel
    .locator('[role="group"][data-name] > a')
    .first()
    .click({ position: { x: 8, y: 8 } });
  await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();
  await page.getByRole('button', { name: /^Back/ }).click();
  await expect(page).toHaveURL(/\/map/);
});

test('the mini map in photo info leaves the loupe for the map', async ({ page }) => {
  const photo = {
    ...ASSETS[0]!,
    exifInfo: { city: 'Bergen', latitude: 60.39, longitude: 5.32 }
  };
  await installMocks(page, {
    assets: [photo],
    mapMarkers: [{ id: photo.id, lat: 60.39, lon: 5.32 }]
  });
  await page.goto('/photos');
  await page
    .locator('[role="group"][data-name] > a')
    .first()
    .click({ position: { x: 8, y: 8 } });
  await page.keyboard.press('i');
  await page.getByRole('link', { name: 'Open Bergen on the map' }).click();
  await expect(page).toHaveURL(/at=60\.39000(,|%2C)5\.32000/);
  await expect(page.getByRole('button', { name: /^Back/ })).toHaveCount(0);
  await expect(page.getByRole('complementary', { name: 'Photos in view' })).toContainText(
    '1 photo'
  );
});

test('the mini map of a photo opened from the map closes the loupe and moves there', async ({
  page
}) => {
  const photos = ASSETS.map((asset, index) => {
    const marker = MARKERS[index];
    return marker
      ? { ...asset, exifInfo: { city: marker.city, latitude: marker.lat, longitude: marker.lon } }
      : asset;
  });
  await installMocks(page, { assets: photos, mapMarkers: MARKERS });
  await page.goto('/map');
  const panel = page.getByRole('complementary', { name: 'Photos in view' });
  await expect(panel).toContainText('4 photos in view');
  await panel
    .locator(`[role="group"][data-name="IMG_0004.ARW"] > a`)
    .click({ position: { x: 8, y: 8 } });
  await page.keyboard.press('i');
  await page.getByRole('link', { name: 'Open Oslo on the map' }).click();
  await expect(page).toHaveURL(/at=59\.91390(,|%2C)10\.75220/);
  await expect(page.locator('[data-loupe-root]')).toHaveCount(0);
  await expect(panel).toContainText('1 photo in view');
});

test('clicking a group lists its photos and keeps them on screen', async ({ page }) => {
  await openStack(page);
  const panel = page.getByRole('complementary', { name: 'Photos in view' });
  await stack(page).click();
  await expect(panel).toContainText('2 photos');
  await expect(panel.locator('[role="group"][data-name]')).toHaveCount(2);
  await expect(panel).toContainText('Show all in view');
  const area = (await page.getByRole('region', { name: 'Map of your photos' }).boundingBox())!;
  await expect
    .poll(async () => {
      const box = await stack(page).first().boundingBox();
      return (
        box !== null &&
        box.y > area.y &&
        box.y + box.height < area.y + area.height &&
        box.x > area.x &&
        box.x + box.width < area.x + area.width
      );
    })
    .toBe(true);
});

test('moving the map after clicking a group lists every photo in view again', async ({ page }) => {
  await openStack(page);
  const panel = page.getByRole('complementary', { name: 'Photos in view' });
  await stack(page).click();
  await expect(panel).toContainText('Show all in view');
  await expect(panel.locator('[role="group"][data-name]')).toHaveCount(2);
  await page.waitForTimeout(1400);
  const area = (await page.getByRole('region', { name: 'Map of your photos' }).boundingBox())!;
  await page.mouse.move(area.x + area.width / 2, area.y + area.height / 2);
  await page.mouse.wheel(0, 2400);
  await expect(panel).not.toContainText('Show all in view');
  await expect(panel).toContainText(/photos? in view/);
  await page.waitForTimeout(800);
  await page.getByRole('button', { name: 'Fit all photos' }).click();
  await expect(panel).toContainText('3 photos in view');
});

test('the photos panel toggles from the header and resizes', async ({ page }) => {
  await openMap(page);
  const panel = page.getByRole('complementary', { name: 'Photos in view' });
  const map = page.getByRole('region', { name: 'Map of your photos' });
  const narrow = (await map.boundingBox())!.width;
  const toggle = page.getByRole('button', { name: 'Photos panel' });
  await expect(toggle).toHaveAttribute('aria-pressed', 'true');
  await toggle.click();
  await expect(panel).toHaveCount(0);
  await expect.poll(async () => (await map.boundingBox())!.width).toBeGreaterThan(narrow + 300);
  await toggle.click();
  await expect(panel).toBeVisible();
  await page.waitForTimeout(400);
  const before = (await panel.boundingBox())!.width;
  const handle = page.getByRole('slider', { name: 'Resize photos panel' });
  const grip = (await handle.boundingBox())!;
  await page.mouse.move(grip.x + grip.width / 2, grip.y + grip.height / 2);
  await page.mouse.down();
  await page.mouse.move(grip.x + grip.width / 2 - 120, grip.y + grip.height / 2, { steps: 6 });
  await page.mouse.up();
  await expect.poll(async () => (await panel.boundingBox())!.width).toBeGreaterThan(before + 100);
  const wide = (await panel.boundingBox())!.width;
  await page.reload();
  await expect(panel).toBeVisible();
  expect(Math.abs((await panel.boundingBox())!.width - wide)).toBeLessThan(2);
});

test('double-clicking a group marks and unmarks its photos without moving the map', async ({
  page
}) => {
  await openStack(page);
  await page.waitForTimeout(800);
  const before = (await stack(page).boundingBox())!;
  const url = page.url();
  const centre = { x: before.x + before.width / 2, y: before.y + before.height / 2 };
  await page.mouse.click(centre.x, centre.y);
  await page.waitForTimeout(300);
  await page.mouse.click(centre.x, centre.y);
  await expect(stack(page)).toHaveAttribute('data-marked', 'all');
  await page.waitForTimeout(900);
  const after = (await stack(page).boundingBox())!;
  expect(Math.abs(after.x - before.x)).toBeLessThan(1);
  expect(Math.abs(after.y - before.y)).toBeLessThan(1);
  expect(page.url()).toBe(url);
  await stack(page).dblclick();
  await expect(stack(page)).toHaveAttribute('data-marked', 'none');
  await page.waitForTimeout(600);
  await stack(page).click({ clickCount: 3 });
  await page.waitForTimeout(600);
  await expect(stack(page)).toHaveAttribute('data-marked', 'all');
  await expect(bulkBar(page)).toContainText('2 selected');
  await page.waitForTimeout(600);
  await stack(page).dblclick();
  await expect(stack(page)).toHaveAttribute('data-marked', 'none');
  await expect(bulkBar(page)).toHaveCount(0);
});
