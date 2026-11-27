import { expect, test, type Locator, type Page } from '@playwright/test';
import { fileURLToPath } from 'node:url';
import { openBulkLocation, openTouchLocation } from './browse';
import { CaptureTimeMock } from './captureTimeMocks';
import { installMocks, numberedAssets } from './helpers';
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

function cameraShift(panel: Locator): Locator {
  return panel.getByRole('region', { name: 'Camera time', exact: true });
}

function shiftHeader(panel: Locator): Locator {
  return cameraShift(panel).getByRole('button', { name: /^Camera time/ });
}

async function shiftCamera(scope: Locator, camera: string, value: string): Promise<void> {
  const box = scope.getByRole('textbox', { name: `Shift ${camera} by`, exact: true });
  await box.fill(value);
  await box.press('Enter');
}

async function pickZone(page: Page, field: Locator, place: string, zone: RegExp): Promise<void> {
  await field.click();
  await field.pressSequentially(place);
  await page.getByRole('option', { name: zone }).click();
}

async function openShifts(panel: Locator): Promise<void> {
  await shiftHeader(panel).click();
  await expect(shiftHeader(panel)).toHaveAttribute('aria-expanded', 'true');
}

async function lineUp(panel: Locator): Promise<void> {
  await openShifts(panel);
  await shiftCamera(cameraShift(panel), 'Unknown camera', '-1:00');
}

async function lineUpOnTouch(page: Page): Promise<void> {
  const panel = page.locator('[data-track-panel]');
  const sheet = page.locator('[data-track-sheet]');
  await expect(panel).toBeVisible();
  if ((await sheet.count()) > 0) {
    await sheet
      .getByRole('button', { name: /^Camera time/ })
      .first()
      .tap();
  } else {
    await shiftHeader(panel).tap();
  }
  await expect(shiftHeader(panel)).toHaveAttribute('aria-expanded', 'true');
  await shiftCamera(cameraShift(panel), 'Unknown camera', '-1:00');
}

async function loadTrack(page: Page): Promise<void> {
  await page.getByLabel('Choose track files').setInputFiles(GPX);
  await expect(page.getByText('Hardangervidda')).toBeVisible();
}

