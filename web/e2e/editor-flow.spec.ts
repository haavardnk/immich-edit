import { expect, test, type Page } from '@playwright/test';
import { ASSET_ID, ASSET_SUMMARY, installMocks, numberedAssets } from './helpers';

test('photos → asset → export tab', async ({ page }) => {
  await installMocks(page);

  await page.goto('/photos');
  const tile = page.locator(`a[href^="/assets/${ASSET_ID}?"]`).first();
  await expect(tile).toBeVisible();
  const box = await tile.boundingBox();
  if (!box) throw new Error('tile has no bounding box');
  await tile.click({ position: { x: box.width - 6, y: box.height - 6 } });
  await page.waitForURL(new RegExp(`/assets/${ASSET_ID}(?:\\?.*)?$`));

  await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();

  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  await expect(page.getByRole('radio', { name: 'Download' })).toBeVisible();
});

test('photos grid restores scroll after editor back', async ({ page }) => {
  const assets = numberedAssets(80);
  await installMocks(page, { assets });

  await page.goto('/photos');
  const scroller = page.getByRole('main').locator('.overflow-y-auto').first();
  await expect(scroller).toBeVisible();
  await scroller.evaluate((el) => {
    el.scrollTop = 900;
    el.dispatchEvent(new Event('scroll'));
  });
  await expect.poll(async () => scroller.evaluate((el) => el.scrollTop)).toBeGreaterThan(500);
  const link = page.getByRole('main').locator('a[href^="/assets/"]').first();
  await expect(link).toBeVisible();
  const href = await link.getAttribute('href');
  if (!href) throw new Error('visible tile has no href');
  const before = await scroller.evaluate((el) => el.scrollTop);

  await link.evaluate((el) => (el as HTMLAnchorElement).click());
  await page.waitForURL(`**${href}`);
  await page.getByRole('button', { name: /^Back/ }).click();
  await page.waitForURL('**/photos');

  await expect.poll(async () => scroller.evaluate((el) => el.scrollTop)).toBeCloseTo(before, 0);
  await expect(link).toBeVisible();
});

test('editor navigation walks past the last loaded page', async ({ page }) => {
  const assets = numberedAssets(80);
  const pages: unknown[] = [];
  await installMocks(page, {
    assets,
    searchPages: [assets.slice(0, 40), assets.slice(40)],
    onMetadata: (body) => void pages.push(body.page)
  });

  await page.goto('/photos');
  const tile = page.locator(`a[href^="/assets/${ASSET_ID}?"]`).first();
  await expect(tile).toBeVisible();
  await tile.evaluate((el) => (el as HTMLAnchorElement).click());
  await page.waitForURL(new RegExp(`/assets/${ASSET_ID}\\?`));
  expect(pages).toEqual([undefined]);

  const toolbar = page.getByRole('navigation', { name: 'Editor toolbar' });
  for (let step = 0; step < 41; step += 1) {
    await page.keyboard.press('ArrowRight');
    await expect(
      toolbar.getByText(`IMG_${String(step + 2).padStart(4, '0')}.ARW`, { exact: true })
    ).toBeVisible();
  }
  expect(pages).toEqual([undefined, 2]);
});

async function expectTileRevealed(page: Page, name: string): Promise<void> {
  const scroller = page.getByRole('main').locator('.overflow-y-auto').first();
  const tile = page.locator(`[role="group"][title="${name}"]`);
  await expect
    .poll(async () => {
      const [outer, inner] = await Promise.all([scroller.boundingBox(), tile.boundingBox()]);
      if (!outer || !inner) return false;
      return inner.y >= outer.y && inner.y + inner.height <= outer.y + outer.height;
    })
    .toBe(true);
  await page.keyboard.press('ArrowRight');
  await expect(tile).toHaveAttribute('data-selected', 'true');
}

test('closing the loupe reveals the photo it ended on', async ({ page }) => {
  await installMocks(page, { assets: numberedAssets(80) });
  await page.goto('/photos');
  await expect(page.locator(`a[href^="/assets/${ASSET_ID}?"]`)).toBeVisible();
  await page.getByLabel('Quick review').first().click();

  for (let step = 0; step < 59; step += 1) await page.keyboard.press('ArrowRight');
  await expect(page.getByRole('img', { name: 'IMG_0060.ARW' })).toBeVisible();
  await page.keyboard.press('Escape');

  await expectTileRevealed(page, 'IMG_0060.ARW');
});

