import { expect, test } from '@playwright/test';
import { openBulkEdits, pick } from './browse';
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

  const drawer = page.getByRole('complementary', { name: 'Jobs' });
  await expect(
    drawer.getByRole('progressbar', { name: 'Export to Immich progress' })
  ).toBeVisible();
  await drawer.getByRole('button', { name: 'Cancel job' }).click();

  await expect(drawer.getByText('Cancelled', { exact: true })).toBeVisible();
  await expect(drawer.getByRole('button', { name: 'Cancel job' })).toHaveCount(0);
  expect(cancelled).toBe(true);
});

test('the drawer lists the metadata each photo could not copy', async ({ page }) => {
  const done = {
    ...RUNNING_JOB,
    id: 'job-2',
    kind: 'download_zip',
    status: 'completed',
    total: 1,
    completed: 1
  };
  await installMocks(page);
  await page.route('**/api/jobs', (route) =>
    route.request().method() === 'GET' ? route.fulfill(json([done])) : route.fallback()
  );
  await page.route('**/api/jobs/job-2', (route) =>
    route.fulfill(
      json({
        job: done,
        items: [
          {
            id: 'item-1',
            job_id: 'job-2',
            asset_id: 'asset-1',
            status: 'completed',
            error: null,
            result: {
              filename: 'IMG_0001_edit.jpg',
              bytes: 10,
              warnings: ['Metadata not copied: no readable EXIF in the original']
            },
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
  await page.getByRole('button', { name: 'Jobs' }).click();
  const drawer = page.getByRole('complementary', { name: 'Jobs' });
  await drawer.getByRole('button', { name: 'Download ZIP details' }).click();

  await expect(drawer.getByText('IMG_0001_edit.jpg')).toBeVisible();
  await expect(
    drawer.getByText('Metadata not copied: no readable EXIF in the original')
  ).toBeVisible();
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
  const assets = numberedAssets(2);
  await installMocks(page, {
    assets,
    edits: assets.map((asset) => ({ id: asset.id, hash: 'h1', updated_at: '2024-05-01T00:00:00Z' }))
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
  const edits = await openBulkEdits(page);
  await edits.getByRole('button', { name: 'Reset edits' }).click();

  const progress = page.getByRole('progressbar', { name: 'Apply Preset progress' });
  await expect(progress).toBeVisible();
  await expect(page.getByTestId('job-toast-outcome')).toHaveText('0 of 2');
  await expect(page).toHaveTitle('(0%) immich-edit');
  await expect(page.getByRole('complementary', { name: 'Jobs' })).toHaveCount(0);

  await expect.poll(() => deliver !== null).toBe(true);
  deliver?.();
  await expect(page.getByTestId('job-toast-outcome')).toHaveText('1 of 2 failed');
  await expect(progress).toBeHidden();
  await expect(page.getByTestId('jobs-attention')).toBeVisible();
  await expect(page).toHaveTitle('(!) immich-edit');

  await page.getByRole('button', { name: 'Details' }).click();
  const drawer = page.getByRole('complementary', { name: 'Jobs' });
  await expect(drawer.getByText('render failed')).toBeVisible();
  await expect(page.getByTestId('job-toast-outcome')).toHaveCount(0);
  await expect(page.getByTestId('jobs-attention')).toHaveCount(0);
  await expect(page).toHaveTitle('immich-edit');
});

test('the jobs panel docks beside the grid and leaves it usable', async ({ page }) => {
  const assets = numberedAssets(3);
  await installMocks(page, { assets });
  await page.route('**/api/jobs', (route) =>
    route.request().method() === 'GET'
      ? route.fulfill(json([{ ...RUNNING_JOB, status: 'completed', completed: 4 }]))
      : route.fallback()
  );
  await page.goto('/photos');
  const trigger = page.getByRole('button', { name: 'Jobs', exact: true });
  await expect(trigger).toHaveAttribute('aria-expanded', 'false');
  await trigger.click();

  const panel = page.getByRole('complementary', { name: 'Jobs' });
  await expect(panel).toBeVisible();
  await expect(trigger).toHaveAttribute('aria-expanded', 'true');
  await expect(page.getByRole('dialog')).toHaveCount(0);
  await expect
    .poll(async () => {
      const [grid, docked] = await Promise.all([
        page.locator('main[data-page]').boundingBox(),
        panel.boundingBox()
      ]);
      return (grid?.x ?? 0) + (grid?.width ?? 0) - (docked?.x ?? 0);
    })
    .toBeLessThanOrEqual(1);

  await pick(page, assets[0]?.originalFileName ?? '');
  await expect(page.getByText('1 selected')).toBeVisible();
  await expect(panel).toBeVisible();

  await trigger.click();
  await expect(panel).toBeHidden();
  await expect(trigger).toHaveAttribute('aria-expanded', 'false');
});
