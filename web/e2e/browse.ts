import { expect, type Locator, type Page } from '@playwright/test';
import { ASSET_SUMMARY, installMocks } from './helpers';

export async function openBrowse(page: Page, layout?: 'desktop' | 'touch'): Promise<void> {
  await installMocks(page);
  await page.goto('/photos');
  if (layout) await expect(page.locator('html')).toHaveAttribute('data-layout', layout);
  await expect(page.getByRole('link', { name: ASSET_SUMMARY.originalFileName })).toBeVisible();
}

export function tile(page: Page, name: string): Locator {
  return page.locator(`[role="group"][data-name="${name}"]`);
}

export async function pick(page: Page, name: string): Promise<void> {
  await tile(page, name).hover();
  await tile(page, name).getByRole('button', { name: 'Select', exact: true }).click();
}

export function bulkBar(page: Page): Locator {
  return page.getByRole('toolbar', { name: 'Selection actions' });
}

export async function openBulkExport(page: Page): Promise<Locator> {
  await bulkBar(page).getByRole('button', { name: 'Export selected photos' }).click();
  const panel = page.getByRole('complementary', { name: 'Export', exact: true });
  await expect(panel).toBeVisible();
  return panel;
}

export async function openBulkEdits(page: Page): Promise<Locator> {
  await bulkBar(page).getByRole('button', { name: 'Edits', exact: true }).click();
  const menu = page.getByRole('group', { name: 'Edits', exact: true });
  await expect(menu).toBeVisible();
  return menu;
}

export async function openBulkMetadata(page: Page): Promise<Locator> {
  await bulkBar(page).getByRole('button', { name: 'Metadata', exact: true }).click();
  const menu = page.getByRole('group', { name: 'Metadata', exact: true });
  await expect(menu).toBeVisible();
  return menu;
}

export async function openBulkLocation(page: Page): Promise<Locator> {
  const metadata = await openBulkMetadata(page);
  await metadata.getByRole('button', { name: 'Location', exact: true }).click();
  const menu = page.getByRole('group', { name: 'Location', exact: true });
  await expect(menu).toBeVisible();
  return menu;
}

export async function openTouchMetadata(page: Page): Promise<Locator> {
  await bulkBar(page).getByRole('button', { name: 'More', exact: true }).tap();
  await page
    .getByRole('dialog', { name: 'More actions' })
    .getByRole('button', { name: 'Metadata' })
    .tap();
  const sheet = page.getByRole('dialog', { name: 'Metadata' });
  await expect(sheet).toBeVisible();
  return sheet;
}

export async function openTouchLocation(page: Page): Promise<Locator> {
  const metadata = await openTouchMetadata(page);
  await metadata.getByRole('button', { name: 'Location' }).tap();
  const sheet = page.getByRole('dialog', { name: 'Location' });
  await expect(sheet).toBeVisible();
  return sheet;
}

export async function selectedNames(page: Page): Promise<string[]> {
  return page.evaluate(() =>
    [...document.querySelectorAll<HTMLElement>('[role="group"][data-selected]')]
      .map((node) => node.getAttribute('data-name') ?? '')
      .sort()
  );
}

export function sizeButton(page: Page, size: string): Locator {
  return page.getByRole('button', { name: `Thumbnail size ${size}`, exact: true });
}

export async function pickSort(page: Page, name: string): Promise<void> {
  await page.getByRole('button', { name: /^Sort:/ }).click();
  await page.locator('[data-popover-content]').getByRole('radio', { name, exact: true }).click();
  await page.keyboard.press('Escape');
}

export function loupeToolbar(page: Page): Locator {
  return page.getByRole('navigation', { name: 'Loupe toolbar' });
}
