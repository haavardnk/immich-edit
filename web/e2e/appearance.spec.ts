import { expect, test, type Locator, type Page } from '@playwright/test';
import { DEFAULT_APPEARANCE, gotoAsset, installMocks, json, type MockAppearance } from './helpers';

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
const BLOCK_APP = '**/_app/immutable/**/*.js';

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

  await page.route(BLOCK_APP, (route) => route.abort());
  await page.reload();

  expect(await rootState(page)).toEqual(LIGHT);
  await expect(page.locator('meta[name="theme-color"]')).toHaveAttribute('content', '#ffffff');
  await expect(page.locator('link[rel="icon"]')).toHaveAttribute('href', /^data:image\/svg\+xml,/);
});

function serveAppearance(page: Page, appearance: MockAppearance): Promise<void> {
  return page.route('**/api/appearance', (route) => route.fulfill(json(appearance)));
}

test('the server appearance replaces a stale cache after boot', async ({ page }) => {
  await installMocks(page);
  const booted = page.waitForResponse('**/api/appearance');
  await page.goto('/photos');
  await booted;
  await expect.poll(() => rootState(page)).toEqual(DARK);

  await serveAppearance(page, { ...DEFAULT_APPEARANCE, mode: 'light' });
  await page.route(BLOCK_APP, (route) => route.abort());
  await page.reload();
  expect(await rootState(page)).toEqual(DARK);

  await page.unroute(BLOCK_APP);
  await page.reload();
  await expect.poll(() => rootState(page)).toEqual(LIGHT);
});

test('a theme tile switches the whole palette and saves it', async ({ page }) => {
  const record = recordAppearance();
  await installMocks(page, { onAppearance: record.onAppearance });
  await openAppearance(page);

  await page.getByRole('radio', { name: 'Catppuccin Mocha' }).click();

  await expect
    .poll(() => rootState(page))
    .toEqual({ dark: true, light: false, primary: '#cba6f7', page: '#1e1e2e' });
  await expect(page.locator('html')).toHaveAttribute('data-theme', '');
  await expect
    .poll(() => record.saved.at(-1))
    .toEqual({ ...DEFAULT_APPEARANCE, dark_theme: 'catppuccin-mocha' });
});

async function top(locator: Locator): Promise<number> {
  const box = await locator.boundingBox();
  if (!box) throw new Error('not laid out');
  return box.y;
}

test('each mode has its own accent row under its theme grid', async ({ page }) => {
  const record = recordAppearance();
  await installMocks(page, { onAppearance: record.onAppearance });
  await openAppearance(page);

  const lightGrid = page.getByRole('radiogroup', { name: 'Light theme' });
  const light = page.getByRole('radiogroup', { name: 'Light accent' });
  const darkGrid = page.getByRole('radiogroup', { name: 'Dark theme' });
  const dark = page.getByRole('radiogroup', { name: 'Dark accent' });
  await expect(page.getByRole('radiogroup', { name: /accent/i })).toHaveCount(2);
  expect(await top(lightGrid)).toBeLessThan(await top(light));
  expect(await top(light)).toBeLessThan(await top(darkGrid));
  expect(await top(darkGrid)).toBeLessThan(await top(dark));

  await light.getByRole('radio', { name: 'Teal' }).click();

  await expect
    .poll(() => record.saved.at(-1))
    .toEqual({ ...DEFAULT_APPEARANCE, accents: { light: 'teal' } });
  await expect(light.getByRole('radio', { name: 'Teal' })).toBeChecked();
  await expect(dark.getByRole('radio', { name: 'Violet' })).toBeChecked();
  expect(await rootState(page)).toEqual(DARK);
});

test('the dark accent row offers the dark theme family accents', async ({ page }) => {
  await installMocks(page, {
    appearance: { ...DEFAULT_APPEARANCE, dark_theme: 'tokyo-night' }
  });
  await openAppearance(page);

  const light = page.getByRole('radiogroup', { name: 'Light accent' });
  const dark = page.getByRole('radiogroup', { name: 'Dark accent' });
  await expect(dark.getByRole('radio', { name: 'Cyan' })).toBeVisible();
  await expect(dark.getByRole('radio', { name: 'Violet' })).toHaveCount(0);
  await expect(light.getByRole('radio', { name: 'Violet' })).toBeVisible();
  await expect(light.getByRole('radio', { name: 'Cyan' })).toHaveCount(0);
});

test('a theme that inherits the package palette drops the violet fallback', async ({ page }) => {
  await installMocks(page, {
    appearance: { ...DEFAULT_APPEARANCE, dark_theme: 'immich-dark' }
  });
  await page.goto('/photos');

  await expect
    .poll(() =>
      page.evaluate(() => {
        const style = getComputedStyle(document.documentElement);
        return [
          style.getPropertyValue('--immich-ui-primary-500').trim(),
          style.getPropertyValue('--color-gray-800').trim()
        ];
      })
    )
    .toEqual(['oklch(83.6% .074 258.58)', 'oklch(27.8% .033 256.848)']);
});

