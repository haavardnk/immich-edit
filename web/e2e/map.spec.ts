import { expect, test, type Page } from '@playwright/test';
import { bulkBar } from './browse';
import { installMocks, NEUTRAL_RECORD, numberedAssets } from './helpers';
import {
  ASSETS,
  MARKERS,
  openMap,
  markers,
  openStack,
  stack,
  STACK,
  STACK_MARKERS
} from './mapFixtures';

async function watchClosingBackdrop(page: Page): Promise<void> {
  await page.evaluate(() => {
    Reflect.set(window, '__closingBackdrop', false);
    const animate = Element.prototype.animate;
    Element.prototype.animate = function (...args: Parameters<Element['animate']>) {
      if (this.classList.contains('bg-image-canvas'))
        Reflect.set(window, '__closingBackdrop', true);
      return animate.apply(this, args);
    };
  });
}

function closingBackdrop(page: Page): Promise<boolean> {
  return page.evaluate(() => Reflect.get(window, '__closingBackdrop') === true);
}

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
  const markers = page.waitForRequest(
    (request) =>
      request.url().includes('/api/map/markers') && request.url().includes('from=2024-01-01')
  );
  await from.locator('[data-segment="month"]').click();
  await page.keyboard.type('01012024');
  await expect(page).toHaveURL(/after=2024-01-01/);
  const sent = new URL((await markers).url()).searchParams;
  expect(sent.get('from')).toBe('2024-01-01T00:00:00');
  expect(sent.has('after')).toBe(false);
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

test('the map comes back where it was after the loupe and the editor', async ({ page }) => {
  await openStack(page);
  await page.goto('/map?at=60.15659,8.03810,6.12');
  await expect(stack(page)).toHaveCount(1);
  await stack(page).click();
  await expect(page).toHaveURL(/at=60\.39750%2C5\.32400%2C12\.00/);
  const panel = page.getByRole('complementary', { name: 'Photos in view' });
  await expect(panel).toContainText('Show all in view');
  const zoomed = new URL(page.url()).searchParams.get('at');

  await panel
    .locator('[role="group"][data-name] > a')
    .first()
    .click({ position: { x: 8, y: 8 } });
  await page.getByRole('button', { name: 'Edit', exact: true }).click();
  await expect(page.getByTestId('preview-image')).toBeVisible();
  const from = new URL(page.url()).searchParams.get('from') ?? '';
  expect(new URL(from, page.url()).searchParams.get('at')).toBe(zoomed);

  await page.evaluate(() => {
    const w = window as unknown as { bare: number };
    w.bare = 0;
    const tick = (): void => {
      const onMap = location.pathname === '/map';
      if (onMap && !document.querySelector('[data-loupe-root]')) w.bare += 1;
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  });
  await page.getByRole('button', { name: /^Back/ }).first().click();
  await expect(page).toHaveURL(/\/map/);
  await expect(page.locator('[data-loupe-root]')).toHaveCount(1);
  expect(await page.evaluate(() => (window as unknown as { bare: number }).bare)).toBe(0);
  await watchClosingBackdrop(page);
  await page.getByRole('button', { name: /^Back/ }).first().click();
  await expect.poll(() => closingBackdrop(page)).toBe(true);
  await expect(page.locator('[data-loupe-root]')).toHaveCount(0);
  await expect(page).toHaveURL(/\/map\?/);
  expect(new URL(page.url()).searchParams.get('at')).toBe(zoomed);
  await expect(panel).toContainText('Show all in view');
  await expect(panel.locator('[role="group"][data-name]')).toHaveCount(2);
});

test('a photo closed after the editor flies from the loupe back to its tile', async ({ page }) => {
  const photos = numberedAssets(40);
  await installMocks(page, {
    assets: photos,
    mapMarkers: photos.map((asset) => ({ id: asset.id, lat: 60.3975, lon: 5.324, city: 'Bergen' }))
  });
  await page.goto('/map');
  const panel = page.getByRole('complementary', { name: 'Photos in view' });
  await expect(panel).toContainText('40 photos in view');
  const tile = panel.locator('[role="group"][data-name="IMG_0030.ARW"]');
  await panel.locator('[role="group"][data-name]').first().hover();
  await expect
    .poll(async () => {
      if (await tile.isVisible()) return true;
      await page.mouse.wheel(0, 300);
      return false;
    })
    .toBe(true);
  await tile.scrollIntoViewIfNeeded();
  await tile.locator('> a').click({ position: { x: 8, y: 8 } });
  await expect(page.locator('[data-loupe-root]')).toHaveCount(1);

  await page.getByRole('button', { name: 'Edit', exact: true }).click();
  const editorPhoto = page.locator('.editor-stage').getByTestId('preview-image');
  await expect(editorPhoto).toBeVisible();
  await page.getByRole('button', { name: /^Back/ }).first().click();
  await expect(page).toHaveURL(/\/map/);
  const track = (await page.getByTestId('loupe-track').boundingBox())!;

  await expect(tile).toBeInViewport();
  const landed = (await tile.locator('img').boundingBox())!;
  await page.keyboard.press('Escape');
  const flight = () =>
    page.evaluate(() =>
      document
        .getAnimations()
        .map((animation) => animation.effect as KeyframeEffect)
        .find((effect) => (effect.target as HTMLElement | null)?.tagName === 'CANVAS')
        ?.getKeyframes()
        .map((frame) => [
          Number.parseFloat(String(frame.left)),
          Number.parseFloat(String(frame.width))
        ])
    );
  await expect.poll(async () => (await flight())?.length, { intervals: [16] }).toBe(2);
  const [from, to] = (await flight())!;
  expect(Math.abs(from![0]! + from![1]! / 2 - (track.x + track.width / 2))).toBeLessThan(3);
  expect(Math.abs(to![0]! - landed.x)).toBeLessThan(3);
});

test('a loupe closed before its edited image loads still flies back to the tile', async ({
  page
}) => {
  await installMocks(page, {
    assets: STACK,
    mapMarkers: STACK_MARKERS,
    editRecord: { ...NEUTRAL_RECORD, hash: 'hash-saved' }
  });
  await page.goto('/map');
  await stack(page).click();
  const panel = page.getByRole('complementary', { name: 'Photos in view' });
  await panel
    .locator('[role="group"][data-name] > a')
    .first()
    .click({ position: { x: 8, y: 8 } });
  await page.getByRole('button', { name: 'Edit', exact: true }).click();
  await expect(page.locator('.editor-stage').getByTestId('preview-image')).toBeVisible();
  const saved = page.waitForRequest(
    (req) => req.method() === 'PUT' && /\/api\/assets\/[^/]+\/edits$/.test(req.url())
  );
  await page.getByRole('slider', { name: 'Exposure', exact: true }).focus();
  await page.keyboard.press('ArrowRight');
  await saved;
  await page.route(/\/api\/assets\/[^/]+\/preview\?.*r=/, async (route) => {
    await new Promise((resolve) => setTimeout(resolve, 3000));
    await route.fallback();
  });

  await page.getByRole('button', { name: /^Back/ }).first().click();
  await expect(page).toHaveURL(/\/map/);
  await watchClosingBackdrop(page);
  await page.keyboard.press('Escape');
  await expect(page.locator('[data-loupe-root]')).toHaveCount(0);
  await expect.poll(() => closingBackdrop(page)).toBe(true);
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
  const toggle = page.getByRole('button', { name: 'Side panel' });
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
  await page.waitForTimeout(100);
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
