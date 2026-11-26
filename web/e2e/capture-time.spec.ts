import { expect, test, type Locator, type Page } from '@playwright/test';
import { settleAnimations } from './animations';
import { bulkBar, openBulkMetadata, openTouchMetadata, pick, tile } from './browse';
import { CaptureTimeMock } from './captureTimeMocks';
import { installMocks, numberedAssets } from './helpers';
import { openTileLoupe } from './touch';
import { selectTwo } from './touchShell';

test.use({ timezoneId: 'Europe/Oslo' });

const SONY = { make: 'Sony', model: 'ILCE-7M3' };
const PHONE = { make: 'Apple', model: 'iPhone 15 Pro' };

const PHOTOS = numberedAssets(4).map((asset, index) => {
  const phone = index % 2 === 1;
  const minute = 3 + index;
  const local = `2026-10-31T13:0${minute}:12.000Z`;
  return {
    ...asset,
    localDateTime: phone ? `2026-10-31T14:0${minute}:12.000Z` : local,
    fileCreatedAt: local,
    exifInfo: {
      ...(phone ? PHONE : SONY),
      dateTimeOriginal: phone ? `2026-10-31T13:0${minute}:12.000+00:00` : local,
      timeZone: phone ? 'UTC+1' : null
    }
  };
});

const [SONY_1, PHONE_1, SONY_2] = PHOTOS;

async function open(page: Page, times: CaptureTimeMock): Promise<void> {
  await installMocks(page, { assets: PHOTOS, captureTime: times, metadataPushConsented: true });
  await page.goto('/photos');
  await expect(tile(page, 'IMG_0001.ARW')).toBeVisible();
}

async function selectAll(page: Page): Promise<void> {
  for (const asset of PHOTOS) await pick(page, asset.originalFileName);
  await expect(bulkBar(page)).toContainText('4 selected');
}

async function openEditor(page: Page) {
  const menu = await openBulkMetadata(page);
  await menu.getByRole('button', { name: 'Capture time…' }).click();
  const dialog = page.getByRole('dialog', { name: 'Capture time' });
  await expect(dialog).toBeVisible();
  return dialog;
}

async function chooseCamera(page: Page, dialog: Locator, from: string, to: RegExp) {
  await dialog.getByRole('button', { name: `Camera: ${from}` }).click();
  await page.getByRole('button', { name: to }).click();
}

async function pickZone(page: Page, dialog: Locator, place: string, zone: RegExp) {
  const field = dialog.getByRole('combobox', { name: 'Time zone' });
  await field.click();
  await field.pressSequentially(place);
  await page.getByRole('option', { name: zone }).click();
}

async function expectNoScroll(dialog: Locator): Promise<void> {
  const change = dialog.getByRole('region', { name: 'Change' });
  expect(
    await change.evaluate((node) => node.scrollHeight - node.clientHeight)
  ).toBeLessThanOrEqual(0);
}

async function expectOneSize(
  page: Page,
  dialog: Locator,
  choose: (chip: Locator) => Promise<void>
): Promise<void> {
  const modes = dialog.getByRole('radiogroup', { name: 'How to change the capture time' });
  const chips = await modes.getByRole('radio').all();
  expect(chips).toHaveLength(3);
  await settleAnimations(page);
  const first = (await dialog.boundingBox())!;
  for (const chip of chips) {
    await choose(chip);
    await expect(chip).toBeChecked();
    await settleAnimations(page);
    const box = (await dialog.boundingBox())!;
    expect([box.y, box.width, box.height]).toEqual([first.y, first.width, first.height]);
    await expectNoScroll(dialog);
  }
}

test('shifting one camera by seconds writes set times and undoes them', async ({ page }) => {
  const times = new CaptureTimeMock();
  await open(page, times);
  await selectAll(page);
  const dialog = await openEditor(page);
  await expect(dialog).toContainText('4 photos · 2 cameras');

  await dialog.getByRole('button', { name: 'Camera: All cameras' }).click();
  await page.getByRole('button', { name: /^ILCE-7M3/ }).click();
  const shift = dialog.getByRole('textbox', { name: 'Shift by' });
  await shift.fill('+1:02:37');
  await shift.press('Enter');

  await expect(dialog.getByRole('button', { name: 'Changed 2' })).toBeVisible();
  await expect(dialog.locator(`[data-capture-row="${SONY_1!.id}"]`)).toContainText('14:05:49');
  await dialog.getByRole('button', { name: 'Apply to 2 photos' }).click();

  await expect(page.getByText('Changed the capture time of 2 photos')).toBeVisible();
  expect(times.calls).toHaveLength(1);
  expect(times.calls[0]).toEqual([
    { id: SONY_1!.id, at: '2026-10-31T13:05:49.000Z', timeZone: 'Europe/Oslo' },
    { id: SONY_2!.id, at: '2026-10-31T13:07:49.000Z', timeZone: 'Europe/Oslo' }
  ]);

  await page.getByRole('button', { name: 'Undo' }).click();
  await expect.poll(() => times.calls.length).toBe(2);
  expect(times.calls[1]!.map((item) => item.id)).toEqual([SONY_1!.id, SONY_2!.id]);
});