function logoStroke(page: Page): Promise<string> {
  return page
    .getByRole('img', { name: 'immich-edit' })
    .first()
    .locator('svg path')
    .first()
    .evaluate((element) => getComputedStyle(element).stroke);
}

const VIOLET = 'rgb(183, 161, 251)';

test('an accent recolours primary, the logo and the favicon but not on-photo strokes', async ({
  page
}) => {
  const record = recordAppearance();
  await installMocks(page, { onAppearance: record.onAppearance });
  await openAppearance(page);

  await page
    .getByRole('radiogroup', { name: 'Dark accent' })
    .getByRole('radio', { name: 'Teal' })
    .click();

  await expect.poll(() => rootState(page)).toEqual({ ...DARK, primary: '#1bcbc4' });
  expect(await logoStroke(page)).toBe('rgb(27, 203, 196)');
  await expect(page.locator('link[rel="icon"]')).toHaveAttribute('href', /%231bcbc4/);
  await expect
    .poll(() => record.saved.at(-1))
    .toEqual({ ...DEFAULT_APPEARANCE, accents: { dark: 'teal' } });

  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Crop' }).click();
  await page.getByRole('button', { name: 'Corner handles' }).click();
  const handle = page.getByRole('button', { name: 'perspective corner 1' });
  await expect(handle).toBeVisible();
  expect(
    await handle.locator('span').evaluate((element) => getComputedStyle(element).backgroundColor)
  ).toBe(VIOLET);
  expect(
    await page
      .locator('polygon[stroke="var(--color-image-accent)"]')
      .evaluate((element) => getComputedStyle(element).stroke)
  ).toBe(VIOLET);

  await openAppearance(page);
  await page.getByRole('radio', { name: 'Light', exact: true }).click();
  await expect.poll(() => rootState(page)).toEqual(LIGHT);
  await page
    .getByRole('radiogroup', { name: 'Light accent' })
    .getByRole('radio', { name: 'Teal' })
    .click();
  await expect.poll(() => rootState(page)).toEqual({ ...LIGHT, primary: '#00756f' });
});

test('the login page paints the cached accent before the app starts', async ({ page }) => {
  await installMocks(page, {
    appearance: { ...DEFAULT_APPEARANCE, accents: { dark: 'coral' } }
  });
  await page.goto('/photos');
  await expect.poll(() => rootState(page)).toEqual({ ...DARK, primary: '#f69472' });

  await page.route('**/api/setup/status', (route) => route.fulfill(json({ configured: true })));
  await page.route('**/api/auth/me', (route) =>
    route.fulfill(json({ code: 'unauthorized', message: 'unauthorized' }, 401))
  );
  await page.route(BLOCK_APP, (route) => route.abort());
  await page.goto('/login');
  expect(await rootState(page)).toEqual({ ...DARK, primary: '#f69472' });
  await expect(page.locator('link[rel="icon"]')).toHaveAttribute('href', /%23f69472/);

  await page.unroute(BLOCK_APP);
  await page.goto('/login');
  await expect(page.getByLabel('Immich email', { exact: true })).toBeVisible();
  expect(await logoStroke(page)).toBe('rgb(246, 148, 114)');
});

test('signing out paints the default theme for the next person', async ({ page }) => {
  await installMocks(page, { appearance: { ...DEFAULT_APPEARANCE, mode: 'light' } });
  await page.goto('/settings#account');
  await expect.poll(() => rootState(page)).toEqual(LIGHT);

  await page.route('**/api/setup/status', (route) => route.fulfill(json({ configured: true })));
  await page.route('**/api/auth/me', (route) =>
    route.fulfill(json({ code: 'unauthorized', message: 'unauthorized' }, 401))
  );
  await page.route(BLOCK_APP, (route) => route.abort());
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
  await page.route(BLOCK_APP, (route) => route.abort());
  await page.reload();
  expect(await rootState(page)).toMatchObject({ light: true, primary: LIGHT.primary });
  await expect(page.locator('#immich-edit-theme')).toHaveCount(0);
  await expect(page.locator('html')).not.toHaveAttribute('data-theme');
  await expect(page.locator('link[rel="icon"]')).not.toHaveAttribute('href', /^data:/);

  await page.unroute(BLOCK_APP);
  await page.reload();
  await expect.poll(() => rootState(page)).toEqual(LIGHT);
  await expect(page.locator('html')).toHaveAttribute('data-theme', '');
});
