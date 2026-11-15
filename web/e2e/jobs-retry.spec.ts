import { expect, test, type Page } from '@playwright/test';
import { seriousViolations } from './axeHelpers';
import { installMocks, json } from './helpers';
import { mockLibrary, openDrawer } from './touchShell';

const FAILED = {
  id: 'job-r',
  kind: 'export_immich',
  status: 'completed',
  target: {},
  params: {},
  total: 3,
  completed: 2,
  failed: 1,
  cancelled_at: null,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:05Z'
};

const REOPENED = { ...FAILED, status: 'pending', failed: 0, updated_at: '2024-01-01T00:00:10Z' };
const DONE = { ...REOPENED, status: 'completed', completed: 3, updated_at: '2024-01-01T00:00:20Z' };

const ITEM = {
  id: 'item-1',
  job_id: FAILED.id,
  asset_id: 'asset-0001',
  position: 0,
  variant: 0,
  status: 'failed',
  error: 'Upload to Immich timed out after 120 s',
  result: { filename: 'DSCF3981.RAF' },
  idempotency_key: null,
  attempts: 1,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:05Z'
};

async function mockRetry(page: Page): Promise<{ retried: () => boolean; finish: () => void }> {
  let retried = false;
  let done = false;
  let release: () => void = () => {};
  const finished = new Promise<void>((resolve) => {
    release = resolve;
  });
  await page.route('**/api/jobs/**', async (route) => {
    const path = new URL(route.request().url()).pathname;
    if (path === `/api/jobs/${FAILED.id}/retry`) {
      retried = true;
      return route.fulfill(json(REOPENED));
    }
    if (path === `/api/jobs/${FAILED.id}/events`) {
      await finished;
      done = true;
      return route.fulfill({
        status: 200,
        contentType: 'text/event-stream',
        body: `event: job\ndata: ${JSON.stringify(DONE)}\n\n`
      });
    }
    if (path === `/api/jobs/${FAILED.id}`) {
      const status = done ? 'completed' : retried ? 'pending' : ITEM.status;
      return route.fulfill(
        json({
          job: done ? DONE : retried ? REOPENED : FAILED,
          items: [{ ...ITEM, status, error: retried ? null : ITEM.error }]
        })
      );
    }
    return route.fallback();
  });
  await page.route('**/api/jobs', (route) =>
    route.request().method() === 'GET' ? route.fulfill(json([FAILED])) : route.fallback()
  );
  return { retried: () => retried, finish: release };
}

test('a failed job retries its failed photos from the expanded row', async ({ page, context }) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  await installMocks(page);
  const job = await mockRetry(page);
  await page.goto('/photos');
  await page.getByRole('button', { name: 'Jobs', exact: true }).click();

  const panel = page.getByRole('dialog', { name: 'Jobs' });
  await expect(panel.getByText('1 of 3 failed', { exact: true })).toBeVisible();
  await expect(panel.getByRole('button', { name: 'Retry 1 failed' })).toHaveCount(0);
  await panel.getByRole('button', { name: 'Export to Immich details' }).click();
  await expect(panel.getByText('Upload to Immich timed out after 120 s')).toBeVisible();
  expect(await seriousViolations(page)).toEqual([]);

  await panel.getByRole('button', { name: 'Copy errors' }).click();
  await expect
    .poll(() => page.evaluate(() => navigator.clipboard.readText()))
    .toBe(
      'Export to Immich job-r: 1 of 3 failed\nDSCF3981.RAF: Upload to Immich timed out after 120 s'
    );

  await panel.getByRole('button', { name: 'Retry 1 failed' }).click();
  const progress = panel.getByRole('progressbar', { name: 'Export to Immich progress' });
  await expect(progress).toBeVisible();
  expect(job.retried()).toBe(true);
  await expect(panel.getByText('Upload to Immich timed out after 120 s')).toBeHidden();
  const item = panel.getByRole('listitem').filter({ hasText: 'DSCF3981.RAF' });
  await expect(item).toContainText('pending');

  await page.waitForRequest(
    (request) => request.method() === 'GET' && new URL(request.url()).pathname === '/api/jobs'
  );
  await expect(progress).toBeVisible();

  job.finish();
  await expect(panel.getByText('Done, 3 photos', { exact: true })).toBeVisible();
  await expect(item).toContainText('completed');
  await expect(panel.getByRole('button', { name: 'Retry 1 failed' })).toHaveCount(0);
});

test.describe('jobs on touch', { tag: '@touch' }, () => {
  test('a failed job offers Retry on its row', async ({ page }) => {
    await mockLibrary(page);
    const job = await mockRetry(page);
    await page.goto('/photos');
    await expect(page.locator('html')).toHaveAttribute('data-layout', 'touch');

    const drawer = await openDrawer(page);
    await drawer.getByRole('button', { name: /^Jobs/ }).tap();
    const sheet = page.getByRole('dialog', { name: 'Jobs' });
    const retry = sheet.getByRole('button', { name: 'Retry 1 failed' });
    await expect(retry).toHaveText('Retry 1');
    expect((await retry.boundingBox())?.height ?? 0).toBeGreaterThanOrEqual(36);
    expect(await seriousViolations(page)).toEqual([]);

    await retry.tap();
    await expect(
      sheet.getByRole('progressbar', { name: 'Export to Immich progress' })
    ).toBeVisible();
    await expect(sheet.getByRole('button', { name: 'Cancel job' })).toHaveText('Cancel');
    expect(job.retried()).toBe(true);
    await sheet.getByRole('button', { name: 'Export to Immich details' }).tap();
    const item = sheet.getByRole('listitem').filter({ hasText: 'DSCF3981.RAF' });
    await expect(item).toContainText('pending');

    job.finish();
    await expect(sheet.getByText('Done, 3 photos', { exact: true })).toBeVisible();
    await expect(item).toContainText('completed');
    await expect(retry).toHaveCount(0);
  });

  test('a finished ZIP job saves its archive from the row', async ({ page }) => {
    const zip = { ...FAILED, id: 'job-z', kind: 'download_zip', completed: 3, failed: 0 };
    await mockLibrary(page);
    await page.route('**/api/jobs', (route) => route.fulfill(json([zip])));
    await page.goto('/photos');

    const drawer = await openDrawer(page);
    await drawer.getByRole('button', { name: /^Jobs/ }).tap();
    const save = page.getByRole('dialog', { name: 'Jobs' }).getByRole('link', { name: 'Save ZIP' });
    await expect(save).toHaveAttribute('href', '/api/jobs/job-z/download');
    await expect(save).toHaveAttribute('download');
  });
});
