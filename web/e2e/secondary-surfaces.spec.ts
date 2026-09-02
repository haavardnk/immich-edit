import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';
import { installMocks, json, SESSION_USER } from './helpers';

const JOB = {
  id: 'job-1',
  kind: 'download_zip',
  status: 'completed',
  target: {},
  params: {},
  total: 2,
  completed: 2,
  failed: 0,
  cancelled_at: null,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:01Z'
};

async function installSecondaryMocks(page: Page): Promise<void> {
  await installMocks(page);
  await page.route('**/api/**', (route) => {
    const path = new URL(route.request().url()).pathname;
    if (path === '/api/auth/sessions') return route.fulfill(json({ sessions: [] }));
    if (path === '/api/admin/users') return route.fulfill(json({ users: [SESSION_USER] }));
    if (path === '/api/admin/instance') {
      return route.fulfill(
        json({
          server_epoch: 1,
          immich_url: 'https://immich.example.com',
          configured_at: '2024-01-01T00:00:00Z'
        })
      );
    }
    if (path === '/api/masks/models') {
      return route.fulfill(
        json({ runtime: 'cpu', enabled: true, models: [], active: {}, semantic_classes: [] })
      );
    }
    if (path === '/api/jobs') return route.fulfill(json([JOB]));
    return route.fallback();
  });
}

async function seriousViolations(page: Page): Promise<string[]> {
  const results = await new AxeBuilder({ page })
    .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
    .analyze();
  return results.violations
    .filter((violation) => violation.impact === 'serious' || violation.impact === 'critical')
    .map((violation) => violation.id);
}

test('login fits mobile and reveals the password', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.route('**/api/**', (route) => {
    const path = new URL(route.request().url()).pathname;
    if (path === '/api/setup/status') return route.fulfill(json({ configured: true }));
    if (path === '/api/auth/me') {
      return route.fulfill({
        status: 401,
        contentType: 'application/json',
        body: JSON.stringify({ code: 'unauthorized', message: 'unauthorized' })
      });
    }
    if (path === '/api/auth/login/password') {
      return route.fulfill({
        status: 401,
        contentType: 'application/json',
        body: JSON.stringify({ code: 'invalid_credentials', message: 'invalid credentials' })
      });
    }
    return route.fulfill(json({}));
  });
  await page.goto('/login');

  const email = page.getByLabel('Immich email', { exact: true });
  const password = page.getByLabel('Immich password', { exact: true });
  await email.focus();
  await page.keyboard.press('Tab');
  await expect(password).toBeFocused();
  await password.fill('secret');
  await page.getByRole('button', { name: 'Show password' }).click();
  await expect(password).toHaveAttribute('type', 'text');
  await email.fill('user@example.com');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByRole('alert')).toHaveText('Invalid credentials.');
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 2)
  ).toBe(true);
  expect(await seriousViolations(page)).toEqual([]);
});

test('setup fits mobile and remains accessible', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.route('**/api/**', (route) => {
    const path = new URL(route.request().url()).pathname;
    if (path === '/api/setup/status') return route.fulfill(json({ configured: false }));
    return route.fulfill(json({}));
  });
  await page.goto('/setup');

  await expect(page.getByRole('heading', { name: 'Connect Immich' })).toBeVisible();
  await expect(page.getByLabel('Immich URL')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Email & password' })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 2)
  ).toBe(true);
  expect(await seriousViolations(page)).toEqual([]);
});

test('route errors use the accessible error surface', async ({ page }) => {
  await installSecondaryMocks(page);
  await page.goto('/missing-route');

  await expect(page.getByRole('heading', { name: 'Something went wrong' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Reload' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Go to Photos' })).toBeVisible();
  expect(await seriousViolations(page)).toEqual([]);
});

test('settings fit the mobile viewport', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installSecondaryMocks(page);
  await page.goto('/settings');

  await expect(page.getByRole('heading', { name: 'Settings', level: 1 })).toBeVisible();
  await page.getByRole('button', { name: 'Administration' }).click();
  const navigation = page.getByRole('navigation', { name: 'Settings' });
  await expect(navigation.getByRole('link')).toHaveCount(2);
  await expect(page.getByRole('button', { name: /App settings/ })).toHaveAttribute(
    'aria-expanded',
    'false'
  );
  await expect(page.getByRole('button', { name: /Account/ })).toHaveAttribute(
    'aria-expanded',
    'false'
  );
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 2)
  ).toBe(true);
});

test('settings deep links and close returns to the previous page', async ({ page }) => {
  await installSecondaryMocks(page);
  await page.goto('/settings#sessions');
  await expect(page.getByRole('button', { name: 'Signed-in devices' })).toHaveAttribute(
    'aria-expanded',
    'true'
  );

  await page.goto('/settings#models');

  await expect(page.getByRole('button', { name: /App settings/ })).toHaveAttribute(
    'aria-expanded',
    'true'
  );
  await expect(page.getByRole('button', { name: /^Mask models/ })).toHaveAttribute(
    'aria-expanded',
    'true'
  );

  await page.goto('/favorites');
  await page.evaluate(() => {
    history.replaceState({ ...history.state, settingsReturnMarker: 'favorites' }, '');
  });
  await page.getByRole('button', { name: 'Settings' }).click();
  await expect(page).toHaveURL('/settings');
  await page.getByRole('link', { name: 'Diagnostics' }).click();
  await expect(page).toHaveURL('/settings/diagnostics');
  await page.getByRole('button', { name: 'Close settings' }).click();
  await expect(page).toHaveURL('/favorites');
  expect(await page.evaluate(() => history.state.settingsReturnMarker)).toBe('favorites');
});

test('jobs drawer stays bounded and restores focus', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installSecondaryMocks(page);
  await page.goto('/photos');

  const trigger = page.getByRole('button', { name: 'Jobs' });
  await trigger.click();

  const drawer = page.getByRole('dialog', { name: 'Jobs' });
  await expect(drawer).toBeVisible();
  await expect(drawer.getByText('completed', { exact: true })).toBeVisible();
  await expect(drawer.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '100');
  await expect(drawer.getByRole('link', { name: 'Download ZIP' })).toBeVisible();

  const bounds = await drawer.boundingBox();
  expect(bounds).not.toBeNull();
  expect(bounds!.x).toBeGreaterThanOrEqual(0);
  expect(bounds!.y).toBeGreaterThanOrEqual(0);
  expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(390);
  expect(bounds!.y + bounds!.height).toBeLessThanOrEqual(844);
  expect(await drawer.evaluate((element) => element.contains(document.activeElement))).toBe(true);

  expect(await seriousViolations(page)).toEqual([]);

  await page.keyboard.press('Escape');
  await expect(drawer).toBeHidden();
  await expect(trigger).toBeFocused();
});
