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

export async function selectedNames(page: Page): Promise<string[]> {
  return page.evaluate(() =>
    [...document.querySelectorAll<HTMLElement>('[role="group"][data-selected]')]
      .map((node) => node.getAttribute('data-name') ?? '')
      .sort()
  );
}
