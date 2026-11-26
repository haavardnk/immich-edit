import { expect, test, type Locator, type Page } from '@playwright/test';
import { fileURLToPath } from 'node:url';
import { bulkBar } from './browse';
import { installMocks, numberedAssets } from './helpers';
import { CaptureTimeMock } from './captureTimeMocks';
import { BERGEN, pointOnMap } from './mapFixtures';
import { selectTwo } from './touchShell';

const GPX = fileURLToPath(new URL('./fixtures/hike.gpx', import.meta.url));

type Written = { id: string; latitude: number; longitude: number };

const TAKEN = ['2026-10-31T09:10:00.000Z', '2026-10-31T09:20:00.000Z', '2026-10-31T11:00:00.000Z'];

const SHOOT = numberedAssets(3).map((asset, index) => ({
  ...asset,
  fileCreatedAt: TAKEN[index]!,
  localDateTime: TAKEN[index]!,
  exifInfo: { dateTimeOriginal: TAKEN[index]!, timeZone: 'Europe/Oslo' }
})) as ReturnType<typeof numberedAssets>;

async function open(page: Page, written: Written[]): Promise<void> {
  await installMocks(page, {
    assets: SHOOT,
    placeName: BERGEN,
    metadataPushConsented: true,
    onLocations: (items) => written.push(...items)
  });
}

async function lineUpOnTouch(page: Page): Promise<void> {
  await page.locator('[data-track-panel]').getByRole('button', { name: 'Use best fit' }).tap();
}

function clockSetup(panel: Locator): Locator {
  return panel.getByRole('button', { name: /Camera clock/ });
}

async function loadTrack(page: Page): Promise<void> {
  await page.getByLabel('Choose track files').setInputFiles(GPX);
  await expect(page.getByText('Hardangervidda')).toBeVisible();
}

test('a track places a selection after the camera clock is lined up', async ({ page }) => {
  const written: Written[] = [];
  await open(page, written);
  await page.goto('/photos');
  for (const name of ['IMG_0003.ARW', 'IMG_0001.ARW', 'IMG_0002.ARW']) {
    const tile = page.locator(`[role="group"][data-name="${name}"]`);
    await tile.hover();
    await tile.getByRole('button', { name: 'Select', exact: true }).click();
  }
  await bulkBar(page).getByRole('button', { name: 'Location', exact: true }).click();
  await page
    .getByRole('group', { name: 'Location' })
    .getByRole('button', { name: 'Match to a track…' })
    .click();

  await expect(page).toHaveURL(/\/map/);
  const panel = page.getByRole('complementary', { name: 'Match track' });
  await expect(panel).toContainText('3 selected photos');
  await loadTrack(page);

  const fit = panel.getByRole('group', { name: 'Best fit' });
  await expect(fit).toContainText('Best fit: Camera 1 h fast');
  await expect(fit).toContainText('Puts 2 of 3 photos on the track');
  await expect(panel.getByRole('button', { name: 'Place 0 photos' })).toBeDisabled();
  await fit.getByRole('button', { name: 'Use best fit' }).click();
  await expect(fit).toHaveCount(0);
  await expect(clockSetup(panel)).toContainText('Camera 1 h fast');
  await clockSetup(panel).click();
  await expect(panel.getByRole('textbox', { name: 'Clock correction' })).toHaveValue('−1:00:00');

  const photos = panel.getByRole('list', { name: 'Photos on the track' });
  await expect(photos).toContainText('On the track');
  await expect(panel.locator('[data-track-photo]')).toContainText([
    'IMG_0001',
    'IMG_0002',
    'IMG_0003'
  ]);
  await expect(photos).toContainText('90 min after end');
  await panel.getByRole('button', { name: 'Place 2 photos' }).click();

  await expect(page.getByText('Placed 2 photos')).toBeVisible();
  expect(written).toEqual([
    { id: SHOOT[0]!.id, latitude: 60.11, longitude: 6.71 },
    { id: SHOOT[1]!.id, latitude: 60.12, longitude: 6.72 }
  ]);
  await expect(panel).toHaveCount(0);
});

