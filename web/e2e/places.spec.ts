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

test('saved and recent places are offered before typing and place photos', async ({ page }) => {
  const places = new PlacesMock({ saved: [STUDIO, HYTTA], recent: [FLAM] });
  const writes: LocationWrite[] = [];
  await open(page, places, writes);
  await page.goto('/photos');
  await pick(page, 'IMG_0005.ARW');
  await bulkBar(page).getByRole('button', { name: 'Location', exact: true }).click();
  await chooseOnMap(page);

  const dialog = page.getByRole('dialog', { name: 'Set location' });
  await dialog.getByRole('textbox', { name: 'Search for a place' }).focus();
  const saved = dialog.getByRole('list', { name: 'Saved places' });
  await expect(saved.getByRole('button')).toHaveText([/Hytta/, /Studio/]);
  await expect(dialog.getByRole('list', { name: 'Recent places' })).toContainText('Flåm');

  await dialog.getByRole('textbox', { name: 'Search for a place' }).fill('vos');
  await expect(saved.getByRole('button')).toHaveText([/Hytta.*Voss/]);
  await saved.getByRole('button', { name: /Hytta/ }).click();
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

  await expect(page.getByText('Saved Bryggen')).toBeVisible();
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
});
