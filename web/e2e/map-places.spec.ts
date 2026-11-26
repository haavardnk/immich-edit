import { expect, test, type Page } from '@playwright/test';
import { type LocationWrite, PLACED, openPlacing } from './mapFixtures';
import { PlacesMock, savedPlace } from './placesMocks';
import { drag, longPress } from './touch';

const HYTTA = savedPlace('place-hytta', 'Hytta', {
  lat: 60.6289,
  lon: 6.4153,
  city: 'Voss',
  state: 'Vestland',
  country: 'Norway'
});

function star(page: Page) {
  return page.locator(`[data-map-place="${HYTTA.id}"]`);
}

test('saved places show as stars that a dropped photo snaps to', async ({ page }) => {
  const writes: LocationWrite[] = [];
  await openPlacing(page, writes, new PlacesMock({ saved: [HYTTA] }));
  await expect(star(page)).toBeVisible();
  await expect(star(page)).toHaveAccessibleName('Saved place Hytta');

  const toggle = page.getByRole('button', { name: 'Saved places' });
  await expect(toggle).toHaveAttribute('aria-pressed', 'true');
  await toggle.click();
  await expect(star(page)).toHaveCount(0);
  await toggle.click();
  await expect(star(page)).toBeVisible();

  const panel = page.getByRole('complementary', { name: 'Photos in view' });
  await panel.getByRole('radio', { name: 'No location' }).click();
  const tile = panel.locator('[role="group"][data-name="IMG_0002.ARW"]');
  const box = (await star(page).boundingBox())!;
  await tile.hover();
  await page.mouse.down();
  await page.mouse.move(box.x + 14 + 18, box.y + box.height / 2 + 12, { steps: 8 });
  await expect(star(page)).toHaveAttribute('data-hit', 'true');
  await page.mouse.up();

  const card = page.locator('[data-drop-confirm]');
  await expect(card).toContainText('Place this photo at Hytta?');
  await expect(card).toContainText('Voss, Vestland, Norway · saved place');
  await card.getByRole('button', { name: 'Place', exact: true }).click();
  await expect.poll(() => writes.length).toBe(1);
  expect(writes[0]).toEqual({ ids: [PLACED[1]!.id], latitude: 60.6289, longitude: 6.4153 });
});

test('a star opens a card that copies its location or places the selection', async ({ page }) => {
  const writes: LocationWrite[] = [];
  await openPlacing(page, writes, new PlacesMock({ saved: [HYTTA] }));
  await star(page).click();
  const card = page.getByRole('dialog', { name: 'Hytta' });
  await expect(card).toContainText('Voss, Vestland, Norway');
  await expect(card.getByRole('button', { name: /^Place/ })).toHaveCount(0);
  await card.getByRole('button', { name: 'Copy location' }).click();
  await expect(page.getByText('Copied Hytta')).toBeVisible();
  await expect(card).toHaveCount(0);

  const panel = page.getByRole('complementary', { name: 'Photos in view' });
  await panel.getByRole('radio', { name: 'No location' }).click();
  const tile = panel.locator('[role="group"][data-name="IMG_0004.ARW"]');
  await tile.hover();
  await tile.getByRole('button', { name: 'Select', exact: true }).click();
  await star(page).click();
  await card.getByRole('button', { name: 'Place 1 selected photo here' }).click();
  await expect.poll(() => writes.length).toBe(1);
  expect(writes[0]).toEqual({ ids: [PLACED[3]!.id], latitude: 60.6289, longitude: 6.4153 });
});

test('the star card leaves out selected photos already at the place', async ({ page }) => {
  const writes: LocationWrite[] = [];
  const bryggen = savedPlace('place-bryggen', 'Bryggen', {
    lat: 60.3975,
    lon: 5.324,
    city: 'Bergen',
    state: null,
    country: null
  });
  await openPlacing(page, writes, new PlacesMock({ saved: [bryggen] }));
  const panel = page.getByRole('complementary', { name: 'Photos in view' });
  const there = panel.locator('[role="group"][data-name="IMG_0001.ARW"]');
  await there.hover();
  await there.getByRole('button', { name: 'Select', exact: true }).click();

  const marker = page.locator(`[data-map-place="${bryggen.id}"]`);
  const card = page.getByRole('dialog', { name: 'Bryggen' });
  await marker.click();
  await expect(card).toBeVisible();
  await expect(card.getByRole('button', { name: /^Place/ })).toHaveCount(0);
  await page.keyboard.press('Escape');

  const away = panel.locator('[role="group"][data-name="IMG_0003.ARW"]');
  await away.hover();
  await away.getByRole('button', { name: 'Select', exact: true }).click();
  await marker.click();
  await card.getByRole('button', { name: 'Place 1 selected photo here' }).click();
  await expect.poll(() => writes.length).toBe(1);
  expect(writes[0]).toEqual({ ids: [PLACED[2]!.id], latitude: 60.3975, longitude: 5.324 });
});

