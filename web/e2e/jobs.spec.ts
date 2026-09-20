import { expect, test } from '@playwright/test';
import { installMocks, json } from './helpers';

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
