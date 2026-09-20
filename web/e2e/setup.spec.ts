import { expect, test } from '@playwright/test';
import { ASSET_SUMMARY, installMocks, json, SESSION_USER } from './helpers';

test('completing setup with an admin password lands in the app', async ({ page }) => {
  let configured = false;
  const submitted: Array<Record<string, unknown>> = [];
  await installMocks(page, { assets: [ASSET_SUMMARY] });
  await page.route('**/api/setup/**', (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname;
    if (path === '/api/setup/status') return route.fulfill(json({ configured }));
    if (path === '/api/setup/providers') {
      return route.fulfill(json({ oauth: false, password_login: true, auto_launch: false }));
    }
    if (path !== '/api/setup/complete') return route.fallback();
    submitted.push((request.postDataJSON() as Record<string, unknown>) ?? {});
    configured = true;
    return route.fulfill(json(SESSION_USER));
  });

  await page.goto('/setup');
  await expect(page.getByRole('heading', { name: 'Connect Immich' })).toBeVisible();

  await page.getByLabel('Immich URL').fill('https://immich.example.com');
  await page.getByLabel('Email', { exact: true }).fill('admin@example.com');
  await page.getByLabel('Password', { exact: true }).fill('secret');
  await page.getByRole('button', { name: /connect and claim instance/i }).click();

  await page.waitForURL(/\/photos$/);
  await expect(page.getByText('IMG_0001.ARW')).toBeVisible();
  expect(submitted).toEqual([
    { immich_url: 'https://immich.example.com', email: 'admin@example.com', password: 'secret' }
  ]);
});

test('setup reports rejected admin credentials', async ({ page }) => {
  await installMocks(page);
  await page.route('**/api/setup/**', (route) => {
    const path = new URL(route.request().url()).pathname;
    if (path === '/api/setup/status') return route.fulfill(json({ configured: false }));
    if (path !== '/api/setup/complete') return route.fallback();
    return route.fulfill({
      status: 401,
      contentType: 'application/json',
      body: JSON.stringify({ code: 'invalid_credentials', message: 'invalid credentials' })
    });
  });

  await page.goto('/setup');
  await page.getByLabel('Immich URL').fill('https://immich.example.com');
  await page.getByLabel('Email', { exact: true }).fill('admin@example.com');
  await page.getByLabel('Password', { exact: true }).fill('secret');
  await page.getByRole('button', { name: /connect and claim instance/i }).click();

  await expect(page.getByRole('alert')).toHaveText('Invalid credentials.');
  await expect(page).toHaveURL(/\/setup$/);
});
