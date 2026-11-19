import { expect, test, type Page } from '@playwright/test';
import { SETTINGS_INDEX } from '../src/lib/settings/settingsIndex';
import { mockDiagnostics } from './diagnosticsMocks';
import { installMocks, json, SESSION_USER, settingsGroup, settingsPages } from './helpers';

function search(page: Page) {
  return page.getByRole('textbox', { name: 'Search settings' });
}

function results(page: Page) {
  return page.getByRole('region', { name: 'Settings search results' });
}

test('typing drops results grouped by page over the settings', async ({ page }) => {
  await installMocks(page);
  await page.goto('/settings/account');
  await search(page).fill('render');

  const preferences = results(page).getByRole('region', { name: 'Preferences results' });
  const diagnostics = results(page).getByRole('region', { name: 'Diagnostics results' });
  await expect(preferences.getByRole('link', { name: /Preview renderer/ })).toBeVisible();
  await expect(diagnostics.getByRole('link', { name: /Render latency/ })).toBeVisible();
  await expect(results(page).locator('mark').first()).toHaveText(/render/i);
  await expect(settingsPages(page)).toBeVisible();
  await expect(settingsGroup(page, 'Signed-in devices')).toBeVisible();

  await search(page).press('Escape');
  await expect(search(page)).toHaveValue('');
  await expect(results(page)).toHaveCount(0);
});

test('slash focuses the search and the keyboard opens and reveals a row', async ({ page }) => {
  await installMocks(page);
  await page.route('**/api/admin/render-defaults', (route) =>
    route.fulfill(json({ lens_auto: true }))
  );
  await page.goto('/settings/preferences');
  await expect(settingsPages(page)).toBeVisible();
  await page.keyboard.press('/');
  await expect(search(page)).toBeFocused();

  await page.keyboard.type('lens');
  await page.keyboard.press('ArrowDown');
  const hit = results(page).getByRole('link', { name: /Automatic lens corrections/ });
  await expect(hit).toBeFocused();
  await page.keyboard.press('Enter');

  await expect(page).toHaveURL('/settings/administration#lens-auto');
  await expect(page.getByRole('heading', { name: 'Administration', level: 1 })).toBeVisible();
  await expect(page.locator('[data-setting-anchor="lens-auto"]')).toHaveAttribute('data-flash', '');
  await expect(results(page)).toHaveCount(0);
  await expect(search(page)).toHaveValue('lens');
});

test('search leaves out admin settings for other users', async ({ page }) => {
  await installMocks(page);
  await page.route('**/api/auth/me', (route) =>
    route.fulfill(json({ ...SESSION_USER, is_admin: false }))
  );
  await page.goto('/settings/preferences');
  await search(page).fill('lens');
  await expect(page.getByText('No settings match “lens”.')).toBeVisible();
  await search(page).fill('render');
  await expect(results(page).getByRole('link', { name: /Render latency/ })).toBeVisible();
});

test('every search result anchor exists on its page', async ({ page }) => {
  await installMocks(page);
  await mockDiagnostics(page);
  await page.route('**/api/admin/render-defaults', (route) =>
    route.fulfill(json({ lens_auto: true }))
  );
  const byPage = Map.groupBy(
    SETTINGS_INDEX.filter((entry) => entry.anchor),
    (entry) => entry.page
  );
  for (const [id, entries] of byPage) {
    await page.goto(`/settings/${id}`);
    for (const entry of entries) {
      await expect(page.locator(`[data-setting-anchor="${entry.anchor}"]`)).toBeAttached();
    }
  }
});

test('picking the result already open reveals its row again', async ({ page }) => {
  await installMocks(page);
  await page.goto('/settings/preferences#layout');
  const row = page.locator('[data-setting-anchor="layout"]');
  await expect(row).toHaveAttribute('data-flash', '');
  await expect(row).not.toHaveAttribute('data-flash');

  await search(page).fill('layout');
  await results(page)
    .getByRole('link', { name: /^Layout/ })
    .click();
  await expect(row).toHaveAttribute('data-flash', '');
});

test.describe('settings search on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('results replace the index and a result pushes its page', async ({ page }) => {
    await installMocks(page);
    await page.goto('/settings');
    await search(page).fill('mode');
    await expect(settingsPages(page)).toHaveCount(0);
    await results(page)
      .getByRole('link', { name: /^Mode System/ })
      .tap();
    await expect(page).toHaveURL('/settings/preferences#mode');
    await expect(page.getByRole('heading', { name: 'Preferences', level: 1 })).toBeVisible();
  });
});
