import { expect, type Locator, type Page } from '@playwright/test';

export async function unfoldExport(scope: Locator): Promise<void> {
  await expect(scope.locator('[data-collapsible-trigger]').first()).toBeVisible();
  const closed = scope.locator('[data-collapsible-trigger][aria-expanded="false"]');
  for (let left = await closed.count(); left > 0; left = await closed.count()) {
    await closed.first().click();
  }
}

export async function openExportTab(page: Page): Promise<Locator> {
  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  const panel = page.getByRole('tabpanel');
  await unfoldExport(panel);
  return panel;
}
