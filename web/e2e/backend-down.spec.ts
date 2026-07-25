import { expect, test } from '@playwright/test';

const UNAVAILABLE = "Can't reach the immich-edit server.";

test('shows an unavailable state instead of the setup wizard when the backend is down', async ({
  page
}) => {
  await page.route('**/api/**', (route) =>
    route.fulfill({ status: 502, contentType: 'text/html', body: 'Bad Gateway' })
  );

  await page.goto('/');

  await expect(page.getByText(UNAVAILABLE)).toBeVisible();
  expect(new URL(page.url()).pathname).not.toBe('/setup');
});

test('recovers automatically once the backend comes back', async ({ page }) => {
  let down = true;

  await page.route('**/api/**', (route) => {
    const path = new URL(route.request().url()).pathname;
    if (down) {
      return route.fulfill({ status: 502, contentType: 'text/html', body: 'Bad Gateway' });
    }
    if (path === '/api/setup/status') {
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ configured: false })
      });
    }
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({})
    });
  });

  await page.goto('/');
  await expect(page.getByText(UNAVAILABLE)).toBeVisible();

  down = false;

  await expect(page.getByText(UNAVAILABLE)).toBeHidden({ timeout: 15_000 });
  await expect(page).toHaveURL(/\/setup$/);
});

test('still routes to setup when the backend reports an unconfigured instance', async ({ page }) => {
  await page.route('**/api/**', (route) => {
    const path = new URL(route.request().url()).pathname;
    const body = path === '/api/setup/status' ? { configured: false } : {};
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(body)
    });
  });

  await page.goto('/');

  await expect(page).toHaveURL(/\/setup$/);
  await expect(page.getByText(UNAVAILABLE)).toBeHidden();
});
