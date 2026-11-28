import { expect, test } from '@playwright/test';
import { tile } from './browse';
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
  await expect(page.getByRole('heading', { name: 'Set up immich-edit' })).toBeVisible();

  await page.getByLabel('Immich URL').fill('https://immich.example.com');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page.getByText('https://immich.example.com')).toBeVisible();
  await page.getByLabel('Email', { exact: true }).fill('admin@example.com');
  await page.getByLabel('Password', { exact: true }).fill('secret');
  await page.getByRole('button', { name: /connect and claim instance/i }).click();

  await page.waitForURL(/\/photos$/);
  await expect(tile(page, ASSET_SUMMARY.originalFileName)).toBeVisible();
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
  await page.getByRole('button', { name: 'Continue' }).click();
  await page.getByLabel('Email', { exact: true }).fill('admin@example.com');
  await page.getByLabel('Password', { exact: true }).fill('secret');
  await page.getByRole('button', { name: /connect and claim instance/i }).click();

  await expect(page.getByRole('alert')).toHaveText('Invalid credentials.');
  await expect(page).toHaveURL(/\/setup$/);
});

test('an Immich that does not answer keeps setup on the server step', async ({ page }) => {
  let degraded = true;
  await installMocks(page);
  await page.route('**/api/setup/**', (route) => {
    const path = new URL(route.request().url()).pathname;
    if (path === '/api/setup/status') return route.fulfill(json({ configured: false }));
    if (path === '/api/setup/providers') {
      return route.fulfill(
        json({ oauth: false, password_login: true, auto_launch: false, button_text: '', degraded })
      );
    }
    return route.fallback();
  });

  await page.goto('/setup');
  await page.getByLabel('Immich URL').fill('https://immich.example.com');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page.getByText("Immich didn't answer at this address.")).toBeVisible();
  await expect(
    page.getByRole('list', { name: 'Setup steps' }).getByRole('listitem').first()
  ).toHaveAttribute('aria-current', 'step');

  degraded = false;
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page.getByLabel('Email', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Edit Immich server' }).click();
  await expect(page.getByLabel('Immich URL')).toHaveValue('https://immich.example.com');
  await expect(page.getByText("Immich didn't answer at this address.")).toHaveCount(0);
});

test.describe('setup on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('each step keeps its primary action at the bottom of the screen', async ({ page }) => {
    await installMocks(page);
    await page.route('**/api/setup/status', (route) => route.fulfill(json({ configured: false })));
    await page.goto('/setup');

    const viewport = page.viewportSize()?.height ?? 0;
    const bottomOf = async (name: RegExp) => {
      const box = await page.getByRole('button', { name }).boundingBox();
      return (box?.y ?? 0) + (box?.height ?? 0);
    };
    await page.getByLabel('Immich URL').fill('https://immich.example.com');
    expect(await bottomOf(/^Continue$/)).toBeGreaterThan(viewport - 60);
    await page.getByRole('button', { name: 'Continue' }).tap();
    await expect(page.getByLabel('Email', { exact: true })).toBeVisible();
    expect(await bottomOf(/connect and claim instance/i)).toBeGreaterThan(viewport - 60);
  });
});