test('placing can correct the capture time too, and one Undo reverts both', async ({ page }) => {
  const written: Written[] = [];
  const times = new CaptureTimeMock();
  await installMocks(page, {
    assets: SHOOT,
    placeName: BERGEN,
    metadataPushConsented: true,
    onLocations: (items) => written.push(...items),
    captureTime: times
  });
  await page.goto('/map');
  await page.getByRole('button', { name: 'Match track' }).click();
  const panel = page.getByRole('complementary', { name: 'Match track' });
  await loadTrack(page);
  const fix = panel.getByRole('checkbox', { name: 'Correct the capture time of 2 photos' });
  await expect(panel.getByRole('checkbox', { name: /Correct the capture time/ })).toHaveCount(0);

  await panel.getByRole('button', { name: 'Use best fit' }).click();
  await expect(fix).not.toBeChecked();
  await expect(panel).toContainText('1 h earlier');
  await fix.check();
  await panel.getByRole('button', { name: 'Place 2 photos' }).click();

  await expect(page.getByText('Placed 2 photos and corrected their capture time')).toBeVisible();
  expect(written).toHaveLength(2);
  expect(times.calls).toEqual([
    [
      { id: SHOOT[0]!.id, minutes: -60 },
      { id: SHOOT[1]!.id, minutes: -60 }
    ]
  ]);
  await page.getByRole('button', { name: 'Undo' }).click();
  await expect.poll(() => times.calls.length).toBe(2);
  expect(times.calls[1]).toEqual([
    { id: SHOOT[0]!.id, minutes: 60 },
    { id: SHOOT[1]!.id, minutes: 60 }
  ]);
});

test('each camera keeps its own clock', async ({ page }) => {
  const [first, second, third] = numberedAssets(3);
  const shots = [
    { asset: first!, at: '2026-10-31T09:10:00.000Z', make: 'SONY', model: 'ILCE-7M3' },
    { asset: second!, at: '2026-10-31T09:20:00.000Z', make: 'SONY', model: 'ILCE-7M3' },
    { asset: third!, at: '2026-10-31T08:15:00.000Z', make: 'Apple', model: 'iPhone 15 Pro' }
  ].map(({ asset, at, make, model }) => ({
    ...asset,
    fileCreatedAt: at,
    localDateTime: at,
    exifInfo: { dateTimeOriginal: at, timeZone: 'Europe/Oslo', make, model }
  })) as ReturnType<typeof numberedAssets>;
  await installMocks(page, { assets: shots, placeName: BERGEN, metadataPushConsented: true });
  await page.goto('/map');
  await page.getByRole('button', { name: 'Match track' }).click();
  const panel = page.getByRole('complementary', { name: 'Match track' });
  await loadTrack(page);

  const fit = panel.getByRole('group', { name: 'Best fit' });
  await expect(fit).toContainText('Best fit for ILCE-7M3: 1 h fast');
  await expect(panel.getByRole('button', { name: 'Place 1 photo' })).toBeVisible();
  await fit.getByRole('button', { name: 'Use best fit' }).click();
  await expect(clockSetup(panel)).toContainText('ILCE-7M3 1 h fast · 1 camera unchanged');
  await expect(panel.getByRole('button', { name: 'Place 3 photos' })).toBeVisible();

  await clockSetup(panel).click();
  const correction = panel.getByRole('textbox', { name: 'Clock correction' });
  await expect(correction).toHaveValue('−1:00:00');
  await panel.getByRole('button', { name: 'Camera: ILCE-7M3' }).click();
  await page.getByRole('button', { name: /iPhone 15 Pro/ }).click();
  await expect(correction).toHaveValue('+0:00:00');
  await expect(panel.getByRole('button', { name: 'Place 3 photos' })).toBeVisible();
});

test('typing a correction moves the photos along the track', async ({ page }) => {
  await open(page, []);
  await page.goto('/map');
  await page.getByRole('button', { name: 'Match track' }).click();
  const panel = page.getByRole('complementary', { name: 'Match track' });
  await expect(panel).toContainText('Choose GPX, KML or TCX files');
  await loadTrack(page);
  await expect(panel).toContainText('photos taken during the track');
  await clockSetup(panel).click();

  const correction = panel.getByRole('textbox', { name: 'Clock correction' });
  await correction.fill('-0:55');
  await correction.press('Enter');
  await expect(clockSetup(panel)).toContainText('Camera 55 min fast');
  await panel.getByRole('button', { name: 'One minute earlier' }).click();
  await expect(correction).toHaveValue('−0:56:00');
  await panel.getByRole('button', { name: 'Close track' }).click();
  await expect(panel).toHaveCount(0);
  await expect(page.getByRole('complementary', { name: 'Photos in view' })).toBeVisible();
});

