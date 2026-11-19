import { expect, test, type Page } from '@playwright/test';
import { installMocks } from './helpers';
import { datedAssets, dayHeader, dayShots } from './timeline';
import { open } from './touchShell';

const ASSETS = datedAssets([
  ...dayShots('2026-10-31', 4, 'Bergen'),
  { at: '2026-10-31T03:00:00Z', city: 'Oslo' },
  { at: '2026-10-31T02:00:00Z', city: 'Oslo' },
  ...dayShots('2026-10-30', 5, 'Ålesund'),
  ...dayShots('2025-07-04', 4)
]);

const SPLIT = [ASSETS.slice(0, 8), ASSETS.slice(8)];

test.use({ timezoneId: 'UTC' });

async function openPhotos(page: Page, opts: Parameters<typeof installMocks>[1] = {}) {
  await page.setViewportSize({ width: 1440, height: 1400 });
  await page.clock.setFixedTime(new Date('2026-11-05T12:00:00Z'));
  await installMocks(page, { assets: ASSETS, ...opts });
  await page.goto('/photos');
  await expect(page.getByTestId('day-header').first()).toBeVisible();
}

function headers(page: Page) {
  return page.getByTestId('day-header');
}

function selectedCount(page: Page) {
  return page.getByRole('toolbar', { name: 'Selection actions' });
}

async function expectBarInGridColumn(page: Page): Promise<void> {
  const bar = await selectedCount(page).boundingBox();
  const column = await page.getByTestId('grid-column').boundingBox();
  if (!bar || !column) throw new Error('no bar or grid column');
  expect(Math.abs(bar.x + bar.width / 2 - (column.x + column.width / 2))).toBeLessThan(1.5);
  expect(bar.x).toBeGreaterThanOrEqual(column.x);
  expect(bar.y + bar.height).toBeLessThanOrEqual(column.y + column.height);
}

test('date taken groups the grid by capture day with place and count', async ({ page }) => {
  await openPhotos(page);

  await expect(headers(page).getByRole('heading')).toHaveText([
    'Saturday 31 October',
    'Friday 30 October',
    'Friday 4 July 2025'
  ]);
  await expect(headers(page)).toHaveText([
    /Saturday 31 October\s*Bergen · 6/,
    /Friday 30 October\s*Ålesund · 5/,
    /Friday 4 July 2025\s*4\s*$/
  ]);
  const header = await dayHeader(page, 'Friday 30 October').boundingBox();
  const firstOfDay = await page.getByRole('link', { name: 'IMG_0007.ARW' }).boundingBox();
  expect(firstOfDay?.y ?? 0).toBeGreaterThan((header?.y ?? 0) + (header?.height ?? 0) - 1);
});

test('other sorts keep the flat grid', async ({ page }) => {
  await page.addInitScript(() =>
    localStorage.setItem(
      'immich-edit:browseSort',
      JSON.stringify({ timeline: { field: 'added', direction: 'desc' } })
    )
  );
  await installMocks(page, { assets: ASSETS });
  await page.goto('/photos');
  await expect(page.getByRole('link', { name: 'IMG_0001.ARW' })).toBeVisible();
  await expect(headers(page)).toHaveCount(0);
  await expect(page.getByRole('slider', { name: 'Timeline' })).toHaveCount(0);
});

test('select day picks the whole day, loading the part past the loaded end', async ({ page }) => {
  await openPhotos(page, { searchPages: SPLIT });
  const friday = dayHeader(page, 'Friday 30 October');
  await expect(friday).toContainText('Ålesund · 2');

  await friday.hover();
  const circle = friday.getByRole('button', { name: 'Select day' });
  await expect(circle).toHaveCSS('opacity', '1');
  await circle.hover();
  await expect(page.locator('.tooltip-content', { hasText: 'Select day' })).toBeVisible();
  await circle.click();

  await expect(selectedCount(page)).toContainText('5 selected');
  await expect(friday).toContainText('Ålesund · 5');
  await expect(friday.getByRole('button', { name: 'Deselect day' })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
  await expect(
    dayHeader(page, 'Saturday 31 October').getByRole('button', { name: 'Select day' })
  ).toHaveCSS('opacity', '1');

  await friday.getByRole('button', { name: 'Deselect day' }).click();
  await expect(selectedCount(page)).toHaveCount(0);
});

test('shift-click on a second day header selects every day between', async ({ page }) => {
  await openPhotos(page);
  const saturday = dayHeader(page, 'Saturday 31 October');
  await saturday.hover();
  await saturday.getByRole('button', { name: 'Select day' }).click();
  await expect(selectedCount(page)).toContainText('6 selected');

  await page.keyboard.down('Shift');
  await dayHeader(page, 'Friday 4 July 2025').getByRole('button', { name: 'Select day' }).click();
  await page.keyboard.up('Shift');

  await expect(selectedCount(page)).toContainText('15 selected');
  await expect(headers(page).getByRole('button', { name: 'Deselect day' })).toHaveCount(3);
  await expectBarInGridColumn(page);
});

test('the select circle stays out of sight until a day is hovered', async ({ page }) => {
  await openPhotos(page);
  const circle = dayHeader(page, 'Friday 30 October').getByRole('button', { name: 'Select day' });
  await page.mouse.move(5, 400);
  await expect(circle).toHaveCSS('opacity', '0');
  await expect(circle).toHaveCSS('width', '0px');
  const before = await dayHeader(page, 'Friday 30 October').getByRole('heading').boundingBox();
  const tile = await page.getByRole('link', { name: 'IMG_0007.ARW' }).boundingBox();
  expect(Math.abs((before?.x ?? 0) - (tile?.x ?? 0))).toBeLessThan(8);
});

test.describe('touch', () => {
  test('@touch day headers show the place on the right and select circles while selecting', async ({
    page
  }) => {
    await page.clock.setFixedTime(new Date('2026-11-05T12:00:00Z'));
    await open(page, '/photos', { assets: ASSETS });
    const saturday = dayHeader(page, 'Saturday 31 October');
    await expect(saturday).toBeVisible();
    await expect(saturday.getByRole('button', { name: 'Select day' })).toBeHidden();

    const place = await saturday.getByText('Bergen · 6').boundingBox();
    const box = await saturday.boundingBox();
    expect((place?.x ?? 0) + (place?.width ?? 0)).toBeGreaterThan(
      (box?.x ?? 0) + (box?.width ?? 0) - 12
    );

    await page.getByRole('button', { name: 'Select', exact: true }).first().tap();
    await expect(selectedCount(page)).toBeVisible();
    const circle = dayHeader(page, 'Friday 30 October').getByRole('button', { name: 'Select day' });
    await expect(circle).toBeVisible();
    const target = await circle.boundingBox();
    expect(target?.height).toBeGreaterThanOrEqual(44);
    expect(target?.width).toBeGreaterThanOrEqual(44);

    await circle.tap();
    await expect(selectedCount(page)).toContainText('6 selected');
    await expectBarInGridColumn(page);
  });
});
