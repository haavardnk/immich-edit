import { expect, test, type Page } from '@playwright/test';
import { bulkBar, pick } from './browse';
import { installMocks } from './helpers';
import { ASSETS, BERGEN, chooseOnMap, type LocationWrite } from './mapFixtures';
import { PlacesMock, savedPlace } from './placesMocks';
import { selectTwo } from './touchShell';

const HYTTA = savedPlace(
  'place-hytta',
  'Hytta',
  { lat: 60.6289, lon: 6.4153, city: 'Voss', state: 'Vestland', country: 'Norway' },
  30
);
const STUDIO = savedPlace(
  'place-studio',
  'Studio',
  { lat: 59.923, lon: 10.759, city: 'Oslo', state: 'Oslo', country: 'Norway' },
  10
);
const FLAM = {
  id: 'recent-flam',
  lat: 60.8628,
  lon: 7.1135,
  city: 'Flåm',
  state: 'Vestland',
  country: 'Norway',
  used_at: '2026-11-20T10:00:00.000Z'
};

async function open(page: Page, places: PlacesMock, writes: LocationWrite[]): Promise<void> {
  await installMocks(page, {
    assets: ASSETS,
    placeName: BERGEN,
    metadataPushConsented: true,
    places,
    onLocation: (body) => writes.push(body)
  });
}