test('a whole-minute shift of a zoned camera is sent as minutes', async ({ page }) => {
  const times = new CaptureTimeMock();
  await open(page, times);
  await selectAll(page);
  const dialog = await openEditor(page);
  await dialog.getByRole('button', { name: 'Camera: All cameras' }).click();
  await page.getByRole('button', { name: /^iPhone 15 Pro/ }).click();
  await dialog.getByRole('button', { name: 'One hour later' }).click();
  await dialog.getByRole('button', { name: 'Apply to 2 photos' }).click();
  await expect.poll(() => times.calls.length).toBe(1);
  expect(times.calls[0]).toEqual([
    { id: PHONE_1!.id, minutes: 60 },
    { id: PHOTOS[3]!.id, minutes: 60 }
  ]);
});

test('a camera matched to a photo from another camera lines up with it', async ({ page }) => {
  const times = new CaptureTimeMock();
  await open(page, times);
  await selectAll(page);
  const dialog = await openEditor(page);
  await dialog.getByRole('radio', { name: 'Set by a photo' }).click();
  await dialog.getByRole('button', { name: 'Camera: All cameras' }).click();
  await page.getByRole('button', { name: /^ILCE-7M3/ }).click();
  const reference = dialog.getByRole('button', {
    name: `Match to ${PHONE_1!.originalFileName} from iPhone 15 Pro`
  });
  await expect(reference).toContainText('iPhone 15 Pro');
  await reference.click();

  await expect(
    dialog.getByRole('paragraph').filter({ hasText: 'ILCE-7M3 +1:01:00' })
  ).toContainText(`Matched to ${PHONE_1!.originalFileName} from iPhone 15 Pro`);
  await dialog.getByRole('button', { name: 'Apply to 2 photos' }).click();
  await expect.poll(() => times.calls.length).toBe(1);
  expect(times.calls[0]!.map((item) => item.id)).toEqual([SONY_1!.id, SONY_2!.id]);
});

test('reference photos show their time in the picked photo zone', async ({ page }) => {
  await open(page, new CaptureTimeMock());
  await selectAll(page);
  const dialog = await openEditor(page);
  await dialog.getByRole('radio', { name: 'Set by a photo' }).click();
  await chooseCamera(page, dialog, 'All cameras', /^ILCE-7M3/);
  const reference = dialog.getByRole('button', {
    name: `Match to ${PHONE_1!.originalFileName} from iPhone 15 Pro`
  });
  await expect(reference).toContainText('14:04:12');
  await pickZone(page, dialog, 'Bangkok', /Asia\/Bangkok/);
  await expect(reference).toContainText('20:04:12');
});

test('the picked reference photo keeps its whole ring inside the strip', async ({ page }) => {
  await open(page, new CaptureTimeMock());
  await selectAll(page);
  const dialog = await openEditor(page);
  await dialog.getByRole('radio', { name: 'Set by a photo' }).click();
  await dialog.getByRole('button', { name: 'Camera: All cameras' }).click();
  await page.getByRole('button', { name: /^ILCE-7M3/ }).click();
  const name = `Match to ${PHONE_1!.originalFileName} from iPhone 15 Pro`;
  const reference = dialog.getByRole('button', { name });
  await reference.click();
  await expect(reference).toHaveAttribute('aria-pressed', 'true');
  await settleAnimations(page);

  const strip = (await dialog
    .getByRole('group', { name: 'Photos from other cameras' })
    .boundingBox())!;
  const box = (await reference.boundingBox())!;
  expect(box.y).toBeGreaterThanOrEqual(strip.y);
  expect(box.x).toBeGreaterThanOrEqual(strip.x);
  const style = await reference.evaluate((node) => {
    const computed = getComputedStyle(node);
    return { shadow: computed.boxShadow, offset: computed.outlineOffset };
  });
  expect(style.shadow).toContain('inset');
  expect(parseFloat(style.offset)).toBeLessThan(0);
});

