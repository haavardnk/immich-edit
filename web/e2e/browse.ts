import { expect, type Page } from '@playwright/test';
import { ASSET_SUMMARY, installMocks } from './helpers';

export async function openBrowse(page: Page, layout?: 'desktop' | 'touch'): Promise<void> {
  await installMocks(page);
  await page.goto('/photos');
  if (layout) await expect(page.locator('html')).toHaveAttribute('data-layout', layout);
  await expect(page.getByRole('link', { name: ASSET_SUMMARY.originalFileName })).toBeVisible();
}
