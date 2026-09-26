import { expect, test } from '@playwright/test';
import { installMocks, json, numberedAssets } from './helpers';

const RUNNING_JOB = {
  id: 'job-1',
  kind: 'export_immich',
  status: 'running',
  target: {},
  params: {},
  total: 4,
  completed: 1,
  failed: 0,
  cancelled_at: null,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:01Z'
};

test('cancelling a running job from the drawer reports the new status', async ({ page }) => {
  let cancelled = false;
  let deliverEvent: (() => void) | null = null;
  await installMocks(page);
  await page.route('**/api/jobs/**', async (route) => {
    const path = new URL(route.request().url()).pathname;
    if (path === '/api/jobs/job-1/cancel') {
      cancelled = true;
      deliverEvent?.();
      return route.fulfill(json({ ok: true }));
    }
    if (path !== '/api/jobs/job-1/events') return route.fallback();
    if (!cancelled) {
      await new Promise<void>((resolve) => {
        deliverEvent = resolve;
      });
    }
    const job = { ...RUNNING_JOB, status: 'cancelled', cancelled_at: '2024-01-01T00:00:02Z' };
    return route.fulfill({
      status: 200,
      contentType: 'text/event-stream',
      body: `event: job\ndata: ${JSON.stringify(job)}\n\n`
    });
  });
  await page.route('**/api/jobs', (route) => {
    if (route.request().method() !== 'GET') return route.fallback();
    return route.fulfill(json([cancelled ? { ...RUNNING_JOB, status: 'cancelled' } : RUNNING_JOB]));
  });

  await page.goto('/photos');
  await page.getByRole('button', { name: 'Jobs' }).click();

  const drawer = page.getByRole('dialog', { name: 'Jobs' });
  await expect(drawer.getByText('running', { exact: true })).toBeVisible();
  await drawer.getByRole('button', { name: 'Cancel job' }).click();

  await expect(drawer.getByText('cancelled', { exact: true })).toBeVisible();
  await expect(drawer.getByRole('button', { name: 'Cancel job' })).toHaveCount(0);
  expect(cancelled).toBe(true);
});

test('a bulk job reports progress in a toast and leaves the drawer closed', async ({ page }) => {
  const queued = {
    ...RUNNING_JOB,
    id: 'job-9',
    kind: 'apply_preset',
    status: 'running',
    total: 2,
    completed: 0
  };
  const finished = { ...queued, status: 'completed', completed: 1, failed: 1 };
  let deliver: (() => void) | null = null;
  await installMocks(page, {
    assets: numberedAssets(2),
    presets: [{ id: 'preset-warm', name: 'Warm', group_name: null, manifest: { ops: {} } }]
  });
  await page.route('**/api/jobs', (route) =>
    route.request().method() === 'POST'
      ? route.fulfill(json(queued))
      : route.fulfill(json(deliver === null ? [queued] : [finished]))
  );
  await page.route('**/api/jobs/job-9/events', async (route) => {
    await new Promise<void>((resolve) => {
      deliver = resolve;
    });
    return route.fulfill({
      status: 200,
      contentType: 'text/event-stream',
      body: `event: job\ndata: ${JSON.stringify(finished)}\n\n`
    });
  });
  await page.route('**/api/jobs/job-9', (route) =>
    route.fulfill(
      json({
        job: finished,
        items: [
          {
            id: 'item-1',
            job_id: 'job-9',
            asset_id: 'asset-1',
            status: 'failed',
            error: 'render failed',
            result: null,
            idempotency_key: null,
            attempts: 1,
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:01Z'
          }
        ]
      })
    )
  );

  await page.goto('/photos');
  const select = page.getByRole('button', { name: 'Select', exact: true });
  await select.nth(0).click();
  await select.nth(0).click();
  await page.getByRole('button', { name: 'Edit and export selected' }).click();
  const dialog = page.getByRole('dialog', { name: 'Edit and export selected' });
  const section = dialog.getByRole('button', { name: 'Presets', exact: true });
  if ((await section.getAttribute('aria-expanded')) !== 'true') await section.click();
  await dialog.getByRole('combobox', { name: 'Select a preset…' }).click();
  await page.getByRole('option', { name: 'Warm', exact: true }).click();
  await dialog.getByRole('button', { name: 'Apply Warm to 2' }).click();

  const progress = page.getByRole('progressbar', { name: 'Apply Preset progress' });
  await expect(progress).toBeVisible();
  await expect(page.getByTestId('job-toast-outcome')).toHaveText('0 of 2');
  await expect(page.getByRole('dialog', { name: 'Jobs' })).toHaveCount(0);

  await expect.poll(() => deliver !== null).toBe(true);
  deliver?.();
  await expect(page.getByTestId('job-toast-outcome')).toHaveText('1 of 2 failed');
  await expect(progress).toBeHidden();
  await expect(page.getByTestId('jobs-attention')).toBeVisible();

  await page.getByRole('button', { name: 'Details' }).click();
  const drawer = page.getByRole('dialog', { name: 'Jobs' });
  await expect(drawer.getByText('render failed')).toBeVisible();
  await expect(page.getByTestId('job-toast-outcome')).toHaveCount(0);
  await expect(page.getByTestId('jobs-attention')).toHaveCount(0);
});
