import { expect, test, type Page } from '@playwright/test';
import { seriousViolations } from './axeHelpers';
import { mockDiagnostics } from './diagnosticsMocks';
import {
  installMocks,
  jobsPanel,
  json,
  openAccountMenu,
  SESSION_USER,
  settingsGroup,
  settingsPages,
  ACCOUNT_NAME
} from './helpers';

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

  await expect(page.getByRole('heading', { name: 'Set up immich-edit' })).toBeVisible();
  await page.getByLabel('Immich URL').fill('https://immich.example.com');
  expect(await seriousViolations(page)).toEqual([]);
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page.getByRole('radio', { name: 'Email & password' })).toBeChecked();
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
  await expect(
    page.locator('header[data-app-bar]').getByRole('button', { name: 'Back', exact: true })
  ).toBeVisible();
  for (const name of ['Preferences', 'Account', /^Administration/, 'Diagnostics']) {
    await expect(settingsPages(page).getByRole('link', { name })).toBeVisible();
  }
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 2)
  ).toBe(true);
});

test('settings sign-out asks first while a transfer runs', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await installSecondaryMocks(page);
  await page.route('**/api/jobs/**', (route) => route.fulfill({ status: 404 }));
  await page.route('**/api/jobs', (route) =>
    route.fulfill(json([{ ...JOB, status: 'running', completed: 1 }]))
  );
  const logouts: string[] = [];
  page.on('request', (request) => {
    if (request.url().includes('/api/auth/logout')) logouts.push(request.url());
  });
  await page.goto('/settings/account');

  await page.getByRole('main').getByRole('button', { name: 'Sign out', exact: true }).click();
  const confirm = page.getByRole('dialog', { name: 'Sign out?' });
  await expect(confirm).toBeVisible();
  await expect(confirm).toContainText('1 export or upload job still running');

  await confirm.getByRole('button', { name: 'Cancel', exact: true }).click();
  await expect(confirm).toBeHidden();
  expect(logouts).toEqual([]);
});

test('settings deep links and close returns to the previous page', async ({ page }) => {
  await installSecondaryMocks(page);
  await page.goto('/settings/account#sessions');
  await expect(page).toHaveURL('/settings/account#sessions');
  await expect(settingsGroup(page, 'Signed-in devices')).toBeVisible();

  await page.goto('/settings/administration#models');
  await expect(page).toHaveURL('/settings/administration#models');
  await expect(settingsGroup(page, 'Models')).toBeVisible();

  await page.goto('/favorites');
  await expect(page.getByRole('button', { name: 'Close settings' })).toHaveCount(0);
  const menu = await openAccountMenu(page);
  await page.evaluate(() => {
    history.replaceState({ ...history.state, settingsReturnMarker: 'favorites' }, '');
  });
  await menu.getByRole('button', { name: 'Settings' }).click();
  await expect(page).toHaveURL('/settings/preferences');
  await settingsPages(page).getByRole('link', { name: 'Diagnostics' }).click();
  await expect(page).toHaveURL('/settings/diagnostics');
  await page.getByRole('button', { name: 'Close settings' }).click();
  await expect(page).toHaveURL('/favorites');
  expect(await page.evaluate(() => history.state.settingsReturnMarker)).toBe('favorites');
});

test('admins can turn off automatic lens corrections', async ({ page }) => {
  const puts: unknown[] = [];
  let lensAuto = true;
  await installSecondaryMocks(page);
  await page.route('**/api/admin/render-defaults', (route) => {
    if (route.request().method() === 'PUT') {
      const body = route.request().postDataJSON() as { lens_auto: boolean };
      puts.push(body);
      lensAuto = body.lens_auto;
    }
    return route.fulfill(json({ lens_auto: lensAuto }));
  });
  await page.goto('/settings/administration#lens-auto');

  const toggle = page.getByRole('switch', { name: 'Automatic lens corrections' });
  await expect(toggle).toBeChecked();
  const listed = page.waitForRequest((req) => new URL(req.url()).pathname === '/api/edits');
  await toggle.click();

  await expect(toggle).not.toBeChecked();
  expect(puts).toEqual([{ lens_auto: false }]);
  await listed;
});