test('editor back reveals the photo reached with the arrow keys', async ({ page }) => {
  await installMocks(page, { assets: numberedAssets(80) });
  await page.goto('/photos');
  const tile = page.locator(`a[href^="/assets/${ASSET_ID}?"]`).first();
  await expect(tile).toBeVisible();
  await tile.evaluate((el) => (el as HTMLAnchorElement).click());
  await page.waitForURL(new RegExp(`/assets/${ASSET_ID}\\?`));

  const toolbar = page.getByRole('navigation', { name: 'Editor toolbar' });
  for (let step = 2; step <= 60; step += 1) {
    await page.keyboard.press('ArrowRight');
    await page.waitForURL(new RegExp(`-0*${step}\\?`));
  }
  await expect(toolbar.getByText('IMG_0060.ARW', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: /^Back/ }).click();
  await page.waitForURL('**/photos');

  await expectTileRevealed(page, 'IMG_0060.ARW');
});

test('back returns to the grid and the first arrow picks up the photo left open', async ({
  page
}) => {
  const second = '00000000-0000-0000-0000-000000000002';
  await installMocks(page, {
    assets: [
      ASSET_SUMMARY,
      { ...ASSET_SUMMARY, id: second, originalFileName: 'IMG_0002.ARW', checksum: 'bbbb' }
    ]
  });

  await page.goto('/search?q=IMG');
  const first = page.locator(`a[href^="/assets/${ASSET_ID}?"]`).first();
  await expect(first).toBeVisible();
  await first.evaluate((el) => (el as HTMLAnchorElement).click());
  await page.waitForURL(new RegExp(`/assets/${ASSET_ID}(?:\\?.*)?$`));

  await page.locator(`a[href^="/assets/${second}?"]`).first().click();
  await page.waitForURL(new RegExp(`/assets/${second}(?:\\?.*)?$`));

  await page.getByRole('button', { name: /^Back/ }).click();
  await page.waitForURL('**/search?q=IMG');
  const tile = page.getByRole('main').locator(`div:has(> a[href^="/assets/${second}?"])`);
  await expect(tile).toBeVisible();
  await expect(page.getByRole('main').locator('[data-selected]')).toHaveCount(0);
  await page.keyboard.press('ArrowRight');
  await expect(tile).toHaveAttribute('data-selected', 'true');
});

test('back from a photo outside the timeline lands on a clean grid', async ({ page }) => {
  await installMocks(page, {
    assets: [{ ...ASSET_SUMMARY, id: '00000000-0000-0000-0000-000000000002' }]
  });

  await page.goto(`/assets/${ASSET_ID}`);
  await page.getByRole('button', { name: /^Back/ }).click();
  await page.waitForURL('**/photos');
  await expect(page.getByRole('main').locator('[data-selected]')).toHaveCount(0);
});

test('back from a deep-linked timeline photo picks it up on the grid', async ({ page }) => {
  await installMocks(page);

  await page.goto(`/assets/${ASSET_ID}`);
  await expect(page.getByRole('link', { name: ASSET_SUMMARY.originalFileName })).toBeVisible();
  await page.getByRole('button', { name: /^Back/ }).click();
  await page.waitForURL('**/photos');
  const tile = page.getByRole('main').locator(`div:has(> a[href^="/assets/${ASSET_ID}?"])`);
  await expect(tile).toBeVisible();
  await page.keyboard.press('ArrowRight');
  await expect(tile).toHaveAttribute('data-selected', 'true');
});

test('the grid keeps its selection across an editor round trip', async ({ page }) => {
  const second = '00000000-0000-0000-0000-000000000002';
  await installMocks(page, {
    assets: [
      ASSET_SUMMARY,
      { ...ASSET_SUMMARY, id: second, originalFileName: 'IMG_0002.ARW', checksum: 'bbbb' }
    ]
  });

  await page.goto('/photos');
  await page.getByRole('button', { name: 'Select', exact: true }).nth(0).click();
  await page.getByRole('button', { name: 'Select', exact: true }).nth(0).click();
  await expect(page.getByText('2 selected')).toBeVisible();

  await page.keyboard.press('d');
  await page.waitForURL('**/assets/**');
  await page.getByRole('button', { name: /^Back/ }).click();
  await page.waitForURL('**/photos');

  await expect(page.getByText('2 selected')).toBeVisible();
});
