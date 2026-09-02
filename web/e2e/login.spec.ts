import { expect, test } from '@playwright/test';

test.beforeEach(async ({ page }) => {
  await page.route('**/api/**', (route) => {
    const p = new URL(route.request().url()).pathname;
    if (p === '/api/setup/status')
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ configured: true })
      });
    if (p === '/api/auth/me')
      return route.fulfill({
        status: 401,
        contentType: 'application/json',
        body: JSON.stringify({ code: 'unauthorized', message: 'unauthorized' })
      });
    return route.fulfill({ status: 200, contentType: 'application/json', body: '{}' });
  });
});

test('login page renders the password sign-in form', async ({ page }) => {
  await page.goto('/login');
  await expect(page.getByRole('heading', { name: 'Sign in' })).toBeVisible();
  await expect(page.getByLabel('Immich email', { exact: true })).toBeVisible();
  await expect(page.getByLabel('Immich password', { exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: /sign in/i })).toBeVisible();
});

test('login page can switch to the API key method', async ({ page }) => {
  await page.goto('/login');
  await page.getByRole('button', { name: /use an immich api key/i }).click();
  await expect(page.getByLabel('Immich API key')).toBeVisible();
});
