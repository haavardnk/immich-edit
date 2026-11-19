import { expect, test } from '@playwright/test';
import { installMocks, json, SESSION_USER, settingsGroup, settingsPages } from './helpers';

const PRESETS = [
  {
    id: 'preset-web',
    name: 'Web 2048',
    destination: 'download',
    form: { format: 'jpeg', quality: 80 },
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z'
  }
];

test('desktop settings list four pages in the sidebar and open on Preferences', async ({
  page
}) => {
  await installMocks(page);
  await page.goto('/settings');
  await expect(page).toHaveURL('/settings/preferences');
  await expect(page.getByRole('heading', { name: 'Preferences', level: 1 })).toBeVisible();
  await expect(page.getByRole('complementary', { name: 'Library' })).toHaveCount(0);

  await expect(settingsPages(page).getByRole('link')).toHaveText([
    /^\s*Preferences\s*$/,
    /^\s*Account\s*$/,
    /^\s*Administration\s*$/,
    /^\s*Diagnostics\s*$/
  ]);
  await expect(settingsPages(page).getByRole('link', { name: 'Preferences' })).toHaveAttribute(
    'aria-current',
    'page'
  );
  for (const name of ['Appearance', 'This device']) {
    await expect(settingsGroup(page, name)).toBeVisible();
  }
  await expect(page.getByRole('radiogroup', { name: 'Layout' })).toBeVisible();
  await expect(page.getByRole('switch', { name: 'Save exports automatically' })).toBeVisible();

  await settingsPages(page).getByRole('link', { name: 'Account' }).click();
  await expect(page).toHaveURL('/settings/account');
  await expect(page.getByRole('heading', { name: 'Account', level: 1 })).toBeVisible();
  for (const name of ['Signed-in devices', 'Export presets']) {
    await expect(settingsGroup(page, name)).toBeVisible();
  }
});

test('rows hold their control and repeat nothing the control shows', async ({ page }) => {
  await installMocks(page);
  await page.goto('/settings/preferences');
  const mode = page.getByRole('radiogroup', { name: 'Mode' });
  await expect(mode.getByRole('radio', { name: 'Dark' })).toBeChecked();
  await expect(page.getByText(/in use$/)).toHaveCount(0);
  await mode.getByRole('radio', { name: 'System' }).click();
  await expect(page.getByText(/^(Dark|Light) in use$/)).toBeVisible();

  const layout = page.getByRole('radiogroup', { name: 'Layout' });
  await expect(page.getByText('Desktop on this screen')).toBeVisible();
  await layout.getByRole('radio', { name: 'Desktop' }).click();
  await expect(page.getByText('Desktop on this screen')).toHaveCount(0);
});

test('old section links land on the row in their new page', async ({ page }) => {
  await installMocks(page);
  await page.goto('/settings/models');
  await expect(page).toHaveURL('/settings/administration#models');
  await expect(settingsGroup(page, 'Models')).toHaveAttribute('data-flash', '');

  await page.goto('/settings#sessions');
  await expect(page).toHaveURL('/settings/account#sessions');
});

test('non-admins get no Administration page', async ({ page }) => {
  await installMocks(page);
  await page.route('**/api/auth/me', (route) =>
    route.fulfill(json({ ...SESSION_USER, is_admin: false }))
  );
  await page.goto('/settings/administration');
  await expect(page).toHaveURL('/settings/preferences');
  await expect(settingsPages(page).getByRole('link', { name: /^Administration/ })).toHaveCount(0);
  await expect(settingsPages(page).getByRole('link', { name: 'Diagnostics' })).toBeVisible();
});

test('export presets can be reviewed and removed on the account page', async ({ page }) => {
  let presets = [...PRESETS];
  await installMocks(page);
  await page.route('**/api/export-presets**', (route) => {
    if (route.request().method() === 'DELETE') {
      presets = [];
      return route.fulfill({ status: 204 });
    }
    return route.fulfill(json(presets));
  });
  await page.goto('/settings/account');
  const group = settingsGroup(page, 'Export presets');
  await expect(group).toContainText('Web 2048');
  await group.getByRole('button', { name: 'Delete export preset Web 2048' }).click();
  await expect(group.getByText('No saved presets')).toBeVisible();
});

test.describe('settings on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('the index lists four pages and each pushes with Back', async ({ page }) => {
    await installMocks(page);
    await page.goto('/settings');
    await expect(page.getByRole('heading', { name: 'Settings', level: 1 })).toBeVisible();
    await expect(settingsPages(page).getByRole('link')).toHaveCount(4);
    await settingsPages(page).getByRole('link', { name: 'Preferences' }).tap();
    await expect(page).toHaveURL('/settings/preferences');
    await expect(page.getByRole('heading', { name: 'Preferences', level: 1 })).toBeVisible();
    await expect(settingsGroup(page, 'This device')).toBeVisible();
    await page
      .locator('header[data-app-bar]')
      .getByRole('button', { name: 'Back', exact: true })
      .tap();
    await expect(page).toHaveURL('/settings');
  });

  test('a theme row opens its grid in a sheet', async ({ page }) => {
    await installMocks(page);
    await page.goto('/settings/preferences');
    await page.getByRole('button', { name: /^Dark theme:/ }).tap();
    const sheet = page.getByRole('dialog', { name: 'Dark theme' });
    await expect(sheet.getByRole('radiogroup', { name: 'Dark theme' })).toBeVisible();
    await expect(sheet.getByRole('radiogroup', { name: 'Dark accent' })).toBeVisible();
  });
});

test.describe('settings on a tablet', { tag: ['@touch', '@tablet'] }, () => {
  test('the sidebar lists the four pages beside the open page', async ({ page }) => {
    await installMocks(page);
    await page.goto('/settings');
    await expect(page).toHaveURL('/settings/preferences');
    await settingsPages(page).getByRole('link', { name: 'Account' }).tap();
    await expect(page.getByRole('heading', { name: 'Account', level: 1 })).toBeVisible();
    await expect(settingsPages(page)).toBeVisible();
  });
});