test('hovering a pin points at its photo and a click opens it large', async ({ page }) => {
  await open(page, []);
  await page.goto('/map');
  await page.getByRole('button', { name: 'Match track' }).click();
  const panel = page.getByRole('complementary', { name: 'Match track' });
  await loadTrack(page);
  await panel.getByRole('button', { name: 'Use best fit' }).click();
  const photos = panel.getByRole('list', { name: 'Photos on the track' });
  const first = photos.locator(`[data-track-photo="${SHOOT[0]!.id}"]`);
  await expect(first).toContainText('On the track');

  await expect(page).toHaveURL(/at=60\.1\d+%2C6\.7/);
  const pin = await pointOnMap(page, 60.11, 6.71);
  await page.mouse.move(pin.x, pin.y);
  await expect(first).toHaveAttribute('aria-current', 'true');
  const card = page.locator('[data-pin-preview]');
  await expect(card).toContainText('IMG_0001.ARW');
  await expect(card).toContainText('On the track');
  await expect(card.getByRole('img', { name: 'IMG_0001.ARW' })).toBeVisible();
  await page.mouse.move(pin.x + 80, pin.y + 80);
  await expect(first).not.toHaveAttribute('aria-current');
  await expect(card).toHaveCount(0);

  await first.hover();
  await expect(first).toHaveAttribute('aria-current', 'true');
  await expect(card).toContainText('IMG_0001.ARW');
  const shown = (await card.boundingBox())!;
  expect(Math.abs(shown.x - (pin.x + 18))).toBeLessThan(4);
  await page.mouse.move(pin.x + 80, pin.y + 80);
  await expect(card).toHaveCount(0);

  await page.mouse.click(pin.x, pin.y);
  const loupe = page.locator('[data-loupe-root]');
  await expect(loupe.getByRole('img', { name: 'IMG_0001.ARW' })).toBeVisible();
  await page.keyboard.press('ArrowRight');
  await expect(loupe.getByRole('img', { name: 'IMG_0002.ARW' })).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(loupe).toHaveCount(0);
  await expect(panel).toBeVisible();

  await photos.getByRole('link', { name: 'Open IMG_0002.ARW' }).click();
  await expect(loupe.getByRole('img', { name: 'IMG_0002.ARW' })).toBeVisible();
});

test('photo markers under the track let the pins be hovered', async ({ page }) => {
  await installMocks(page, {
    assets: SHOOT,
    mapMarkers: [{ id: SHOOT[2]!.id, lat: 60.11, lon: 6.71, city: 'Bergen' }],
    placeName: BERGEN,
    metadataPushConsented: true
  });
  await page.goto('/map');
  await page.getByRole('button', { name: 'Match track' }).click();
  const panel = page.getByRole('complementary', { name: 'Match track' });
  await loadTrack(page);
  await panel.getByRole('button', { name: 'Use best fit' }).click();
  await expect(page).toHaveURL(/at=60\.115\d*%2C6\.715/);

  const marker = page.locator(`[data-map-photo="${SHOOT[2]!.id}"]`);
  await expect(marker).toHaveCSS('pointer-events', 'none');
  await expect(marker).toHaveCSS('opacity', '0.35');

  const pin = await pointOnMap(page, 60.11, 6.71);
  await page.mouse.move(pin.x, pin.y);
  await expect(panel.locator(`[data-track-photo="${SHOOT[0]!.id}"]`)).toHaveAttribute(
    'aria-current',
    'true'
  );
  await expect(page.locator('[data-pin-preview]')).toContainText('IMG_0001.ARW');
});

function trackBox(page: Page): Locator {
  return page
    .getByRole('region', { name: 'Track' })
    .getByRole('button', { name: /Choose GPX, KML or TCX files/ });
}

test('the whole track box opens the file picker', async ({ page }) => {
  await open(page, []);
  await page.goto('/map');
  await page.getByRole('button', { name: 'Match track' }).click();
  await expect(trackBox(page)).toContainText('or drop them on the map');
  const chooser = page.waitForEvent('filechooser');
  await trackBox(page).click({ position: { x: 8, y: 8 } });
  await (await chooser).setFiles(GPX);
  await expect(page.getByText('Hardangervidda')).toBeVisible();
});