const BRYGGEN = savedPlace('place-bryggen', 'Bryggen', {
  lat: 60.3975,
  lon: 5.324,
  city: 'Bergen',
  state: 'Vestland',
  country: 'Norway'
});

const FLAM = {
  id: 'recent-flam',
  lat: 60.8636,
  lon: 7.1133,
  city: 'Flåm',
  state: 'Vestland',
  country: 'Norway',
  used_at: '2026-11-20T10:00:00Z'
};

function placesPanel(page: Page) {
  return page.getByRole('complementary', { name: 'Photos in view' });
}

async function openPlacesTab(page: Page, places: PlacesMock): Promise<void> {
  await openPlacing(page, [], places);
  await placesPanel(page).getByRole('radio', { name: 'Places' }).click();
}

test('the Places tab lists, renames and removes saved places', async ({ page }) => {
  const places = new PlacesMock({ saved: [HYTTA, BRYGGEN], recent: [FLAM] });
  await openPlacesTab(page, places);
  const panel = placesPanel(page);
  const saved = panel.getByRole('list', { name: 'Saved places' });
  await expect(panel).toContainText('2 saved · 1 recent');
  await expect(saved.locator('[data-place-row="place-bryggen"]')).toContainText('1 photo');
  await expect(saved.locator('[data-place-row="place-hytta"]')).toContainText('no photos yet');

  await saved.locator('[data-place-row="place-hytta"]').hover();
  await expect(star(page)).toHaveAttribute('data-hit', 'true');

  await saved.getByRole('button', { name: 'Actions for Hytta' }).click();
  await page.getByRole('button', { name: 'Rename', exact: true }).click();
  const field = saved.getByRole('textbox', { name: 'Name of Hytta' });
  await field.fill('Hytta på Voss');
  await field.press('Enter');
  await expect(saved).toContainText('Hytta på Voss');
  expect(places.saved.find((p) => p.id === HYTTA.id)?.name).toBe('Hytta på Voss');

  await saved.getByRole('button', { name: 'Actions for Bryggen' }).click();
  await page.getByRole('button', { name: 'Remove', exact: true }).click();
  await expect(saved).not.toContainText('Bryggen');
  await page.getByRole('button', { name: 'Undo' }).click();
  await expect(saved).toContainText('Bryggen');

  await panel.getByRole('button', { name: 'Clear', exact: true }).click();
  await expect(panel.getByRole('list', { name: 'Recent places' })).toHaveCount(0);
  expect(places.recent).toEqual([]);
});

