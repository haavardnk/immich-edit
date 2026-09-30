import { expect, test } from '@playwright/test';
import { ASSET_SUMMARY, installMocks } from './helpers';

const A = '00000000-0000-0000-0000-000000000001';
const B = '00000000-0000-0000-0000-000000000002';
const C = '00000000-0000-0000-0000-000000000003';

const ASSETS = [A, B, C].map((id, i) => ({
  ...ASSET_SUMMARY,
  id,
  originalFileName: `IMG_000${i + 1}.ARW`,
  fileCreatedAt: `2024-0${i + 1}-01T00:00:00Z`
}));

test('the editor warms both neighbours once its preview and filmstrip show', async ({ page }) => {
  await installMocks(page, { assets: ASSETS });
  const warmed: unknown[] = [];
  await page.route('**/api/frames/warm', async (route) => {
    warmed.push(route.request().postDataJSON());
    await route.fulfill({ status: 202, body: '' });
  });
  let release = (): void => undefined;
  const listed = new Promise<void>((resolve) => (release = resolve));
  await page.route(/\/api\/search\/(metadata|window)$/, async (route) => {
    await listed;
    await route.fallback();
  });

  await page.goto(`/assets/${B}`);
  await expect(page.getByTestId('preview-image')).toBeVisible();
  expect(warmed).toEqual([]);
  release();
  await expect(page.getByRole('link', { name: 'IMG_0003.ARW' })).toBeVisible();

  await expect.poll(() => warmed).toEqual([{ ids: [A, C] }]);
  await page.keyboard.press('ArrowRight');
  await expect(page).toHaveURL(new RegExp(A));
  await expect.poll(() => warmed).toEqual([{ ids: [A, C] }, { ids: [B] }]);
});
