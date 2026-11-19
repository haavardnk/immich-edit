import { expect, test } from '@playwright/test';
import { installMocks, json, openAccountMenu } from './helpers';

const SESSIONS = ['one', 'two', 'three'].map((id, index) => ({
  id,
  current: index === 0,
  created_at: '2024-01-01T00:00:00Z',
  last_seen_at: '2024-01-02T00:00:00Z',
  user_agent: `Browser ${id}`,
  ip: null
}));

test('the account menu replaces the theme and shortcut buttons in the top bar', async ({
  page
}) => {
  await installMocks(page);
  await page.route('**/api/auth/sessions', (route) => route.fulfill(json({ sessions: SESSIONS })));
  await page.goto('/photos');

  const bar = page.getByRole('navigation', { name: 'Global navigation' });
  await expect(bar.getByRole('button', { name: /^Switch to (light|dark) theme$/ })).toHaveCount(0);
  await expect(bar.getByRole('button', { name: 'Keyboard shortcuts' })).toHaveCount(0);
  await expect(bar.getByRole('button', { name: 'Jobs', exact: true })).toBeVisible();
  await expect(bar.getByRole('button', { name: 'Settings', exact: true })).toHaveCount(0);

  const menu = await openAccountMenu(page);
  await expect(menu.getByText('user@example.com').first()).toBeVisible();
  await expect(menu.getByText('admin', { exact: true })).toBeVisible();
  await expect(menu.getByText('password', { exact: true })).toHaveCount(0);
  await expect(menu.getByRole('radiogroup', { name: 'Theme' })).toBeVisible();
  await expect(menu.getByRole('button', { name: 'Signed-in devices' })).toContainText('3');

  await menu.getByRole('button', { name: 'Signed-in devices' }).click();
  await expect(menu).toBeHidden();
  await expect(page).toHaveURL(/\/settings/);
  await expect(page.getByRole('button', { name: 'Revoke session for Browser two' })).toBeVisible();
});

test('settings and sign out are one click from the account menu', async ({ page }) => {
  await installMocks(page);
  await page.goto('/photos');

  await (await openAccountMenu(page)).getByRole('button', { name: 'Settings' }).click();
  await expect(page).toHaveURL(/\/settings/);

  const logout = page.waitForRequest((r) => r.url().includes('/api/auth/logout'));
  await (await openAccountMenu(page)).getByRole('button', { name: 'Sign out' }).click();
  await logout;
});
