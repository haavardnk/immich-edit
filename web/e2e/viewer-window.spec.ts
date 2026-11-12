import { expect, test, type BrowserContext, type Locator, type Page } from '@playwright/test';
import { ASSET_SUMMARY, gotoAsset, installMocks, type PreviewRequest } from './helpers';

const SECOND = {
  ...ASSET_SUMMARY,
  id: '00000000-0000-0000-0000-000000000002',
  originalFileName: 'IMG_0002.ARW'
};

async function openViewer(page: Page): Promise<Page> {
  await page.getByRole('button', { name: 'More editor actions' }).click();
  const [viewer] = await Promise.all([
    page.waitForEvent('popup'),
    page.getByRole('button', { name: 'Open viewer window' }).click()
  ]);
  await viewer.waitForLoadState();
  return viewer;
}

function badge(viewer: Page): Locator {
  return viewer.getByTestId('viewer-sync');
}

function exposure(page: Page): Locator {
  return page
    .locator('div.group', { has: page.getByRole('button', { name: 'Exposure', exact: true }) })
    .getByRole('slider');
}

async function mock(context: BrowserContext, viewerRenders: PreviewRequest[]): Promise<void> {
  await installMocks(context, {
    assets: [ASSET_SUMMARY, SECOND],
    onPreview: (body) => {
      if (body.lane === 'viewer') viewerRenders.push(body);
    }
  });
}

test('the viewer window mirrors the editor and follows it', async ({ page, context }) => {
  const renders: PreviewRequest[] = [];
  await mock(context, renders);
  await gotoAsset(page);
  await expect(page.getByTestId('preview-image')).toBeVisible();

  const viewer = await openViewer(page);
  await expect(badge(viewer)).toHaveText('Synced');
  await expect(viewer.getByTestId('viewer-image')).toBeVisible();
  await expect(viewer).toHaveTitle(`${ASSET_SUMMARY.originalFileName} · Viewer`);
  await expect(viewer.getByTestId('viewer-zoom')).toHaveText(/^\d+%$/);
  await expect.poll(() => renders.length).toBeGreaterThan(0);

  await exposure(page).fill('1');
  await expect
    .poll(() =>
      renders.some((body) => (body.edits.basic as { exposure_ev: number }).exposure_ev === 1)
    )
    .toBe(true);

  await exposure(page).blur();
  const fit = await viewer.getByTestId('viewer-zoom').textContent();
  await page.keyboard.press('z');
  await expect(viewer.getByTestId('viewer-zoom')).not.toHaveText(fit ?? '');
  await page.keyboard.press('z');
  await expect(viewer.getByTestId('viewer-zoom')).toHaveText(fit ?? '');

  await page.keyboard.press('y');
  await expect(viewer.locator('[data-compare="split"]')).toBeVisible();
  await expect(viewer.locator('[data-compare-label]')).toHaveText(['Original', 'Edited']);
  await expect(viewer.getByTestId('viewer-left')).toBeVisible();
  await page.keyboard.press('y');
  await expect(viewer.locator('[data-compare="side"]')).toBeVisible();
  await expect(viewer.locator('[data-compare-label]')).toHaveText(['Original', 'Current']);
  await page.keyboard.press('y');
  await expect(viewer.locator('[data-compare]')).toHaveCount(0);

  await page.goto(`/assets/${SECOND.id}`);
  await expect(viewer).toHaveTitle(`${SECOND.originalFileName} · Viewer`);
  await expect(badge(viewer)).toHaveText('Synced');

  await page.getByRole('button', { name: /^Back/ }).click();
  await expect(badge(viewer)).toHaveText('Not synced');
  await expect(viewer.getByTestId('viewer-image')).toBeVisible();
});

test('a viewer opened alone waits for the editor', async ({ page, context }) => {
  await mock(context, []);
  await page.goto('/viewer');
  await expect(
    page.getByRole('status').filter({ hasText: 'Waiting for the editor' })
  ).toBeVisible();
  await expect(badge(page)).toHaveText('Not synced');
  await expect(page.getByRole('navigation')).toHaveCount(0);

  await page.keyboard.press('f');
  await expect.poll(() => page.evaluate(() => document.fullscreenElement !== null)).toBe(true);
  await page.keyboard.press('f');
  await expect.poll(() => page.evaluate(() => document.fullscreenElement !== null)).toBe(false);
});
