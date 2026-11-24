import { expect, test, type Page } from '@playwright/test';
import { type LocationWrite, PLACED, openPlacing } from './mapFixtures';
import { PlacesMock, savedPlace } from './placesMocks';

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
    city: 'Bergen'
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

test.describe('saved places on the touch map', { tag: '@touch' }, () => {
  test('a tapped star offers its location', async ({ page }) => {
    await openPlacing(page, [], new PlacesMock({ saved: [HYTTA] }));
    await expect(star(page)).toBeVisible();
    await star(page).tap();
    const card = page.getByRole('dialog', { name: 'Hytta' });
    await expect(card).toBeVisible();
    await card.getByRole('button', { name: 'Copy location' }).tap();
    await expect(page.getByText('Copied Hytta')).toBeVisible();
  });
});
