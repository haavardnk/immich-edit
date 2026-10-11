import { expect, test } from '@playwright/test';
import { ASSET_SUMMARY, installMocks, openFilters } from './helpers';

const A = '00000000-0000-0000-0000-000000000001';
const B = '00000000-0000-0000-0000-000000000002';
const C = '00000000-0000-0000-0000-000000000003';

const ASSETS = [
  {
    ...ASSET_SUMMARY,
    id: A,
    originalFileName: 'IMG_0001.ARW',
    fileCreatedAt: '2024-01-01T00:00:00Z'
  },
  {
    ...ASSET_SUMMARY,
    id: B,
    originalFileName: 'IMG_0002.ARW',
    fileCreatedAt: '2024-02-01T00:00:00Z'
  },
  {
    ...ASSET_SUMMARY,
    id: C,
    originalFileName: 'IMG_0003.ARW',
    fileCreatedAt: '2024-03-01T00:00:00Z'
  }
];

test('opening the editor directly rebuilds the filmstrip from the return path', async ({
  page
}) => {
  await installMocks(page, { assets: ASSETS });

  await page.goto(`/assets/${B}?from=%2Fphotos`);

  await expect(page.getByRole('link', { name: 'IMG_0001.ARW' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'IMG_0003.ARW' })).toBeVisible();
});

test('a deep link without a return path gets its timeline neighbours', async ({ page }) => {
  await installMocks(page, { assets: ASSETS });

  await page.goto(`/assets/${B}`);
  await expect(page.getByRole('link', { name: 'IMG_0001.ARW' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'IMG_0003.ARW' })).toBeVisible();

  await page.keyboard.press('ArrowRight');
  await expect(page).toHaveURL(new RegExp(A));
});

test('arrow keys move through the collection after a direct editor load', async ({ page }) => {
  await installMocks(page, { assets: ASSETS });

  await page.goto(`/assets/${B}?from=%2Ftags%2Ftag-1`);
  await expect(page.getByRole('link', { name: 'IMG_0003.ARW' })).toBeVisible();

  await page.keyboard.press('ArrowRight');
  await expect(page).toHaveURL(new RegExp(C));

  await page.keyboard.press('ArrowLeft');
  await expect(page).toHaveURL(new RegExp(B));
});

test('a restored grid keeps the filters that were active when the editor opened', async ({
  page
}) => {
  await installMocks(page, { assets: ASSETS });

  const names: (string | undefined)[] = [];
  const track = (body: unknown): string | undefined =>
    (body as { filter?: { filename?: string } }).filter?.filename;
  await page.route('**/api/search/metadata', async (route) => {
    names.push(track(route.request().postDataJSON()));
    await route.fallback();
  });
  await page.route('**/api/search/window', async (route) => {
    names.push(track(route.request().postDataJSON()));
    await route.fallback();
  });

  await page.goto('/photos');
  await openFilters(page, 'File');
  await page.getByLabel('Filename').fill('IMG_0001');
  await expect(page).toHaveURL(/\/photos\?filename=IMG_0001$/);
  await expect.poll(() => names).toContain('IMG_0001');

  await page.getByRole('link', { name: 'IMG_0002.ARW' }).click({ position: { x: 8, y: 8 } });
  await expect(page).toHaveURL(new RegExp(`/assets/${B}\\?from=%2Fphotos%3Ffilename%3DIMG_0001$`));
  names.splice(0);
  await page.reload();
  await expect(page.getByRole('link', { name: 'IMG_0001.ARW' })).toBeVisible();
  expect(names).toContain('IMG_0001');

  await page.keyboard.press('g');
  await expect(page).toHaveURL(/\/photos\?filename=IMG_0001$/);
  await expect(page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')).toHaveText(
    ['Name: IMG_0001']
  );
});

test('filters live in the URL, survive a reload and step with Back and Forward', async ({
  page
}) => {
  await installMocks(page, { assets: ASSETS });
  await page.goto('/photos');
  const chips = page.getByRole('list', { name: 'Active filters' }).getByRole('listitem');

  const panel = await openFilters(page, 'Library', 'Rating & label', 'File');
  await panel.getByRole('checkbox', { name: 'Favorites only' }).click();
  await expect(page).toHaveURL(/\/photos\?favorite=1$/);
  await panel.getByRole('radio', { name: '3 stars' }).click();
  await expect(page).toHaveURL(/\/photos\?favorite=1&rating=3$/);
  await panel.getByLabel('Filename').fill('IMG');
  await expect(page).toHaveURL(/&filename=IMG$/);
  await panel.getByLabel('Filename').fill('IMG_0001');
  await expect(page).toHaveURL(/\/photos\?favorite=1&rating=3&filename=IMG_0001$/);

  await page.reload();
  await expect(chips).toHaveCount(3);
  await expect(chips.filter({ hasText: 'Favorites' })).toHaveCount(1);
  await expect(chips.filter({ hasText: 'Name: IMG_0001' })).toHaveCount(1);

  await page.goBack();
  await expect(page).toHaveURL(/\/photos\?favorite=1&rating=3$/);
  await expect(chips).toHaveCount(2);
  await page.goBack();
  await expect(page).toHaveURL(/\/photos\?favorite=1$/);
  await expect(chips).toHaveText(['Favorites']);
  await page.goForward();
  await expect(page).toHaveURL(/\/photos\?favorite=1&rating=3$/);
  await expect(chips).toHaveCount(2);

  await page.getByRole('link', { name: 'Photos', exact: true }).click();
  await expect(page).toHaveURL(/\/photos$/);
  await expect(chips).toHaveCount(0);
});

test('unusable filter params are dropped from the URL', async ({ page }) => {
  await installMocks(page, { assets: ASSETS });

  await page.goto('/photos?rating=banana&after=2024-02-30&favorite=1&visibility=trash');

  await expect(page).toHaveURL(/\/photos\?favorite=1$/);
  await expect(page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')).toHaveText(
    ['Favorites']
  );
});

test('a camera the library does not know is dropped without a history entry', async ({ page }) => {
  await installMocks(page, { assets: ASSETS });
  await page.goto('/photos');
  await openFilters(page, 'Camera');
  let release = (): void => undefined;
  const held = new Promise<void>((resolve) => (release = resolve));
  await page.route('**/api/search/suggestions', async (route) => {
    await held;
    await route.fallback();
  });

  await page.goto('/photos?make=Bogus');
  await expect(page.getByRole('list', { name: 'Active filters' })).toContainText('Bogus');
  const length = await page.evaluate(() => history.length);
  release();

  await expect(page).toHaveURL(/\/photos$/);
  expect(await page.evaluate(() => history.length)).toBe(length);
});
