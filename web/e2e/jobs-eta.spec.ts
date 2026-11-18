import { expect, test, type Locator, type Page } from '@playwright/test';
import { installMocks, json } from './helpers';
import { mockLibrary } from './touchShell';

const RUNNING_JOB = {
  id: 'job-eta',
  kind: 'export_immich',
  status: 'running',
  target: null,
  params: null,
  total: 12,
  completed: 3,
  failed: 0,
  cancelled_at: null,
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:30Z',
  eta_secs: 150,
  item_ms: 16667
};

const ETA = '3 min left';

async function openWithJob(page: Page, touch: boolean): Promise<void> {
  await (touch ? mockLibrary(page) : installMocks(page));
  await page.route('**/api/jobs/**', (route) => route.fulfill({ status: 404 }));
  await page.route('**/api/jobs', (route) => route.fulfill(json([RUNNING_JOB])));
  await page.goto('/photos');
}

function progressControl(page: Page): Locator {
  return page.getByRole('button', { name: `Jobs: Exporting 3 of 12, ${ETA}` });
}

async function expectEtaInside(control: Locator): Promise<void> {
  await expect(control).toBeVisible();
  const eta = control.getByText(ETA, { exact: true });
  await expect(eta).toBeVisible();
  const [outer, inner] = await Promise.all([control.boundingBox(), eta.boundingBox()]);
  expect(inner?.x ?? -1).toBeGreaterThanOrEqual(outer?.x ?? 0);
  expect((inner?.x ?? 0) + (inner?.width ?? 0)).toBeLessThanOrEqual(
    (outer?.x ?? 0) + (outer?.width ?? 0)
  );
}

async function expectEtaInJobs(page: Page): Promise<void> {
  const jobs = page.getByRole('dialog', { name: 'Jobs' });
  await expect(jobs).toBeVisible();
  await expect(jobs).toContainText(`3 / 12 · ${ETA}`);
}

test('the jobs drawer shows the ETA of a running job', async ({ page }) => {
  await openWithJob(page, false);
  await page.getByRole('button', { name: 'Jobs', exact: true }).click();
  await expectEtaInJobs(page);
});

test.describe('jobs ETA on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('the jobs pill and sheet show the ETA in portrait and landscape', async ({ page }) => {
    await openWithJob(page, true);
    await expectEtaInside(progressControl(page));

    const portrait = page.viewportSize();
    await page.setViewportSize({ width: portrait?.height ?? 915, height: portrait?.width ?? 412 });
    await expectEtaInside(progressControl(page));
    await progressControl(page).tap();
    await expectEtaInJobs(page);
  });
});

test.describe('jobs ETA on a tablet', { tag: ['@touch', '@tablet'] }, () => {
  test('the sidebar jobs card and sheet show the ETA', async ({ page }) => {
    await openWithJob(page, true);
    await expectEtaInside(progressControl(page));
    await progressControl(page).tap();
    await expectEtaInJobs(page);
  });
});
