import { expect, test, type Locator, type Page } from '@playwright/test';
import { fileURLToPath } from 'node:url';
import { installMocks, numberedAssets } from './helpers';
import { markers, openMap, openStack, pointOnMap, stack } from './mapFixtures';

const GPX = fileURLToPath(new URL('./fixtures/hike.gpx', import.meta.url));

const TAKEN = ['2026-10-31T09:10:00.000Z', '2026-10-31T09:20:00.000Z', '2026-10-31T11:00:00.000Z'];

const SHOOT = numberedAssets(3).map((asset, index) => ({
  ...asset,
  fileCreatedAt: TAKEN[index]!,
  localDateTime: TAKEN[index]!,
  exifInfo: { dateTimeOriginal: TAKEN[index]!, timeZone: 'Europe/Oslo' }
})) as ReturnType<typeof numberedAssets>;

function fullscreen(page: Page) {
  return page.locator('[data-map-fullscreen]');
}

function exitButton(page: Page) {
  return page.getByRole('button', { name: 'Exit fullscreen', exact: true });
}

function card(page: Page) {
  return page.locator('[data-track-card]');
}

async function openTrack(page: Page): Promise<void> {
  await installMocks(page, { assets: SHOOT, metadataPushConsented: true });
  await page.goto('/map');
  await page.getByRole('button', { name: 'Geotag from track', exact: true }).click();
  await page.getByLabel('Choose track files').setInputFiles(GPX);
  await expect(page.getByText('Hardangervidda').first()).toBeVisible();
}

async function boxOf(locator: Locator) {
  await expect.poll(async () => (await locator.boundingBox()) !== null).toBe(true);
  return (await locator.boundingBox())!;
}

async function expectTopRight(page: Page, bar: Locator): Promise<void> {
  const zoom = page.getByRole('button', { name: 'Zoom in', exact: true });
  const right = async (locator: Locator) => {
    const box = await locator.boundingBox();
    return box ? box.x + box.width : NaN;
  };
  await expect
    .poll(async () => Math.abs((await right(exitButton(page))) - (await right(zoom))))
    .toBeLessThan(2);
  const viewport = page.viewportSize()!;
  const shown = await boxOf(bar);
  expect(shown.x).toBeGreaterThan(viewport.width / 2);
  expect(shown.y).toBeLessThan(20);
  expect((await boxOf(zoom)).y).toBeGreaterThan(shown.y + shown.height);
}

async function expectCovers(page: Page): Promise<void> {
  const box = (await fullscreen(page).boundingBox())!;
  const viewport = page.viewportSize()!;
  expect(Math.round(box.x)).toBe(0);
  expect(Math.round(box.y)).toBe(0);
  expect(Math.round(box.width)).toBe(viewport.width);
  expect(Math.round(box.height)).toBe(viewport.height);
}

test('the map goes fullscreen from its header, F and Escape', async ({ page }) => {
  await openMap(page);
  const panel = page.getByRole('complementary', { name: 'Photos in view' });
  await expect(panel).toBeVisible();
  await page.getByRole('button', { name: 'Fullscreen', exact: true }).click();
  await expect(fullscreen(page)).toBeVisible();
  await expectCovers(page);
  await expect(page.getByRole('heading', { name: 'Map', exact: true })).toHaveCount(0);
  await expect(panel).toBeHidden();
  await expect(markers(page).first()).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(fullscreen(page)).toHaveCount(0);
  await expect(panel).toBeVisible();
  await page.keyboard.press('f');
  await expect(fullscreen(page)).toBeVisible();
  await page.keyboard.press('f');
  await expect(fullscreen(page)).toHaveCount(0);
});

test('the fullscreen bar exits and keeps saved places and fit at hand', async ({ page }) => {
  await openMap(page);
  await page.keyboard.press('f');
  const bar = page.locator('[data-map-view-bar]');
  const stars = bar.getByRole('button', { name: 'Saved places', exact: true });
  await expect(stars).toHaveAttribute('aria-pressed', 'true');
  await stars.click();
  await expect(stars).toHaveAttribute('aria-pressed', 'false');
  await expect(bar.getByRole('button', { name: 'Fit all photos', exact: true })).toBeEnabled();
  await expectTopRight(page, bar);
  await exitButton(page).click();
  await expect(fullscreen(page)).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Saved places', exact: true })).toHaveAttribute(
    'aria-pressed',
    'false'
  );
});

test('Escape leaves fullscreen before it clears a selection', async ({ page }) => {
  await openStack(page);
  await page.keyboard.press('f');
  await stack(page).dblclick();
  await expect(stack(page)).toHaveAttribute('data-marked', 'all');
  await page.keyboard.press('Escape');
  await expect(fullscreen(page)).toHaveCount(0);
  await expect(stack(page)).toHaveAttribute('data-marked', 'all');
  await page.keyboard.press('Escape');
  await expect(stack(page)).toHaveAttribute('data-marked', 'none');
});

test('a fullscreen track keeps its summary and Place in a floating card', async ({ page }) => {
  await openTrack(page);
  const panel = page.getByRole('complementary', { name: 'Geotag from track' });
  await expect(panel).toBeVisible();
  await page.keyboard.press('f');
  await expect(panel).toBeHidden();
  await expect(card(page)).toContainText(/\d of \d photos on the track/);
  await expect(card(page)).not.toContainText('Camera time');
  await expect(card(page).getByRole('button', { name: /^(Place|Save time)/ })).toBeVisible();
  const photos = page
    .locator('[data-map-view-bar]')
    .getByRole('button', { name: 'Photos on the map', exact: true });
  await expect(photos).toHaveAttribute('aria-pressed', 'true');
  const viewport = page.viewportSize()!;
  await expect
    .poll(async () => {
      const box = await card(page).boundingBox();
      return box ? Math.abs(box.x + box.width / 2 - viewport.width / 2) : Infinity;
    })
    .toBeLessThan(2);
  await page.keyboard.press('Escape');
  await expect(card(page)).toHaveCount(0);
  await expect(panel).toBeVisible();
});

