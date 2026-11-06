import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';
import { ASSET_ID, DEFAULT_APPEARANCE, gotoAsset, installMocks, openFilters } from './helpers';

const IMPACTS = new Set(['serious', 'critical']);

async function audit(page: Parameters<typeof gotoAsset>[0]): Promise<string[]> {
  const results = await new AxeBuilder({ page })
    .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
    .analyze();
  return results.violations
    .filter((v) => IMPACTS.has(v.impact ?? ''))
    .map(
      (v) =>
        `${v.id}: ${v.nodes.map((n) => `${n.target.join(' ')} (${n.failureSummary})`).join(', ')}`
    );
}

test('the photo grid has no serious accessibility violations', async ({ page }) => {
  await installMocks(page);
  await page.goto('/photos');
  await expect(page.getByRole('link').first()).toBeVisible();

  expect(await audit(page)).toEqual([]);
});

for (const [theme, path] of [
  ['immich-edit-light', '/photos'],
  ['immich-edit-light', '/settings#appearance'],
  ['immich-edit-light', '/settings#layout'],
  ['catppuccin-latte', '/photos'],
  ['catppuccin-latte', '/settings#appearance'],
  ['one-light', '/photos'],
  ['one-light', '/settings#appearance'],
  ['everforest-light', '/photos'],
  ['everforest-light', '/settings#appearance']
]) {
  test(`${path} has no serious accessibility violations in ${theme}`, async ({ page }) => {
    await installMocks(page, {
      appearance: { ...DEFAULT_APPEARANCE, mode: 'light', light_theme: theme }
    });
    await page.goto(path);
    await expect(page.locator('html')).toHaveClass(/\blight\b/);
    await expect(page.getByRole('link').first()).toBeVisible();

    expect(await audit(page)).toEqual([]);
  });
}

test('open browse filters have no serious accessibility violations', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installMocks(page);
  await page.goto('/photos');

  await openFilters(page, 'Date', 'File', 'Camera', 'Place');
  await expect(page.getByRole('button', { name: 'City', exact: true })).toBeVisible();
  expect(await audit(page)).toEqual([]);
});

test('saved views and the save form have no serious accessibility violations', async ({ page }) => {
  await installMocks(page, {
    savedViews: [
      {
        id: 'view-1',
        name: 'Favorites',
        url: '/photos?favorite=1',
        created_at: '2026-01-01T00:00:00Z',
        updated_at: '2026-01-01T00:00:00Z'
      }
    ]
  });
  await page.goto('/photos?favorite=1');

  const panel = await openFilters(page);
  await panel.getByRole('button', { name: 'Save view', exact: true }).click();
  await expect(panel.getByRole('textbox', { name: 'View name' })).toBeFocused();
  const saved = page.getByRole('list', { name: 'Saved views' }).getByRole('link');
  await expect(saved).toHaveAttribute('aria-current', 'page');
  await expect(saved).toHaveAttribute('tabindex', '0');
  await expect(saved).not.toHaveAttribute('type');
  expect(await audit(page)).toEqual([]);
});

test('the loupe and mobile action menu have no serious accessibility violations', async ({
  page
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installMocks(page);
  await page.goto('/photos');
  await expect(page.locator(`a[href^="/assets/${ASSET_ID}?"]`)).toBeVisible();

  await page.getByLabel('Quick review').click();
  await page.getByRole('button', { name: 'More loupe actions' }).click();
  await expect(page.getByRole('button', { name: /^Clipping overlay/ })).toBeVisible();

  expect(await audit(page)).toEqual([]);
});

test('the editor has no serious accessibility violations', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  expect(await audit(page)).toEqual([]);
});

test('the hue wheel reports hue and saturation together', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Color Grading', exact: true }).click();
  const wheel = page.getByRole('slider', { name: 'Hue and saturation wheel' }).first();
  await expect(wheel).toHaveAttribute('aria-valuetext', 'hue 0°, saturation 0%');

  await wheel.focus();
  await wheel.press('ArrowRight');
  await wheel.press('ArrowUp');
  await expect(wheel).toHaveAttribute('aria-valuetext', 'hue 1°, saturation 1%');

  expect(await audit(page)).toEqual([]);
});

test('open shared overlays stay accessible and inside the viewport', async ({ page }) => {
  await page.setViewportSize({ width: 1024, height: 768 });
  await installMocks(page);
  await page.goto('/photos');

  await page.getByRole('button', { name: 'Keyboard shortcuts' }).click();
  const dialog = page.getByRole('dialog', { name: 'Keyboard shortcuts' });
  await expect(dialog).toBeVisible();
  expect(await audit(page)).toEqual([]);

  const dialogBox = await dialog.boundingBox();
  expect(dialogBox).not.toBeNull();
  expect(dialogBox!.x).toBeGreaterThanOrEqual(0);
  expect(dialogBox!.y).toBeGreaterThanOrEqual(0);
  expect(dialogBox!.x + dialogBox!.width).toBeLessThanOrEqual(1024);
  expect(dialogBox!.y + dialogBox!.height).toBeLessThanOrEqual(768);

  await page.keyboard.press('Escape');
  await gotoAsset(page);
  await page.getByRole('button', { name: 'Zoom', exact: true }).click();

  const popover = page.getByRole('button', { name: 'Zoom In' }).locator('..');
  await expect(popover).toBeVisible();
  expect(await audit(page)).toEqual([]);

  const popoverBox = await popover.boundingBox();
  expect(popoverBox).not.toBeNull();
  expect(popoverBox!.x).toBeGreaterThanOrEqual(0);
  expect(popoverBox!.y).toBeGreaterThanOrEqual(0);
  expect(popoverBox!.x + popoverBox!.width).toBeLessThanOrEqual(1024);
  expect(popoverBox!.y + popoverBox!.height).toBeLessThanOrEqual(768);
});

test('visible feedback has accessible live-region semantics', async ({ page }) => {
  await installMocks(page, { smartFails: true });
  await page.goto('/search?q=red%20car');

  const message = 'Smart search unavailable, showing filename matches';
  const toast = page.getByRole('status').filter({ hasText: message });
  await expect(toast).toBeVisible();
  await expect(toast).toHaveAttribute('aria-live', 'polite');
  expect(await audit(page)).toEqual([]);
});

test('the login page has no serious accessibility violations', async ({ page }) => {
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

  await page.goto('/login');
  await expect(page.getByLabel('Immich email', { exact: true })).toBeVisible();

  expect(await audit(page)).toEqual([]);
});
