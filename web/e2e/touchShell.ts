import { expect, type Locator, type Page } from '@playwright/test';
import { ASSET_SUMMARY, installMocks, json, numberedAssets, type InstallOpts } from './helpers';

export const PEOPLE = [
  { id: 'person-ingrid', name: 'Ingrid', assetIds: [ASSET_SUMMARY.id] },
  { id: 'person-lars', name: 'Lars', assetIds: [ASSET_SUMMARY.id] }
];

export const ALBUMS = [
  { id: 'album-lofoten', albumName: 'Lofoten winter', assetIds: ['a', 'b', 'c'] },
  { id: 'album-family', albumName: 'Family', assetIds: ['a'] }
];

export const TAGS = [
  { id: 'tag-sky', name: 'Sky', value: 'Sky' },
  { id: 'tag-reject', name: 'reject', value: 'immich-edit/reject' }
];

export const EDITS = numberedAssets(5).map((asset, index) => ({
  id: asset.id,
  hash: `h${index}`,
  updated_at: `2026-0${index + 1}-01T00:00:00Z`
}));

export const FOLDERS = ['/photos/2026/Norway', '/photos/2026/Bergen', '/photos/2025'];

export async function mockLibrary(page: Page, opts: InstallOpts = {}): Promise<void> {
  await installMocks(page, { people: PEOPLE, albums: ALBUMS, tags: TAGS, edits: EDITS, ...opts });
  await page.route('**/api/folders/paths', (route) => route.fulfill(json(FOLDERS)));
  await page.route('**/api/folders/assets?*', (route) => route.fulfill(json([ASSET_SUMMARY])));
}

export async function open(page: Page, path: string, opts: InstallOpts = {}): Promise<void> {
  await mockLibrary(page, opts);
  await page.goto(path);
  await expect(page.locator('html')).toHaveAttribute('data-layout', 'touch');
}

export function tabBar(page: Page): Locator {
  return page.getByRole('navigation', { name: 'Main' });
}

export function tab(page: Page, name: string): Locator {
  return tabBar(page).getByRole('link', { name, exact: true });
}

export function drawer(page: Page): Locator {
  return page.getByRole('dialog', { name: 'Account' });
}

export function avatar(page: Page): Locator {
  return page.getByRole('button', { name: 'Account', exact: true });
}

export function backButton(page: Page): Locator {
  return page.locator('header[data-app-bar]').getByRole('button', { name: 'Back', exact: true });
}

export async function expectTab(page: Page, name: string): Promise<void> {
  await expect(tab(page, name)).toHaveAttribute('aria-current', 'page');
  await expect(tabBar(page).locator('[aria-current="page"]')).toHaveCount(1);
}

export async function openDrawer(page: Page): Promise<Locator> {
  await avatar(page).tap();
  const panel = drawer(page);
  await expect(panel).toBeVisible();
  await expect.poll(async () => (await panel.boundingBox())?.x ?? Number.NaN).toBeCloseTo(0, 0);
  return panel;
}

export async function expectTargets(locator: Locator, min: number): Promise<void> {
  for (const element of await locator.all()) {
    expect((await element.boundingBox())?.height ?? 0).toBeGreaterThanOrEqual(min);
  }
}

export async function expectBarDockedUnderGrid(page: Page): Promise<void> {
  await expect
    .poll(async () => {
      const bar = await page.getByRole('toolbar', { name: 'Selection actions' }).boundingBox();
      const column = await page.getByTestId('grid-column').boundingBox();
      if (!bar || !column) return false;
      const gaps = [bar.x - column.x, bar.width - column.width, bar.y - column.y - column.height];
      return gaps.every((gap) => Math.abs(gap) < 1);
    })
    .toBe(true);
}
