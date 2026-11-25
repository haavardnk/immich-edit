import { expect, test, type Page } from '@playwright/test';
import { bulkBar, pick } from './browse';
import { installMocks } from './helpers';
import { BERGEN, type LocationWrite, PLACED } from './mapFixtures';
import { selectTwo } from './touchShell';

const COPIED = { lat: 60.3975, lon: 5.324, place: BERGEN };

async function openPhotos(page: Page, writes: LocationWrite[]): Promise<void> {
  await installMocks(page, {
    assets: PLACED,
    placeName: BERGEN,
    metadataPushConsented: true,
    onLocation: (body) => writes.push(body)
  });
  await page.goto('/photos');
}

async function clearSelection(page: Page): Promise<void> {
  await bulkBar(page).getByRole('button', { name: 'Clear selection' }).click();
  await expect(bulkBar(page)).toHaveCount(0);
}

function locationMenu(page: Page) {
  return page.getByRole('group', { name: 'Location' });
}

test('a location copied from one photo pastes onto a selection', async ({ page }) => {
  const writes: LocationWrite[] = [];
  await openPhotos(page, writes);
  await pick(page, 'IMG_0002.ARW');
  await bulkBar(page).getByRole('button', { name: 'Location', exact: true }).click();
  await expect(locationMenu(page).getByRole('button', { name: 'Paste location' })).toBeDisabled();
  await expect(locationMenu(page)).toContainText('Copy a location first');
  await expect(locationMenu(page)).toContainText('This photo has no location');
  await page.keyboard.press('Escape');
  await clearSelection(page);

  await pick(page, 'IMG_0001.ARW');
  await page.keyboard.press('Alt+Shift+C');
  await expect(page.getByText('Copied Bergen')).toBeVisible();
  await clearSelection(page);

  await pick(page, 'IMG_0002.ARW');
  await pick(page, 'IMG_0004.ARW');
  await bulkBar(page).getByRole('button', { name: 'Location', exact: true }).click();
  const paste = locationMenu(page).getByRole('button', { name: 'Paste location' });
  await expect(paste).toContainText('Bergen');
  await paste.click();

  await expect(page.getByText('Location set for 2 photos')).toBeVisible();
  expect(writes).toHaveLength(1);
  expect(writes[0]!.ids.sort()).toEqual([PLACED[1]!.id, PLACED[3]!.id].sort());
  expect(writes[0]).toMatchObject({ latitude: 60.3975, longitude: 5.324 });

  await page.keyboard.press('Alt+Shift+V');
  await expect.poll(() => writes.length).toBe(2);
});

test.describe('location on touch', { tag: '@touch' }, () => {
  test('the More sheet pastes the location copied earlier in the session', async ({ page }) => {
    const writes: LocationWrite[] = [];
    await page.addInitScript((copied) => {
      sessionStorage.setItem('immich-edit:copiedLocation', JSON.stringify(copied));
    }, COPIED);
    await openPhotos(page, writes);
    await selectTwo(page);
    await bulkBar(page).getByRole('button', { name: 'More', exact: true }).tap();
    await page
      .getByRole('dialog', { name: 'More actions' })
      .getByRole('button', { name: 'Location' })
      .tap();
    const sheet = page.getByRole('dialog', { name: 'Location' });
    await expect(sheet.getByRole('button', { name: 'Copy location' })).toBeDisabled();
    const paste = sheet.getByRole('button', { name: 'Paste location' });
    await expect(paste).toContainText('Bergen');
    await paste.tap();

    await expect(sheet).toHaveCount(0);
    await expect.poll(() => writes.length).toBe(1);
    expect(writes[0]!.ids.sort()).toEqual([PLACED[0]!.id, PLACED[1]!.id].sort());
  });
});