test('one time for all spaces the photos a second apart by file name', async ({ page }) => {
  const times = new CaptureTimeMock();
  await open(page, times);
  await selectAll(page);
  const dialog = await openEditor(page);
  await dialog.getByRole('radio', { name: 'One time for all' }).click();
  await dialog.getByRole('button', { name: 'Apply to 4 photos' }).click();
  await expect.poll(() => times.calls.length).toBe(1);
  const sets = times.calls[0]!;
  expect(sets.map((item) => item.id)).toEqual(PHOTOS.map((asset) => asset.id));
  const stamps = sets.map((item) => Date.parse(item.at ?? ''));
  expect(stamps.slice(1).map((stamp, index) => stamp - stamps[index]!)).toEqual([1000, 1000, 1000]);
  expect(new Set(sets.map((item) => item.timeZone))).toEqual(new Set(['Europe/Oslo']));
});

test('a preview row picks the photo to set by and its camera', async ({ page }) => {
  await open(page, new CaptureTimeMock());
  await selectAll(page);
  const dialog = await openEditor(page);
  await dialog.getByRole('radio', { name: 'Set by a photo' }).click();
  await dialog.getByRole('button', { name: 'Camera: All cameras' }).click();
  await page.getByRole('button', { name: /^ILCE-7M3/ }).click();
  const preview = dialog.getByRole('region', { name: 'Preview' });
  const row = preview.getByRole('button', { name: `Pick ${PHONE_1!.originalFileName}` });
  await row.click();

  await expect(row).toHaveAttribute('aria-pressed', 'true');
  await expect(dialog.getByRole('button', { name: 'Camera: iPhone 15 Pro' })).toBeVisible();
  await expect(dialog.getByRole('img', { name: PHONE_1!.originalFileName })).toBeVisible();
});

test('a time zone picked for one camera leaves the other camera alone', async ({ page }) => {
  const times = new CaptureTimeMock();
  await open(page, times);
  await selectAll(page);
  const dialog = await openEditor(page);
  const zone = dialog.getByRole('combobox', { name: 'Time zone' });
  await chooseCamera(page, dialog, 'All cameras', /^iPhone 15 Pro/);
  await expect(zone).toHaveValue('As shot (UTC+1)');
  await chooseCamera(page, dialog, 'iPhone 15 Pro', /^ILCE-7M3/);
  await expect(zone).toHaveValue('As shot (Oslo · UTC+1)');
  await pickZone(page, dialog, 'Bangkok', /Asia\/Bangkok/);
  await chooseCamera(page, dialog, 'ILCE-7M3', /^iPhone 15 Pro/);
  await expect(zone).toHaveValue('As shot (UTC+1)');
  await chooseCamera(page, dialog, 'iPhone 15 Pro', /^All cameras/);
  await expect(zone).toHaveValue('Mixed');
  await dialog.getByRole('button', { name: 'Apply to 2 photos' }).click();
  await expect.poll(() => times.calls.length).toBe(1);
  expect(times.calls[0]!.map((item) => item.id).sort()).toEqual([SONY_1!.id, SONY_2!.id].sort());
  expect(new Set(times.calls[0]!.map((item) => item.timeZone))).toEqual(new Set(['Asia/Bangkok']));
});

test('one time for all sets every camera and offers no camera', async ({ page }) => {
  const times = new CaptureTimeMock();
  await open(page, times);
  await selectAll(page);
  const dialog = await openEditor(page);
  await chooseCamera(page, dialog, 'All cameras', /^ILCE-7M3/);
  await dialog.getByRole('radio', { name: 'One time for all' }).click();
  await expect(dialog.getByRole('button', { name: /^Camera:/ })).toHaveCount(0);
  await dialog.getByRole('button', { name: 'Apply to 4 photos' }).click();
  await expect.poll(() => times.calls.length).toBe(1);
  expect(times.calls[0]).toHaveLength(4);
});

test('set by a photo with a camera and zone fits the dialog without scrolling', async ({
  page
}) => {
  await open(page, new CaptureTimeMock());
  await selectAll(page);
  const dialog = await openEditor(page);
  await settleAnimations(page);
  const first = (await dialog.boundingBox())!;
  await dialog.getByRole('radio', { name: 'Set by a photo' }).click();
  await chooseCamera(page, dialog, 'All cameras', /^ILCE-7M3/);
  await expect(dialog.getByRole('group', { name: 'Photos from other cameras' })).toBeVisible();
  await pickZone(page, dialog, 'Bangkok', /Asia\/Bangkok/);
  await expect(dialog.getByRole('combobox', { name: 'Camera time zone' })).toBeVisible();
  await settleAnimations(page);
  await expectNoScroll(dialog);
  const box = (await dialog.boundingBox())!;
  expect([box.y, box.width, box.height]).toEqual([first.y, first.width, first.height]);
  const photo = dialog.getByRole('img', { name: SONY_1!.originalFileName });
  expect((await photo.boundingBox())!.height).toBeGreaterThanOrEqual(80);
});

