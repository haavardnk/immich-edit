import { expect, test } from '@playwright/test';
import { installMocks, json, settingsGroup } from './helpers';

const OTHER = {
  id: 'user-other',
  email: 'other@example.com',
  name: 'Other User',
  is_admin: false,
  access_enabled: true
};

const FAILURE = json({ code: 'internal', message: 'disk is read-only' }, 500);

test('a failed purge explains itself inside the confirmation', async ({ page }) => {
  await installMocks(page);
  await page.route('**/api/admin/users**', (route) => {
    if (route.request().method() === 'DELETE') return route.fulfill(FAILURE);
    return route.fulfill(json({ users: [OTHER] }));
  });
  await page.goto('/settings/administration#users');

  await settingsGroup(page, 'Users')
    .getByRole('button', { name: 'Purge local data for other@example.com' })
    .click();
  const dialog = page.getByRole('dialog', { name: 'Purge local data?' });
  await dialog.getByRole('textbox', { name: 'Confirm email' }).fill(OTHER.email);
  await dialog.getByRole('button', { name: 'Purge data' }).click();

  await expect(dialog.getByText('disk is read-only')).toBeVisible();
  await dialog.getByRole('button', { name: 'Cancel' }).click();
  await expect(dialog).toHaveCount(0);
  await expect(page.getByText('disk is read-only')).toHaveCount(0);
});

test('an instance that fails to load says so on its row, not in the rebind form', async ({
  page
}) => {
  await installMocks(page);
  await page.route('**/api/admin/instance', (route) => route.fulfill(FAILURE));
  await page.goto('/settings/administration#instance');

  const row = page.locator('[data-setting-anchor="instance"]');
  await expect(row.getByText('disk is read-only')).toBeVisible();
  await row.getByRole('button', { name: 'Change server…' }).click();
  const dialog = page.getByRole('dialog', { name: 'Change Immich server' });
  await expect(dialog).toBeVisible();
  await expect(dialog.getByText('disk is read-only')).toHaveCount(0);
});