test('a star card renames its place and moves it by dragging the star', async ({ page }) => {
  const places = new PlacesMock({ saved: [HYTTA] });
  await openPlacing(page, [], places);
  await star(page).click();
  const card = page.getByRole('dialog', { name: 'Hytta' });
  await card.getByRole('button', { name: 'Rename Hytta' }).click();
  const field = card.getByRole('textbox', { name: 'Name of Hytta' });
  await field.fill('Hytta på Voss');
  await field.press('Enter');
  await expect.poll(() => places.saved[0]?.name).toBe('Hytta på Voss');

  await page
    .getByRole('dialog', { name: 'Hytta på Voss' })
    .getByRole('button', { name: 'Move', exact: true })
    .click();
  const bar = page.getByRole('toolbar', { name: 'Move Hytta på Voss' });
  await expect(bar).toBeVisible();
  const photo = page.locator('[data-map-photo]').first();
  await expect(photo).toHaveCSS('opacity', '0.35');
  await expect(photo).toHaveCSS('pointer-events', 'none');
  const box = (await star(page).boundingBox())!;
  await page.mouse.move(box.x + 14, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(box.x + 94, box.y + box.height / 2 + 60, { steps: 8 });
  await page.mouse.up();
  await bar.getByRole('button', { name: 'Done' }).click();
  await expect(bar).toHaveCount(0);
  await expect
    .poll(() =>
      page
        .locator('[data-map-photo]')
        .evaluateAll(
          (items) => items.filter((item) => getComputedStyle(item).opacity !== '1').length
        )
    )
    .toBe(0);
  await expect.poll(() => places.saved[0]?.lat).not.toBe(HYTTA.lat);
  expect(places.saved[0]!.lon).toBeGreaterThan(HYTTA.lon);
  expect(places.saved[0]!.city).toBe('Bergen');

  await page.getByRole('button', { name: 'Undo' }).click();
  await expect.poll(() => places.saved[0]?.lat).toBe(HYTTA.lat);
});

test('right-clicking the map saves a place there', async ({ page }) => {
  const places = new PlacesMock({ saved: [] });
  await openPlacing(page, [], places);
  const area = (await page.getByRole('region', { name: 'Map of your photos' }).boundingBox())!;
  await page.mouse.click(area.x + area.width / 2 + 40, area.y + area.height / 2 - 30, {
    button: 'right'
  });
  await page
    .getByRole('group', { name: 'This spot' })
    .getByRole('button', { name: 'Save place here' })
    .click();
  const card = page.getByRole('dialog', { name: 'New place' });
  const name = card.getByRole('textbox', { name: 'Place name' });
  await expect(name).toHaveValue('Bergen');
  await name.fill('Fjellstua');
  await card.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(card).toHaveCount(0);
  await expect.poll(() => places.saved.map((p) => p.name)).toEqual(['Fjellstua']);
  expect(places.saved[0]!.city).toBe('Bergen');
  await expect(page.locator(`[data-map-place="${places.saved[0]!.id}"]`)).toBeVisible();
});

test('New place in the Places tab saves the map centre', async ({ page }) => {
  const places = new PlacesMock({ saved: [] });
  await openPlacesTab(page, places);
  await placesPanel(page).getByRole('button', { name: 'New place' }).click();
  const card = page.getByRole('dialog', { name: 'New place' });
  await card.getByRole('textbox', { name: 'Place name' }).fill('Midt i');
  await card.getByRole('textbox', { name: 'Place name' }).press('Enter');
  await expect.poll(() => places.saved.map((p) => p.name)).toEqual(['Midt i']);
});

test.describe('saved places on the touch map', { tag: '@touch' }, () => {
  test('a tapped star offers its location', async ({ page }) => {
    await openPlacing(page, [], new PlacesMock({ saved: [HYTTA] }));
    await expect(star(page)).toBeVisible();
    await star(page).tap();
    const sheet = page.getByRole('dialog').filter({ hasText: 'Voss, Vestland, Norway' });
    await expect(sheet).toBeVisible();
    await sheet.getByRole('button', { name: 'Copy location' }).tap();
    await expect(page.getByText('Copied Hytta')).toBeVisible();
  });

  test('a place moves under the centre mark', async ({ page }) => {
    const places = new PlacesMock({ saved: [HYTTA] });
    await openPlacing(page, [], places);
    const pill = page.locator('[data-map-pill]');
    const phone = (await pill.count()) > 0;
    if (phone) await pill.tap();
    else await page.getByRole('button', { name: 'Photos panel' }).tap();
    const scope = phone ? page.locator('[data-map-sheet]') : placesPanel(page);
    await scope.getByRole('radio', { name: 'Places' }).tap();
    await scope.getByRole('button', { name: 'Actions for Hytta' }).tap();
    await page.getByRole('button', { name: 'Move on the map' }).tap();
    const bar = page.getByRole('toolbar', { name: 'Move Hytta' });
    await expect(bar).toBeVisible();
    await expect(page.locator('[data-place-mark]')).toBeVisible();

    const area = (await page.getByRole('region', { name: 'Map of your photos' }).boundingBox())!;
    const from = { x: area.x + area.width / 2, y: area.y + area.height / 2 };
    await drag(page, from, { x: from.x - 90, y: from.y - 60 }, { settleMs: 50 });
    await expect(page.locator('[data-place-mark] ~ [role="status"]')).toContainText('Bergen');
    await bar.getByRole('button', { name: 'Done' }).tap();
    await expect(bar).toHaveCount(0);
    await expect.poll(() => places.saved[0]?.lon).toBeGreaterThan(HYTTA.lon);
    expect(places.saved[0]!.lat).toBeLessThan(HYTTA.lat);
  });

  test('a long press on the map saves a place', async ({ page }) => {
    const places = new PlacesMock({ saved: [] });
    await openPlacing(page, [], places);
    const area = (await page.getByRole('region', { name: 'Map of your photos' }).boundingBox())!;
    await longPress(page, area.x + area.width / 2 + 30, area.y + area.height / 3);
    const menu = page.getByRole('dialog').filter({ hasText: 'This spot' });
    await menu.getByRole('button', { name: 'Save place here' }).tap();
    const form = page.getByRole('dialog', { name: 'New place' });
    await expect(form.getByRole('textbox', { name: 'Place name' })).toHaveValue('Bergen');
    await form.getByRole('button', { name: 'Save', exact: true }).tap();
    await expect.poll(() => places.saved.map((p) => p.name)).toEqual(['Bergen']);
  });
});