async function expectBoundedJobsDrawer(
  page: Page,
  viewport: { width: number; height: number },
  downloadName: string
): Promise<void> {
  const drawer = jobsPanel(page);
  await expect(drawer).toBeVisible();
  await expect(drawer.getByText('Done, 2 photos', { exact: true })).toBeVisible();
  await expect(drawer.getByRole('progressbar')).toHaveCount(0);
  const download = drawer.getByRole('link', { name: downloadName });
  await expect(download).toBeVisible();
  await expect(download).not.toHaveAttribute('type');

  await expect
    .poll(async () => {
      const bounds = await drawer.boundingBox();
      return (
        bounds !== null &&
        bounds.x >= 0 &&
        bounds.y >= 0 &&
        bounds.x + bounds.width <= viewport.width &&
        bounds.y + bounds.height <= viewport.height
      );
    })
    .toBe(true);
  expect(await drawer.evaluate((element) => element.contains(document.activeElement))).toBe(true);

  expect(await seriousViolations(page)).toEqual([]);

  await page.keyboard.press('Escape');
  await expect(drawer).toBeHidden();
}

test('jobs drawer stays bounded and restores focus on desktop', async ({ page }) => {
  const viewport = { width: 1280, height: 800 };
  await page.setViewportSize(viewport);
  await installSecondaryMocks(page);
  await page.goto('/photos');

  const trigger = page.getByRole('button', { name: 'Jobs', exact: true });
  await trigger.click();

  await expectBoundedJobsDrawer(page, viewport, 'Download ZIP');
  await expect(trigger).toBeFocused();
});

test('jobs drawer opened from the account drawer stays bounded on a phone', async ({ page }) => {
  const viewport = { width: 390, height: 844 };
  await page.setViewportSize(viewport);
  await installSecondaryMocks(page);
  await page.goto('/photos');

  const trigger = page.getByRole('button', { name: ACCOUNT_NAME });
  await trigger.click();
  await page
    .getByRole('dialog', { name: 'Account' })
    .getByRole('button', { name: 'Jobs', exact: true })
    .click();

  await expectBoundedJobsDrawer(page, viewport, 'Save ZIP');
  await expect(trigger).toBeFocused();
});

test('diagnostics break render time down by stage and plan', async ({ page }) => {
  await installSecondaryMocks(page);
  await mockDiagnostics(page);
  await page.goto('/settings/diagnostics');

  await expect(page.getByText('3.2.2', { exact: true })).toBeVisible();
  await expect(page.getByText('8.00GiB dedicated')).toBeVisible();
  await expect(page.getByText('7.20GiB')).toBeVisible();
  const table = page.getByRole('table', { name: 'GPU render stages' });
  await expect(table.getByRole('columnheader', { name: 'GPU p50' })).toBeVisible();
  await expect(
    table.getByRole('row', { name: /demosaic 300µs 500µs 2\.1ms 2\.6ms/ })
  ).toBeVisible();
  await expect(table.getByRole('row', { name: /encode 9\.4ms 12\.0ms — —/ })).toBeVisible();
  await expect(page.getByRole('table', { name: 'CPU render stages' })).toHaveCount(0);
  const plans = page.getByRole('table', { name: 'Render plans' });
  await expect(plans.getByRole('row', { name: /GPU Single pass 14 1/ })).toBeVisible();
  await expect(plans.getByRole('row', { name: /GPU Display tiles 2048px 2 24/ })).toBeVisible();
  await expect(page.getByText('75% (3 / 4)')).toBeVisible();
  await expect(page.getByText('GPU stage times are off')).toHaveCount(0);

  expect(await seriousViolations(page)).toEqual([]);
});