test('the saved places menu and typed search both place photos', async ({ page }) => {
  const places = new PlacesMock({ saved: [STUDIO, HYTTA], recent: [FLAM] });
  const writes: LocationWrite[] = [];
  await open(page, places, writes);
  await page.goto('/photos');
  await pick(page, 'IMG_0005.ARW');
  await bulkBar(page).getByRole('button', { name: 'Location', exact: true }).click();
  await chooseOnMap(page);

  const dialog = page.getByRole('dialog', { name: 'Set location' });
  const search = dialog.getByRole('textbox', { name: 'Search for a place' });
  const saved = dialog.getByRole('list', { name: 'Saved places' });
  await search.focus();
  await expect(saved).toHaveCount(0);

  await dialog.getByRole('button', { name: 'Saved places' }).click();
  const menu = page.getByRole('group', { name: 'Saved places' });
  await expect(menu.getByRole('button')).toHaveText([/Hytta/, /Studio/]);
  await expect(page.getByRole('group', { name: 'Recent places' })).toContainText('Flåm');
  await menu.getByRole('button', { name: /Studio/ }).click();
  await expect(menu).toHaveCount(0);
  await expect(search).toHaveValue('Studio');
  await search.focus();
  await expect(saved).toHaveCount(0);
  await expect(dialog.getByRole('button', { name: 'Saved place', exact: true })).toHaveAttribute(
    'aria-pressed',
    'true'
  );

  await search.fill('vos');
  const rows = saved.getByRole('button').filter({ hasText: /\w/ });
  await expect(rows).toHaveText([/Hytta.*Voss/]);
  await rows.filter({ hasText: 'Hytta' }).click();
  await expect(dialog.getByRole('button', { name: 'Saved place', exact: true })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
  await dialog.getByRole('button', { name: 'Save', exact: true }).click();

  await expect(dialog).toHaveCount(0);
  expect(writes).toEqual([{ ids: [ASSETS[4]!.id], latitude: 60.6289, longitude: 6.4153 }]);
  await expect
    .poll(() => places.saved.find((p) => p.id === HYTTA.id)?.used_at)
    .not.toBe(HYTTA.used_at);
});

test('the star in the picker saves the spot as a named place', async ({ page }) => {
  const places = new PlacesMock();
  const writes: LocationWrite[] = [];
  await open(page, places, writes);
  await page.goto('/photos');
  await pick(page, 'IMG_0005.ARW');
  await bulkBar(page).getByRole('button', { name: 'Location', exact: true }).click();
  await chooseOnMap(page);

  const dialog = page.getByRole('dialog', { name: 'Set location' });
  await dialog.getByRole('textbox', { name: 'Search for a place' }).fill('60.3975, 5.324');
  await dialog.getByRole('textbox', { name: 'Search for a place' }).press('Enter');
  await expect(dialog.getByRole('status').filter({ hasText: 'Bergen' })).toBeVisible();
  await dialog.getByRole('button', { name: 'Save place' }).click();
  const name = page.getByRole('textbox', { name: 'Name' });
  await expect(name).toHaveValue('Bergen');
  await name.fill('Bryggen');
  await name.press('Enter');

  await expect(page.getByText('Saved Bryggen', { exact: true })).toBeVisible();
  await expect(dialog.getByRole('button', { name: 'Saved place', exact: true })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
  expect(places.saved).toMatchObject([
    { name: 'Bryggen', lat: 60.3975, lon: 5.324, city: 'Bergen' }
  ]);
});

test('the Location menu places a selection at a saved place in one click', async ({ page }) => {
  const places = new PlacesMock({ saved: [HYTTA, STUDIO] });
  const writes: LocationWrite[] = [];
  await open(page, places, writes);
  await page.goto('/photos');
  await pick(page, 'IMG_0004.ARW');
  await pick(page, 'IMG_0005.ARW');
  await bulkBar(page).getByRole('button', { name: 'Location', exact: true }).click();
  const saved = page.getByRole('group', { name: 'Saved places' });
  await expect(saved.getByRole('button')).toHaveText([/Hytta\s*Voss/, /Studio\s*Oslo/]);
  await saved.getByRole('button', { name: 'Studio' }).click();

  await expect(page.getByText('Location set for 2 photos')).toBeVisible();
  expect(writes).toHaveLength(1);
  expect(writes[0]).toMatchObject({ latitude: 59.923, longitude: 10.759 });
});

test('a saved place moves to the pin, is renamed and removed from the picker', async ({ page }) => {
  const places = new PlacesMock({ saved: [HYTTA, STUDIO] });
  await open(page, places, []);
  await page.goto('/photos');
  await pick(page, 'IMG_0005.ARW');
  await bulkBar(page).getByRole('button', { name: 'Location', exact: true }).click();
  await chooseOnMap(page);
  const dialog = page.getByRole('dialog', { name: 'Set location' });
  const search = dialog.getByRole('textbox', { name: 'Search for a place' });
  await search.fill('59.9, 10.7');
  await search.press('Enter');
  await search.fill('stu');
  const saved = dialog.getByRole('list', { name: 'Saved places' });
  await expect(saved).not.toContainText('Manage');

  await saved.getByRole('button', { name: 'Actions for Studio' }).click();
  await page.getByRole('button', { name: 'Move Studio to the pin' }).click();
  await expect.poll(() => places.saved.find((p) => p.id === STUDIO.id)?.lat).toBe(59.9);
  expect(places.saved.find((p) => p.id === STUDIO.id)?.lon).toBe(10.7);

  await search.fill('hyt');
  await saved.getByRole('button', { name: 'Actions for Hytta' }).click();
  await page.getByRole('button', { name: 'Rename', exact: true }).click();
  const field = saved.getByRole('textbox', { name: 'Name of Hytta' });
  await field.fill('Hytta på Voss');
  await field.press('Enter');
  await expect.poll(() => places.saved.find((p) => p.id === HYTTA.id)?.name).toBe('Hytta på Voss');

  await search.focus();
  await saved.getByRole('button', { name: 'Actions for Hytta på Voss' }).click();
  await page.getByRole('button', { name: 'Remove', exact: true }).click();
  await expect.poll(() => places.saved.map((p) => p.name)).toEqual(['Studio']);
});

test('Settings no longer lists saved places', async ({ page }) => {
  await open(page, new PlacesMock({ saved: [HYTTA] }), []);
  await page.goto('/settings/account');
  await expect(page.getByRole('heading', { name: 'Account' })).toBeVisible();
  await expect(page.locator('[data-setting-anchor="saved-places"]')).toHaveCount(0);
  await expect(page.getByText('Saved places')).toHaveCount(0);
});

test.describe('saved places on touch', { tag: '@touch' }, () => {
  test('the Location sheet places a selection at a saved place', async ({ page }) => {
    const places = new PlacesMock({ saved: [HYTTA], recent: [FLAM] });
    const writes: LocationWrite[] = [];
    await open(page, places, writes);
    await page.goto('/photos');
    await selectTwo(page);
    await bulkBar(page).getByRole('button', { name: 'More', exact: true }).tap();
    await page
      .getByRole('dialog', { name: 'More actions' })
      .getByRole('button', { name: 'Location' })
      .tap();
    const sheet = page.getByRole('dialog', { name: 'Location' });
    await expect(sheet.getByRole('group', { name: 'Recent' })).toContainText('Flåm');
    await sheet
      .getByRole('group', { name: 'Saved places' })
      .getByRole('button', { name: 'Hytta' })
      .tap();

    await expect.poll(() => writes.length).toBe(1);
    expect(writes[0]).toMatchObject({ latitude: 60.6289, longitude: 6.4153 });
  });

  test('the picker offers saved places without opening the keyboard', async ({ page }) => {
    const places = new PlacesMock({ saved: [HYTTA, STUDIO] });
    await open(page, places, []);
    await page.goto('/photos');
    await selectTwo(page);
    await bulkBar(page).getByRole('button', { name: 'More', exact: true }).tap();
    await page
      .getByRole('dialog', { name: 'More actions' })
      .getByRole('button', { name: 'Location' })
      .tap();
    await page
      .getByRole('dialog', { name: 'Location' })
      .getByRole('button', { name: 'Choose on map' })
      .tap();
    const dialog = page.getByRole('dialog', { name: /Set location/ });
    await dialog.getByRole('button', { name: 'Saved places' }).tap();
    const menu = page.getByRole('group', { name: 'Saved places' });
    await menu.getByRole('button', { name: /Hytta/ }).tap();
    await expect(dialog.getByRole('textbox', { name: 'Search for a place' })).not.toBeFocused();
    await expect(dialog.getByRole('button', { name: 'Saved place', exact: true })).toHaveAttribute(
      'aria-pressed',
      'true'
    );
  });
});