test('fullscreen stays under the loupe and ends when the map is left for Library', async ({
  page
}) => {
  await openTrack(page);
  const panel = page.getByRole('complementary', { name: 'Geotag from track' });
  await panel.getByRole('button', { name: /^Camera time/ }).click();
  const shift = panel.getByRole('textbox', { name: 'Shift Unknown camera by', exact: true });
  await shift.fill('-1:00');
  await shift.press('Enter');
  await shift.blur();
  await expect(page).toHaveURL(/at=60\.1\d+%2C6\.7/);
  await page.keyboard.press('f');
  await expect(fullscreen(page)).toBeVisible();
  const pin = await pointOnMap(page, 60.11, 6.71);
  await page.mouse.move(pin.x, pin.y);
  await expect(page.locator('[data-pin-preview]')).toContainText('IMG_0001.ARW');
  await page.mouse.click(pin.x, pin.y);
  const loupe = page.locator('[data-loupe-root]');
  await expect(loupe.getByRole('img', { name: 'IMG_0001.ARW' })).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(loupe).toHaveCount(0);
  await expect(fullscreen(page)).toBeVisible();
  await expect(card(page)).toBeVisible();

  await page.goto('/library');
  await page.getByRole('link', { name: /^Map/ }).filter({ visible: true }).first().click();
  await expect(page.getByRole('button', { name: 'Fullscreen', exact: true })).toBeVisible();
  await page.keyboard.press('f');
  await expect(fullscreen(page)).toBeVisible();
  await page.goBack();
  await expect(page).toHaveURL(/\/library$/);
  await page.getByRole('link', { name: /^Map/ }).filter({ visible: true }).first().click();
  await expect(page).toHaveURL(/\/map/);
  await expect(page.getByRole('button', { name: 'Fullscreen', exact: true })).toBeVisible();
  await expect(fullscreen(page)).toHaveCount(0);
});

test.describe('fullscreen map on touch', { tag: '@touch' }, () => {
  test('the map covers the screen and its bar uses touch-size buttons', async ({ page }) => {
    await openMap(page);
    await page.getByRole('button', { name: 'Fullscreen', exact: true }).tap();
    await expect(fullscreen(page)).toBeVisible();
    await expectCovers(page);
    await expect(page.locator('[data-map-pill]')).toHaveCount(0);
    const exit = (await exitButton(page).boundingBox())!;
    expect(Math.round(exit.width)).toBe(44);
    expect(Math.round(exit.height)).toBe(44);
    await exitButton(page).tap();
    await expect(fullscreen(page)).toHaveCount(0);
  });

  test('a tapped marker zooms without opening the tray', async ({ page }) => {
    await openMap(page);
    await page.getByRole('button', { name: 'Fullscreen', exact: true }).tap();
    await expect(page).toHaveURL(/at=/);
    const before = new URL(page.url()).searchParams.get('at');
    await markers(page).first().tap();
    await expect.poll(() => new URL(page.url()).searchParams.get('at')).not.toBe(before);
    await expect(page.locator('[data-map-sheet]')).toHaveCount(0);
    await expect(fullscreen(page)).toBeVisible();
  });

  test('a track card stacks on portrait screens and sits in a row on landscape', async ({
    page
  }, info) => {
    await installMocks(page, { assets: SHOOT, metadataPushConsented: true });
    await page.goto('/map');
    await page.getByRole('button', { name: 'Geotag from track', exact: true }).tap();
    await page.getByLabel('Choose track files').setInputFiles(GPX);
    await expect(page.getByText('Hardangervidda').first()).toBeVisible();
    await page.getByRole('button', { name: 'Fullscreen', exact: true }).tap();
    await expect(page.locator('[data-track-sheet]')).toBeHidden();
    const place = card(page).getByRole('button', { name: /^(Place|Save time)/ });
    await expect(place).toBeVisible();
    const photos = page
      .locator('[data-map-view-bar]')
      .getByRole('button', { name: 'Photos on the map', exact: true });
    expect((await boxOf(photos)).width).toBe(44);
    const viewport = page.viewportSize()!;
    const box = await boxOf(card(page));
    const button = await boxOf(place);
    if (info.project.name === 'touch-phone') {
      expect(Math.round(viewport.width - box.width)).toBe(24);
      expect(button.width).toBeGreaterThan(box.width - 40);
      await page.setViewportSize({ width: 844, height: 390 });
      await expect
        .poll(async () => (await card(page).boundingBox())?.height ?? Infinity)
        .toBeLessThan(90);
      const row = await boxOf(card(page));
      expect(Math.round(844 - (row.x + row.width))).toBe(12);
      expect(row.width).toBeLessThan(700);
      await expect(page.getByRole('complementary', { name: 'Geotag from track' })).toBeHidden();
    } else {
      expect(Math.abs(box.x + box.width / 2 - viewport.width / 2)).toBeLessThan(2);
      expect(button.width).toBeLessThan(box.width / 2);
      await page.setViewportSize({ width: 820, height: 1180 });
      await expect.poll(async () => (await card(page).boundingBox())?.width).toBe(796);
      const column = await boxOf(place);
      expect(column.width).toBeGreaterThan(760);
    }
    await exitButton(page).tap();
    await expect(card(page)).toHaveCount(0);
  });
});
