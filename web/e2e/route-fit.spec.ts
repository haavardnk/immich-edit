import { expect, test, type Page } from '@playwright/test';
import { ASSET_SUMMARY, installMocks, json } from './helpers';

const ROUTES = [
  { path: '/photos', text: 'Photos' },
  { path: '/favorites', text: 'Favorites' },
  { path: '/edited', text: 'Edited' },
  { path: '/folders', text: 'Select a folder' },
  { path: '/albums/album-1', text: 'Review album' },
  { path: '/people/person-1', text: 'Person' },
  { path: '/tags/tag-1', text: 'Tag' },
  { path: '/search', text: 'Search your library' }
];

async function installRouteMocks(page: Page): Promise<void> {
  await installMocks(page);
  await page.route('**/api/albums/album-1', (route) =>
    route.fulfill(
      json({
        id: 'album-1',
        albumName: 'Review album',
        assetCount: 1,
        assets: [ASSET_SUMMARY],
        updatedAt: '2024-01-01T00:00:00Z'
      })
    )
  );
}

for (const size of [
  { width: 1600, height: 832 },
  { width: 390, height: 844 }
]) {
  test(`browse routes fit ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await installRouteMocks(page);

    for (const route of ROUTES) {
      await page.goto(route.path);
      await expect(page.getByText(route.text, { exact: true }).first()).toBeVisible();
      expect(
        await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 2)
      ).toBe(true);
    }
  });
}
