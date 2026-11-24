import { expect, test } from '@playwright/test';
import { ASSET_ID, installMocks } from './helpers';

test('opening a video in the editor shows an unsupported notice without requesting preview', async ({
  page
}) => {
  let previewCalls = 0;
  await installMocks(page);
  await page.route(`**/api/assets/${ASSET_ID}`, (route) => {
    if (route.request().method() !== 'GET') return route.fallback();
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        id: ASSET_ID,
        originalFileName: 'CLIP0001.MP4',
        type: 'VIDEO',
        fileCreatedAt: '2024-06-15T12:00:00Z',
        isFavorite: false,
        exifInfo: null,
        tags: []
      })
    });
  });
  await page.route(`**/api/assets/${ASSET_ID}/preview**`, (route) => {
    previewCalls += 1;
    return route.fulfill({ status: 500, body: 'should not be called' });
  });

  await page.goto(`/assets/${ASSET_ID}`);
  await expect(page.getByText('Videos are not supported in the editor.')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Retry' })).toHaveCount(0);
  expect(previewCalls).toBe(0);
});

test('a 422 unsupported_format preview response shows an unsupported notice without retry', async ({
  page
}) => {
  await installMocks(page);
  await page.route(`**/api/assets/${ASSET_ID}/preview**`, (route) =>
    route.fulfill({
      status: 422,
      contentType: 'application/json',
      body: JSON.stringify({
        code: 'unsupported_format',
        message: 'Videos are not supported'
      })
    })
  );

  await page.goto(`/assets/${ASSET_ID}`);
  await expect(page.getByText('Videos are not supported')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Retry' })).toHaveCount(0);
});