test.describe('a camera without a time zone', () => {
  test.use({ timezoneId: 'America/New_York' });

  test('reads its clock in the zone where the track was recorded', async ({ page }) => {
    const wall = ['2026-10-31T09:10:00.000Z', '2026-10-31T09:20:00.000Z'];
    const unzoned = numberedAssets(2).map((asset, index) => ({
      ...asset,
      fileCreatedAt: wall[index]!,
      localDateTime: wall[index]!,
      exifInfo: { dateTimeOriginal: wall[index]!, timeZone: null }
    })) as ReturnType<typeof numberedAssets>;
    await installMocks(page, { assets: unzoned, placeName: BERGEN, metadataPushConsented: true });
    await page.goto('/map');
    await page.getByRole('button', { name: 'Match track' }).click();
    const panel = page.getByRole('complementary', { name: 'Match track' });
    await loadTrack(page);
    await clockSetup(panel).click();
    const clock = panel.getByRole('region', { name: 'Camera clock' });
    await expect(clock).toContainText('from the track');
    await expect(clock.getByRole('combobox', { name: 'Camera time zone' })).toHaveValue(
      /Europe\/Oslo/
    );
    await expect(panel.getByRole('button', { name: '2 on track' })).toBeVisible();
  });
});

test.describe('track matching on touch', { tag: '@touch' }, () => {
  test('a tap anywhere on the track box opens the file picker', async ({ page }) => {
    await open(page, []);
    await page.goto('/map');
    await page.getByRole('button', { name: 'Match track' }).tap();
    await expect(trackBox(page)).not.toContainText('drop');
    const chooser = page.waitForEvent('filechooser');
    await trackBox(page).tap({ position: { x: 8, y: 8 } });
    await (await chooser).setFiles(GPX);
    await expect(page.getByText('Hardangervidda')).toBeVisible();
  });

  test('the More sheet opens track matching for a selection', async ({ page }) => {
    const written: Written[] = [];
    await open(page, written);
    await page.goto('/photos');
    await selectTwo(page);
    await bulkBar(page).getByRole('button', { name: 'More', exact: true }).tap();
    await page
      .getByRole('dialog', { name: 'More actions' })
      .getByRole('button', { name: 'Location' })
      .tap();
    await page
      .getByRole('dialog', { name: 'Location' })
      .getByRole('button', { name: 'Match to a track…' })
      .tap();
    await expect(page).toHaveURL(/\/map/);
    await loadTrack(page);
    await lineUpOnTouch(page);
    await page.getByRole('button', { name: 'Place 2 photos' }).tap();
    await expect.poll(() => written.length).toBe(2);
  });

  test('a tapped photo shows its card first, then opens large', async ({ page }) => {
    await open(page, []);
    await page.goto('/map');
    await page.getByRole('button', { name: 'Match track' }).tap();
    await loadTrack(page);
    await lineUpOnTouch(page);
    const panel = page.locator('[data-track-panel]');
    const loupe = page.locator('[data-loupe-root]');
    const strip = panel.getByRole('list', { name: 'Photos on the track' });
    const inStrip = strip.getByRole('link', { name: 'IMG_0002.ARW, On the track' });
    if ((await inStrip.count()) === 0) {
      await panel.getByRole('link', { name: 'Open IMG_0002.ARW' }).tap();
      await expect(loupe.getByRole('img', { name: 'IMG_0002.ARW' })).toBeVisible();
      return;
    }
    await inStrip.tap();
    await expect(inStrip).toHaveAttribute('aria-current', 'true');
    const card = page.locator('[data-pin-preview]');
    await expect(card).toContainText('IMG_0002.ARW');
    await expect(loupe).toHaveCount(0);
    await card.getByRole('link', { name: 'Open IMG_0002.ARW' }).tap();
    await expect(loupe.getByRole('img', { name: 'IMG_0002.ARW' })).toBeVisible();
  });

  test('going back to the library closes the track', async ({ page }, info) => {
    test.skip(info.project.name !== 'touch-phone', 'phone has Back');
    await open(page, []);
    await page.goto('/library');
    await page.getByRole('link', { name: /^Map/ }).filter({ visible: true }).first().tap();
    await page.getByRole('button', { name: 'Match track' }).tap();
    await loadTrack(page);
    await expect(page.locator('[data-track-sheet]')).toBeVisible();
    await page.getByRole('button', { name: 'Back', exact: true }).tap();
    await expect(page).toHaveURL(/\/library$/);
    await page.getByRole('link', { name: /^Map/ }).filter({ visible: true }).first().tap();
    await expect(page.getByRole('region', { name: 'Map of your photos' })).toBeVisible();
    await expect(page.locator('[data-track-sheet]')).toHaveCount(0);
    await expect(page.getByRole('button', { name: 'Match track' })).toHaveAttribute(
      'aria-pressed',
      'false'
    );
  });

  test('the phone sheet offers the capture time correction', async ({ page }, info) => {
    test.skip(info.project.name !== 'touch-phone', 'phone sheet');
    const times = new CaptureTimeMock();
    await installMocks(page, {
      assets: SHOOT,
      placeName: BERGEN,
      metadataPushConsented: true,
      captureTime: times
    });
    await page.goto('/map');
    await page.getByRole('button', { name: 'Match track' }).tap();
    await loadTrack(page);
    await lineUpOnTouch(page);
    await page.getByRole('button', { name: 'Resize sheet' }).tap();
    const sheet = page.locator('[data-track-sheet]');
    const fix = sheet.getByRole('checkbox', { name: 'Correct the capture time of 2 photos' });
    await fix.tap();
    await expect(fix).toBeChecked();
    await sheet.getByRole('button', { name: 'Place 2 photos' }).tap();
    await expect.poll(() => times.calls.length).toBe(1);
  });

  test('the phone sheet minimizes to its title row', async ({ page }, info) => {
    test.skip(info.project.name !== 'touch-phone', 'phone sheet');
    await open(page, []);
    await page.goto('/map');
    await page.getByRole('button', { name: 'Match track' }).tap();
    await loadTrack(page);
    const sheet = page.locator('[data-track-sheet]');
    const resize = page.getByRole('button', { name: 'Resize sheet' });
    const height = async (): Promise<number> => (await sheet.boundingBox())!.height;
    const peek = await height();
    await resize.tap();
    await expect.poll(height).toBeGreaterThan(peek + 100);
    await resize.tap();
    await expect.poll(height).toBeLessThan(140);
    await expect(sheet.getByRole('heading', { name: /photos on the track/ })).toBeInViewport();
    await resize.tap();
    await expect.poll(height).toBeGreaterThan(peek - 4);
    await expect(sheet.getByRole('list', { name: 'Photos on the track' })).toBeInViewport();
  });

  test('a tapped dot previews its photo and a second tap opens it', async ({ page }, info) => {
    test.skip(info.project.name !== 'touch-tablet', 'tablet panel');
    await open(page, []);
    await page.goto('/map');
    await page.getByRole('button', { name: 'Match track' }).tap();
    await loadTrack(page);
    const panel = page.getByRole('complementary', { name: 'Match track' });
    await panel.getByRole('button', { name: 'Use best fit' }).tap();
    await expect(page).toHaveURL(/%2C1[2-9]\.\d\d$/);
    const pin = await pointOnMap(page, 60.11, 6.71);
    const card = page.locator('[data-pin-preview]');
    await page.touchscreen.tap(pin.x, pin.y);
    await expect(card).toContainText('IMG_0001.ARW');
    expect((await card.boundingBox())!.height).toBeLessThan(120);
    await expect(page.locator('[data-loupe-root]')).toHaveCount(0);
    await page.touchscreen.tap(pin.x - 150, pin.y - 150);
    await expect(card).toHaveCount(0);
    await page.touchscreen.tap(pin.x, pin.y);
    await expect(card).toContainText('IMG_0001.ARW');
    await page.waitForTimeout(600);
    await page.touchscreen.tap(pin.x, pin.y);
    await expect(
      page.locator('[data-loupe-root]').getByRole('img', { name: 'IMG_0001.ARW' })
    ).toBeVisible();
  });

  test('phone landscape docks the track panel beside the map', async ({ page }, info) => {
    test.skip(info.project.name !== 'touch-phone', 'phone only');
    await page.setViewportSize({ width: 844, height: 390 });
    await open(page, []);
    await page.goto('/map');
    await page.getByRole('button', { name: 'Match track' }).tap();
    const panel = page.getByRole('complementary', { name: 'Match track' });
    await expect(panel).toBeVisible();
    await loadTrack(page);
    const box = (await panel.boundingBox())!;
    expect(Math.round(box.width)).toBe(300);
    expect(box.x + box.width).toBeGreaterThan(800);
    await expect(panel.getByRole('button', { name: /^Place/ })).toBeVisible();
  });
});