test('a track places a selection after the camera time is lined up', async ({ page }) => {
  const written: Written[] = [];
  await open(page, written);
  await page.goto('/photos');
  for (const name of ['IMG_0003.ARW', 'IMG_0001.ARW', 'IMG_0002.ARW']) {
    const tile = page.locator(`[role="group"][data-name="${name}"]`);
    await tile.hover();
    await tile.getByRole('button', { name: 'Select', exact: true }).click();
  }
  await openBulkLocation(page);
  await page
    .getByRole('group', { name: 'Location' })
    .getByRole('button', { name: 'Match to a track…' })
    .click();

  await expect(page).toHaveURL(/\/map/);
  const panel = page.getByRole('complementary', { name: 'Match track' });
  await expect(panel).toContainText('3 selected photos');
  await loadTrack(page);

  await expect(shiftHeader(panel)).toContainText('No change');
  await expect(panel.getByRole('button', { name: 'Place 0 photos', exact: true })).toBeDisabled();
  await openShifts(panel);
  const shifts = cameraShift(panel);
  await expect(
    shifts.getByRole('combobox', { name: 'Unknown camera time zone', exact: true })
  ).toHaveValue('As shot (Oslo · UTC+1)');
  await shiftCamera(shifts, 'Unknown camera', '-1:00');
  await expect(shiftHeader(panel)).toContainText('−1:00:00');
  await expect(
    shifts.getByRole('textbox', { name: 'Shift Unknown camera by', exact: true })
  ).toHaveValue('−1:00:00');

  const photos = panel.getByRole('list', { name: 'Photos on the track' });
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

test('placing can change the capture time too, and one Undo reverts both', async ({ page }) => {
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
  const fix = panel.getByRole('checkbox', {
    name: 'Save corrected capture time in Immich (2 photos)'
  });
  await expect(panel.getByRole('checkbox', { name: /Save corrected capture time/ })).toHaveCount(0);

  await lineUp(panel);
  await expect(fix).not.toBeChecked();
  await fix.check();
  await panel.getByRole('button', { name: 'Place 2 & save time' }).click();

  await expect(
    page.getByText('Placed 2 photos and changed the capture time of 2 photos')
  ).toBeVisible();
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

function shot(
  asset: ReturnType<typeof numberedAssets>[number],
  at: string,
  model: [string, string]
): ReturnType<typeof numberedAssets>[number] {
  const [make, name] = model;
  const osloWall = new Date(Date.parse(at) + 3_600_000).toISOString();
  return {
    ...asset,
    fileCreatedAt: at,
    localDateTime: osloWall,
    exifInfo: { dateTimeOriginal: at, timeZone: 'Europe/Oslo', make, model: name }
  } as ReturnType<typeof numberedAssets>[number];
}

const SONY: [string, string] = ['SONY', 'ILCE-7M3'];
const PHONE: [string, string] = ['Apple', 'iPhone 15 Pro'];

test('each camera keeps its own time zone and shift', async ({ page }) => {
  const [first, second, third] = numberedAssets(3);
  const shots = [
    shot(first!, '2026-10-31T09:10:00.000Z', SONY),
    shot(second!, '2026-10-31T09:20:00.000Z', SONY),
    shot(third!, '2026-10-31T08:15:00.000Z', PHONE)
  ];
  await installMocks(page, { assets: shots, placeName: BERGEN, metadataPushConsented: true });
  await page.goto('/map');
  await page.getByRole('button', { name: 'Match track' }).click();
  const panel = page.getByRole('complementary', { name: 'Match track' });
  await loadTrack(page);

  await expect(shiftHeader(panel)).toContainText('No change');
  await expect(panel.getByRole('button', { name: 'Place 1 photo', exact: true })).toBeVisible();
  await openShifts(panel);
  const shifts = cameraShift(panel);
  const sony = shifts.getByRole('group', { name: 'ILCE-7M3', exact: true });
  const phone = shifts.getByRole('group', { name: 'iPhone 15 Pro', exact: true });
  const sonyRow = sony.getByRole('button', { name: /^ILCE-7M3,/ });
  await expect(sonyRow).toHaveAccessibleName('ILCE-7M3, 2 photos, No change');
  await expect(shifts.getByRole('textbox')).toHaveCount(0);

  await sonyRow.click();
  await expect(sonyRow).toHaveAttribute('aria-expanded', 'true');
  const zone = sony.getByRole('combobox', { name: 'ILCE-7M3 time zone', exact: true });
  await expect(zone).toHaveValue('As shot (Oslo · UTC+1)');
  await pickZone(page, zone, 'Helsinki', /Europe\/Helsinki/);
  await expect(shiftHeader(panel)).toContainText('ILCE-7M3 Helsinki · 1 unchanged');
  await expect(sonyRow).toHaveAccessibleName('ILCE-7M3, 2 photos, Helsinki');
  await expect(panel.getByRole('button', { name: 'Place 3 photos', exact: true })).toBeVisible();
  await expect(panel.locator('[data-track-photo]')).toContainText([
    'IMG_0001',
    'IMG_0003',
    'IMG_0002'
  ]);

  await phone.getByRole('button', { name: /^iPhone 15 Pro,/ }).click();
  await expect(phone.getByRole('textbox', { name: 'Shift iPhone 15 Pro by' })).toHaveValue(
    '+0:00:00'
  );
  await expect(sony.getByRole('textbox')).toHaveCount(0);
  await expect(sony.getByRole('combobox')).toHaveCount(0);
});

test('a track logged in local time gets its time zone', async ({ page }) => {
  await open(page, []);
  await page.goto('/map');
  await page.getByRole('button', { name: 'Match track' }).click();
  const panel = page.getByRole('complementary', { name: 'Match track' });
  await loadTrack(page);
  await expect(panel.getByRole('button', { name: 'Place 0 photos', exact: true })).toBeDisabled();

  const track = panel.getByRole('region', { name: 'Track', exact: true });
  const header = track.getByRole('button', { name: /Hardangervidda/ });
  await header.click();
  const zone = track.getByRole('combobox', { name: 'Hardangervidda time zone', exact: true });
  await expect(zone).toHaveValue('UTC');
  await pickZone(page, zone, 'Cape Verde', /Atlantic\/Cape Verde/);
  await expect(header).toContainText('Cape Verde');
  await expect(panel.getByRole('button', { name: 'Place 2 photos', exact: true })).toBeEnabled();
  await expect(panel.getByRole('checkbox', { name: /Save corrected capture time/ })).toHaveCount(0);
});

test('photos with a location can get their capture time saved alone', async ({ page }) => {
  const times = new CaptureTimeMock();
  const located = SHOOT.map((asset, index) => ({
    ...asset,
    exifInfo: { ...asset.exifInfo, latitude: 60.11 + index * 0.01, longitude: 6.71 }
  })) as ReturnType<typeof numberedAssets>;
  await installMocks(page, {
    assets: located,
    placeName: BERGEN,
    metadataPushConsented: true,
    captureTime: times
  });
  await page.goto('/map');
  await page.getByRole('button', { name: 'Match track' }).click();
  const panel = page.getByRole('complementary', { name: 'Match track' });
  await loadTrack(page);
  await lineUp(panel);

  const photos = panel.getByRole('list', { name: 'Photos on the track' });
  await expect(photos.locator(`[data-track-photo="${SHOOT[0]!.id}"]`)).toContainText(
    /m from track/
  );
  await expect(panel.getByRole('button', { name: 'Place 0 photos', exact: true })).toBeDisabled();
  await panel
    .getByRole('checkbox', { name: 'Save corrected capture time in Immich (2 photos)' })
    .check();
  await panel.getByRole('button', { name: 'Save time for 2 photos', exact: true }).click();

  await expect(page.getByText('Changed the capture time of 2 photos')).toBeVisible();
  expect(times.calls).toEqual([
    [
      { id: SHOOT[0]!.id, minutes: -60 },
      { id: SHOOT[1]!.id, minutes: -60 }
    ]
  ]);
  await expect(panel).toHaveCount(0);
});

test('typing a shift moves the photos along the track', async ({ page }) => {
  await open(page, []);
  await page.goto('/map');
  await page.getByRole('button', { name: 'Match track' }).click();
  const panel = page.getByRole('complementary', { name: 'Match track' });
  await expect(panel).toContainText('Choose GPX, KML or TCX files');
  await loadTrack(page);
  await openShifts(panel);

  const shift = cameraShift(panel).getByRole('textbox', {
    name: 'Shift Unknown camera by',
    exact: true
  });
  await shift.fill('-0:55');
  await shift.press('Enter');
  await expect(shiftHeader(panel)).toContainText('−0:55:00');
  await panel.getByRole('button', { name: 'One minute earlier' }).click();
  await expect(shift).toHaveValue('−0:56:00');
  await panel.getByRole('button', { name: 'Close track' }).click();
  await expect(panel).toHaveCount(0);
  await expect(page.getByRole('complementary', { name: 'Photos in view' })).toBeVisible();
});

test('a track shift moves the track under the photos', async ({ page }) => {
  await open(page, []);
  await page.goto('/map');
  await page.getByRole('button', { name: 'Match track' }).click();
  const panel = page.getByRole('complementary', { name: 'Match track' });
  await loadTrack(page);
  await expect(panel.getByRole('button', { name: 'Place 0 photos', exact: true })).toBeDisabled();

  const track = panel.getByRole('region', { name: 'Track', exact: true });
  const header = track.getByRole('button', { name: /Hardangervidda/ });
  await header.click();
  const shift = track.getByRole('textbox', { name: 'Shift Hardangervidda by' });
  await shift.fill('1:00');
  await shift.press('Enter');
  await expect(header).toContainText('+1:00:00');
  await expect(panel.getByRole('button', { name: 'Place 2 photos', exact: true })).toBeEnabled();
  await expect(panel.getByRole('list', { name: 'Photos on the track' })).toContainText('10:10:00');
  await expect(panel.getByRole('checkbox', { name: /Save corrected capture time/ })).toHaveCount(0);
});

test('hovering a pin points at its photo and a click opens it large', async ({ page }) => {
  await open(page, []);
  await page.goto('/map');
  await page.getByRole('button', { name: 'Match track' }).click();
  const panel = page.getByRole('complementary', { name: 'Match track' });
  await loadTrack(page);
  await lineUp(panel);
  const photos = panel.getByRole('list', { name: 'Photos on the track' });
  const first = photos.locator(`[data-track-photo="${SHOOT[0]!.id}"]`);
  await expect(first).toContainText('10:10:00 → 09:10:00');

  await expect(page).toHaveURL(/at=60\.1\d+%2C6\.7/);
  const pin = await pointOnMap(page, 60.11, 6.71);
  await page.mouse.move(pin.x, pin.y);
  await expect(first).toHaveAttribute('aria-current', 'true');
  const card = page.locator('[data-pin-preview]');
  await expect(card).toContainText('IMG_0001.ARW');
  await expect(card).toContainText('10:10:00 → 09:10:00');
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

test('photo markers dim under the track and can be hidden', async ({ page }) => {
  await installMocks(page, {
    assets: SHOOT,
    mapMarkers: [{ id: SHOOT[2]!.id, lat: 60.11, lon: 6.71, city: 'Bergen' }],
    placeName: BERGEN,
    metadataPushConsented: true
  });
  await page.goto('/map');
  const toggle = page.getByRole('button', { name: 'Photos on the map', exact: true });
  await expect(toggle).toHaveCount(0);
  await page.getByRole('button', { name: 'Match track' }).click();
  const panel = page.getByRole('complementary', { name: 'Match track' });
  await loadTrack(page);
  await lineUp(panel);
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

  await expect(toggle).toHaveAttribute('aria-pressed', 'true');
  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-pressed', 'false');
  await expect(marker).toHaveCount(0);
  await toggle.click();
  await expect(marker).toBeVisible();
});

test('track and camera time sections slide open and closed', async ({ page }) => {
  await open(page, []);
  await page.goto('/map');
  await page.getByRole('button', { name: 'Match track' }).click();
  const panel = page.getByRole('complementary', { name: 'Match track' });
  await loadTrack(page);
  await page.evaluate(() => {
    const slides: string[] = [];
    Reflect.set(window, '__trackSlides', slides);
    const animate = Element.prototype.animate;
    Element.prototype.animate = function (...args: Parameters<Element['animate']>) {
      const owner = this.closest('[aria-label]')?.getAttribute('aria-label');
      if (this.id && owner) slides.push(owner);
      return animate.apply(this, args);
    };
  });
  const slides = () => page.evaluate(() => Reflect.get(window, '__trackSlides') as string[]);

  await openShifts(panel);
  await expect.poll(slides).toContain('Camera time');
  const track = panel.getByRole('region', { name: 'Track', exact: true });
  await track.getByRole('button', { name: /Hardangervidda/ }).click();
  await expect.poll(slides).toContain('Track');
  const opened = (await slides()).length;

  await shiftHeader(panel).click();
  await expect(shiftHeader(panel)).toHaveAttribute('aria-expanded', 'false');
  await expect.poll(async () => (await slides()).length).toBeGreaterThan(opened);
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

  test('reads its time in the zone of the track', async ({ page }) => {
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
    await openShifts(panel);
    const shifts = cameraShift(panel);
    await expect(
      shifts.getByRole('combobox', { name: 'Unknown camera time zone', exact: true })
    ).toHaveValue('As shot (Oslo · UTC+1)');
    await expect(shiftHeader(panel)).toContainText('No change');
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
    await openTouchLocation(page);
    await page
      .getByRole('dialog', { name: 'Location' })
      .getByRole('button', { name: 'Match to a track…' })
      .tap();
    await expect(page).toHaveURL(/\/map/);
    await loadTrack(page);
    await lineUpOnTouch(page);
    await page.getByRole('button', { name: 'Place 2 photos', exact: true }).tap();
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
    const inStrip = strip.getByRole('link', { name: 'IMG_0002.ARW', exact: true });
    const sheet = page.locator('[data-track-sheet]');
    if ((await sheet.count()) > 0) {
      const resize = page.getByRole('button', { name: 'Resize sheet' });
      await resize.tap();
      await resize.tap();
      await expect(inStrip).toBeVisible();
    }
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

  test('the phone sheet offers the capture time change', async ({ page }, info) => {
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
    const sheet = page.locator('[data-track-sheet]');
    const fix = sheet.getByRole('checkbox', {
      name: 'Save corrected capture time in Immich (2 photos)'
    });
    await fix.tap();
    await expect(fix).toBeChecked();
    await sheet.getByRole('button', { name: 'Place 2 & save time', exact: true }).tap();
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
    await expect(
      sheet.getByRole('link', { name: `Open ${SHOOT[0]!.originalFileName}` })
    ).toBeInViewport();
    await resize.tap();
    await expect.poll(height).toBeLessThan(140);
    await expect(sheet.getByRole('heading', { name: /photos on the track/ })).toBeInViewport();
    await resize.tap();
    await expect.poll(height).toBeGreaterThan(peek - 4);
    await expect(sheet.getByRole('list', { name: 'Photos on the track' })).toBeInViewport();
  });

  test('a tap on the map shrinks the large phone sheet', async ({ page }, info) => {
    test.skip(info.project.name !== 'touch-phone', 'phone sheet');
    await open(page, []);
    await page.goto('/map');
    await page.getByRole('button', { name: 'Match track' }).tap();
    await loadTrack(page);
    const sheet = page.locator('[data-track-sheet]');
    const height = async (): Promise<number> => (await sheet.boundingBox())!.height;
    const peek = await height();
    await page.getByRole('button', { name: 'Resize sheet' }).tap();
    await expect.poll(height).toBeGreaterThan(peek + 100);
    await page.getByRole('heading', { name: 'Map', level: 1 }).tap();
    await expect.poll(height).toBeLessThan(peek + 4);
    await expect(sheet.getByRole('heading', { name: /photos on the track/ })).toBeVisible();
  });

  test('a tapped dot previews its photo and a second tap opens it', async ({ page }, info) => {
    test.skip(info.project.name !== 'touch-tablet', 'tablet panel');
    await open(page, []);
    await page.goto('/map');
    await page.getByRole('button', { name: 'Match track' }).tap();
    await loadTrack(page);
    await lineUpOnTouch(page);
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
