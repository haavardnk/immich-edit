import { expect, test, type Page } from '@playwright/test';
import { json, SESSION_USER, settingsGroup } from './helpers';

const MODEL = {
  id: 'sam2_tiny',
  name: 'SAM 2 Tiny',
  kind: 'click',
  tier: 'recommended',
  license: 'Apache-2.0',
  source: 'facebook/sam2-hiera-tiny',
  notes: 'Click to add or remove parts of a mask.',
  size_bytes: 200,
  input_edge: 1024,
  gpu_ms: 729,
  gpu_mb: 417,
  cpu_ms: 3200,
  cpu_mb: 700,
  installed: false,
  installing: false,
  progress_bytes: 0,
  install_error: null
};

async function installSettingsMocks(
  page: Page,
  models: () => Array<typeof MODEL>,
  onInstall?: () => void,
  onCancel?: () => void
): Promise<void> {
  await page.route('**/api/**', (route) => {
    const p = new URL(route.request().url()).pathname;
    if (p === '/api/setup/status') return route.fulfill(json({ configured: true }));
    if (p === '/api/auth/me') return route.fulfill(json(SESSION_USER));
    if (p === '/api/auth/sessions') return route.fulfill(json({ sessions: [] }));
    if (p === '/api/export-presets') return route.fulfill(json([]));
    if (p === '/api/masks/models')
      return route.fulfill(
        json({ runtime: 'cpu', enabled: true, models: models(), active: {}, semantic_classes: [] })
      );
    if (p.startsWith('/api/admin/models/') && p.endsWith('/install')) {
      onCancel?.();
      return route.fulfill({ status: 204 });
    }
    if (p.startsWith('/api/admin/models/')) {
      onInstall?.();
      return route.fulfill({ status: 202, contentType: 'application/json', body: '{}' });
    }
    return route.fulfill(json({}));
  });
}

test('the models section lists models with their install state', async ({ page }) => {
  await installSettingsMocks(page, () => [MODEL]);
  await page.goto('/settings/administration#models');

  await expect(page).toHaveURL('/settings/administration#models');
  const group = settingsGroup(page, 'Models');
  await expect(group.getByText('SAM 2 Tiny')).toBeVisible();
  await expect(group.getByText('Runtime cpu')).toBeVisible();
  await expect(group.getByRole('button', { name: 'Download SAM 2 Tiny' })).toBeVisible();
  await expect(group.getByText(/installed/)).toHaveCount(0);
});

test('download button shows install progress and surfaces failures', async ({ page }) => {
  let current = { ...MODEL };
  await installSettingsMocks(
    page,
    () => [current],
    () => {
      current = { ...MODEL, installing: true, progress_bytes: 100 };
    }
  );
  await page.goto('/settings/administration');
  await page.getByRole('button', { name: 'Download SAM 2 Tiny' }).click();

  await expect(
    page.getByRole('progressbar', { name: 'SAM 2 Tiny download progress' })
  ).toHaveAttribute('aria-valuenow', '50');

  current = { ...MODEL, install_error: 'download server returned 404' };
  await expect(page.getByText('Download failed: download server returned 404')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Retry' })).toBeVisible();
});

test('a running download can be cancelled', async ({ page }) => {
  let current = { ...MODEL };
  const cancels: string[] = [];
  await installSettingsMocks(
    page,
    () => [current],
    () => {
      current = { ...MODEL, installing: true, progress_bytes: 100 };
    },
    () => {
      cancels.push(current.id);
      current = { ...MODEL };
    }
  );
  await page.goto('/settings/administration');
  await page.getByRole('button', { name: 'Download SAM 2 Tiny' }).click();
  await expect(
    page.getByRole('progressbar', { name: 'SAM 2 Tiny download progress' })
  ).toBeVisible();

  await page.getByRole('button', { name: 'Cancel download SAM 2 Tiny' }).click();

  await expect(page.getByRole('button', { name: 'Download SAM 2 Tiny' })).toBeEnabled();
  await expect(
    page.getByRole('progressbar', { name: 'SAM 2 Tiny download progress' })
  ).toBeHidden();
  expect(cancels).toEqual(['sam2_tiny']);
});
