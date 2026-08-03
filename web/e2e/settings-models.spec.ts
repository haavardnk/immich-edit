import { expect, test, type Page } from '@playwright/test';
import { json, SESSION_USER } from './helpers';

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
  onInstall?: () => void
): Promise<void> {
  await page.route('**/api/**', (route) => {
    const p = new URL(route.request().url()).pathname;
    if (p === '/api/setup/status') return route.fulfill(json({ configured: true }));
    if (p === '/api/auth/me') return route.fulfill(json(SESSION_USER));
    if (p === '/api/masks/models')
      return route.fulfill(
        json({ runtime: 'cpu', enabled: true, models: models(), active: {}, semantic_classes: [] })
      );
    if (p.startsWith('/api/admin/models/')) {
      onInstall?.();
      return route.fulfill({ status: 202, contentType: 'application/json', body: '{}' });
    }
    return route.fulfill(json({}));
  });
}

test('mask models section starts collapsed and expands on click', async ({ page }) => {
  await installSettingsMocks(page, () => [MODEL]);
  await page.goto('/settings');

  const header = page.getByRole('button', { name: /mask models/i });
  await expect(header).toBeVisible();
  await expect(page.getByText('0 of 1 installed')).toBeVisible();
  await expect(page.getByText('SAM 2 Tiny')).toBeHidden();

  await header.click();
  await expect(page.getByText('SAM 2 Tiny')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Download', exact: true })).toBeVisible();
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
  await page.goto('/settings');
  await page.getByRole('button', { name: /mask models/i }).click();
  await page.getByRole('button', { name: 'Download', exact: true }).click();

  await expect(page.getByRole('button', { name: '50%', exact: true })).toBeVisible();
  await expect(page.getByText('Downloading 50%')).toBeVisible();

  current = { ...MODEL, install_error: 'download server returned 404' };
  await expect(page.getByText('Download failed: download server returned 404')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Retry' })).toBeVisible();
});
