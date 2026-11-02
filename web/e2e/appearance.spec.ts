import { expect, test, type Page } from '@playwright/test';
import { DEFAULT_APPEARANCE, installMocks, type MockAppearance } from './helpers';

interface RootState {
  dark: boolean;
  light: boolean;
  primary: string;
  page: string;
}

function rootState(page: Page): Promise<RootState> {
  return page.evaluate(() => {
    const root = document.documentElement;
    const style = getComputedStyle(root);
    return {
      dark: root.classList.contains('dark'),
      light: root.classList.contains('light'),
      primary: style.getPropertyValue('--immich-ui-primary-500').trim(),
      page: style.getPropertyValue('--immich-ui-light').trim()
    };
  });
}

async function emulateScheme(page: Page, colorScheme: 'light' | 'dark'): Promise<void> {
  await page.emulateMedia({ colorScheme });
  await page.waitForFunction(
    (dark) => matchMedia('(prefers-color-scheme: dark)').matches === dark,
    colorScheme === 'dark'
  );
  await page.evaluate(
    () => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)))
  );
}

async function openAppearance(page: Page): Promise<void> {
  await page.goto('/settings#appearance');
  await expect(page.getByRole('radiogroup', { name: 'Mode' })).toBeVisible();
}

function recordAppearance(): {
  saved: MockAppearance[];
  onAppearance: (body: MockAppearance) => void;
} {
  const saved: MockAppearance[] = [];
  return { saved, onAppearance: (body) => saved.push(body) };
}

const LIGHT = { dark: false, light: true, primary: '#7159b0', page: '#ffffff' };
const DARK = { dark: true, light: false, primary: '#b7a1fb', page: 'oklch(0% 0 0)' };

test('the default dark theme survives an OS switch to light', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'dark' });
  await installMocks(page);
  await page.goto('/photos');
  await expect(page.getByRole('link', { name: 'Photos' })).toBeVisible();

  await emulateScheme(page, 'light');

  expect(await rootState(page)).toEqual(DARK);
});

test('choosing light mode in settings recolours the app and saves it', async ({ page }) => {
  const record = recordAppearance();
  await installMocks(page, { onAppearance: record.onAppearance });
  await openAppearance(page);

  await page.getByRole('radio', { name: 'Light', exact: true }).click();

  await expect.poll(() => rootState(page)).toEqual(LIGHT);
  await expect(page.locator('meta[name="theme-color"]')).toHaveAttribute('content', '#ffffff');
  await expect.poll(() => record.saved.at(-1)).toEqual({ ...DEFAULT_APPEARANCE, mode: 'light' });
});

test('system mode follows the OS scheme', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'light' });
  await installMocks(page, { appearance: { ...DEFAULT_APPEARANCE, mode: 'system' } });
  await page.goto('/photos');
  await expect.poll(() => rootState(page)).toEqual(LIGHT);

  await emulateScheme(page, 'dark');
  expect(await rootState(page)).toEqual(DARK);
  await expect(page.locator('meta[name="theme-color"]')).toHaveAttribute('content', '#000000');

  await emulateScheme(page, 'light');
  expect(await rootState(page)).toEqual(LIGHT);
});

test('the top bar toggle flips the scheme and saves the mode', async ({ page }) => {
  const record = recordAppearance();
  await installMocks(page, { onAppearance: record.onAppearance });
  await page.goto('/photos');

  await page.getByRole('button', { name: 'Switch to light theme' }).click();
  await expect.poll(() => rootState(page)).toEqual(LIGHT);
  await expect.poll(() => record.saved.at(-1)?.mode).toBe('light');

  await page.getByRole('button', { name: 'Switch to dark theme' }).click();
  await expect.poll(() => rootState(page)).toEqual(DARK);
  await expect.poll(() => record.saved.at(-1)?.mode).toBe('dark');
});

test('a reload paints the saved theme before the app starts', async ({ page }) => {
  await installMocks(page);
  await page.goto('/photos');
  await page.getByRole('button', { name: 'Switch to light theme' }).click();
  await expect.poll(() => rootState(page)).toEqual(LIGHT);

  await page.route('**/_app/immutable/**/*.js', (route) => route.abort());
  await page.reload();

  expect(await rootState(page)).toEqual(LIGHT);
  await expect(page.locator('meta[name="theme-color"]')).toHaveAttribute('content', '#ffffff');
  await expect(page.locator('link[rel="icon"]')).toHaveAttribute('href', /^data:image\/svg\+xml,/);
});

test('the server appearance wins over a stale cache', async ({ page }) => {
  await installMocks(page, { appearance: { ...DEFAULT_APPEARANCE, mode: 'light' } });
  await page.goto('/photos');

  await expect.poll(() => rootState(page)).toEqual(LIGHT);
});

test('signing out paints the default theme for the next person', async ({ page }) => {
  await installMocks(page, { appearance: { ...DEFAULT_APPEARANCE, mode: 'light' } });
  await page.goto('/settings#account');
  await expect.poll(() => rootState(page)).toEqual(LIGHT);

  await page.route('**/api/setup/status', (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '{"configured":true}' })
  );
  await page.route('**/api/auth/me', (route) =>
    route.fulfill({
      status: 401,
      contentType: 'application/json',
      body: '{"code":"unauthorized","message":"unauthorized"}'
    })
  );
  await page.route('**/_app/immutable/**/*.js', (route) => route.abort());
  await page.getByRole('button', { name: 'Sign out' }).click();
  await page.waitForURL('**/login');

  expect(await rootState(page)).toEqual(DARK);
  await expect(page.locator('meta[name="theme-color"]')).toHaveAttribute('content', '#000000');
});

test('a cache from another theme revision is not painted', async ({ page }) => {
  await installMocks(page, { appearance: { ...DEFAULT_APPEARANCE, mode: 'light' } });
  await page.goto('/photos');
  await expect.poll(() => rootState(page)).toEqual(LIGHT);
  await expect(page.locator('html')).toHaveAttribute('data-theme-revision', /^[0-9a-f]{8}$/);

  await page.evaluate(() => {
    const cache: unknown = JSON.parse(localStorage.getItem('immich-edit:theme') ?? 'null');
    localStorage.setItem(
      'immich-edit:theme',
      JSON.stringify({ ...(cache instanceof Object ? cache : {}), revision: 'stale' })
    );
  });
  await page.route('**/_app/immutable/**/*.js', (route) => route.abort());
  await page.reload();
  expect(await rootState(page)).toMatchObject({ light: true, primary: LIGHT.primary });
  await expect(page.locator('#immich-edit-theme')).toHaveCount(0);
  await expect(page.locator('html')).not.toHaveAttribute('data-theme');
  await expect(page.locator('link[rel="icon"]')).not.toHaveAttribute('href', /^data:/);

  await page.unroute('**/_app/immutable/**/*.js');
  await page.reload();
  await expect.poll(() => rootState(page)).toEqual(LIGHT);
  await expect(page.locator('html')).toHaveAttribute('data-theme', '');
});