test('one photo offers only the modes that fit one photo', async ({ page }) => {
  await open(page, new CaptureTimeMock());
  await pick(page, SONY_1!.originalFileName);
  const dialog = await openEditor(page);
  const modes = dialog.getByRole('radiogroup', { name: 'How to change the capture time' });
  await expect(modes.getByRole('radio')).toHaveText(['Shift']);
  await settleAnimations(page);
  const change = (await dialog.getByRole('region', { name: 'Change' }).boundingBox())!;
  for (const field of [
    dialog.getByRole('textbox', { name: 'Shift by' }),
    dialog.getByRole('combobox', { name: 'Time zone' })
  ]) {
    const box = (await field.boundingBox())!;
    expect(box.x + box.width).toBeLessThanOrEqual(change.x + change.width);
  }
});

test('switching modes keeps the dialog one size', async ({ page }) => {
  await open(page, new CaptureTimeMock());
  await selectAll(page);
  const dialog = await openEditor(page);
  await expectOneSize(page, dialog, (chip) => chip.click());
});

test('the modes share one row and the shift and zone fields line up', async ({ page }) => {
  await open(page, new CaptureTimeMock());
  await selectAll(page);
  const dialog = await openEditor(page);
  await settleAnimations(page);
  const modes = dialog.getByRole('radiogroup', { name: 'How to change the capture time' });
  const chip = (await modes.getByRole('radio', { name: 'Shift' }).boundingBox())!;
  expect((await modes.boundingBox())!.height).toBeCloseTo(chip.height, 0);
  const shiftLabel = (await dialog.getByText('Shift by', { exact: true }).boundingBox())!;
  const zoneLabel = (await dialog.getByText('Time zone', { exact: true }).boundingBox())!;
  expect(zoneLabel.width).toBeCloseTo(shiftLabel.width, 0);
  const shift = (await dialog.getByRole('textbox', { name: 'Shift by' }).boundingBox())!;
  const zone = dialog.getByRole('combobox', { name: 'Time zone' });
  await expect(zone).toHaveValue('As shot (UTC+1)');
  expect((await zone.boundingBox())!.height).toBeCloseTo(shift.height, 0);
});

test('the info panel moves one photo to another zone and keeps its shown time', async ({
  page
}) => {
  const times = new CaptureTimeMock();
  await open(page, times);
  await openTileLoupe(page, tile(page, 'IMG_0002.ARW'));
  await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();
  await page.keyboard.press('i');
  const panel = page.getByRole('complementary', { name: 'Info' });
  await panel.getByRole('button', { name: /^Taken / }).click();
  const zone = page.getByRole('combobox', { name: 'Time zone' });
  await expect(zone).toHaveValue(/UTC\+1/);
  await zone.click();
  await zone.pressSequentially('Bangkok');
  await page.getByRole('option', { name: /Asia\/Bangkok/ }).click();
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.getByText('Changed the capture time of 1 photo')).toBeVisible();
  expect(times.calls[0]).toEqual([{ id: PHONE_1!.id, minutes: -360, timeZone: 'Asia/Bangkok' }]);
});

test.describe('capture time on touch', { tag: '@touch' }, () => {
  test('the Metadata sheet opens the editor and applies a shift', async ({ page }) => {
    const times = new CaptureTimeMock();
    await installMocks(page, { assets: PHOTOS, captureTime: times, metadataPushConsented: true });
    await page.goto('/photos');
    await selectTwo(page);
    const sheet = await openTouchMetadata(page);
    await sheet.getByRole('button', { name: 'Capture time…' }).tap();
    const editor = page.getByRole('dialog', { name: 'Capture time' });
    await expect(editor).toBeVisible();
    await editor.getByRole('button', { name: 'One hour later' }).tap();
    await editor.getByRole('button', { name: /^Apply to/ }).tap();
    await expect.poll(() => times.calls.length).toBe(1);
  });

  test('switching modes keeps the editor one size', async ({ page }) => {
    await installMocks(page, {
      assets: PHOTOS,
      captureTime: new CaptureTimeMock(),
      metadataPushConsented: true
    });
    await page.goto('/photos');
    await selectTwo(page);
    const sheet = await openTouchMetadata(page);
    await sheet.getByRole('button', { name: 'Capture time…' }).tap();
    const editor = page.getByRole('dialog', { name: 'Capture time' });
    await expect(editor).toBeVisible();
    await expectOneSize(page, editor, (chip) => chip.tap());
  });
});
