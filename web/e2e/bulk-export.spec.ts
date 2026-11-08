import { expect, test, type Page } from '@playwright/test';
import { installMocks, json, numberedAssets, type InstallOpts } from './helpers';
import { tapCentre } from './touch';
import { open } from './touchShell';

const ASSETS = numberedAssets(2);

interface QueuedJob {
  finish: () => void;
}

async function mockJobQueue(page: Page): Promise<QueuedJob> {
  let kind = 'download_zip';
  let done = false;
  let release: () => void = () => {};
  const finished = new Promise<void>((resolve) => (release = resolve));
  const job = () => ({
    id: 'job-queued',
    kind,
    status: done ? 'completed' : 'running',
    target: {},
    params: {},
    total: 2,
    completed: done ? 2 : 0,
    failed: 0,
    cancelled_at: null,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z'
  });
  await page.route('**/api/jobs', (route) => {
    if (route.request().method() !== 'POST') return route.fulfill(json([job()]));
    kind = (route.request().postDataJSON() as { kind: string }).kind;
    return route.fulfill(json(job()));
  });
  await page.route('**/api/jobs/job-queued', (route) =>
    route.fulfill(json({ job: job(), items: [] }))
  );
  await page.route('**/api/jobs/job-queued/events', async (route) => {
    await finished;
    done = true;
    return route.fulfill({
      status: 200,
      contentType: 'text/event-stream',
      body: `event: job\ndata: ${JSON.stringify(job())}\n\n`
    });
  });
  await page.route('**/api/jobs/job-queued/download', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/zip',
      headers: { 'content-disposition': 'attachment; filename="immich-edit-export.zip"' },
      body: 'zip'
    })
  );
  return { finish: release };
}

async function expectJobsOn(page: Page, label: string): Promise<void> {
  const drawer = page.getByRole('dialog', { name: 'Jobs' });
  await expect(drawer).toBeVisible();
  await expect(drawer.getByRole('progressbar', { name: `${label} progress` })).toBeVisible();
}

async function openDesktopExport(page: Page, count = 2, opts: InstallOpts = {}) {
  await installMocks(page, { assets: ASSETS, ...opts });
  const queue = await mockJobQueue(page);
  await page.goto('/photos');
  const select = page.getByRole('button', { name: 'Select', exact: true });
  for (let picked = 0; picked < count; picked += 1) await select.nth(0).click();
  await expect(page.getByText(`${count} selected`)).toBeVisible();
  await page.getByRole('button', { name: 'Edit and export selected' }).click();
  const dialog = page.getByRole('dialog', { name: 'Edit and export selected' });
  await dialog.getByRole('tab', { name: 'Export' }).click();
  return { dialog, queue };
}

async function openTouchExport(page: Page, count = 2, opts: InstallOpts = {}) {
  await open(page, '/photos', { assets: ASSETS, ...opts });
  const queue = await mockJobQueue(page);
  await page.getByRole('button', { name: 'Select photos' }).tap();
  for (const asset of ASSETS.slice(0, count)) {
    await tapCentre(page, page.locator(`[data-name="${asset.originalFileName}"]`));
  }
  await expect(page.locator('header[data-app-bar]')).toContainText(`${count} selected`);
  await page
    .getByRole('toolbar', { name: 'Selection actions' })
    .getByRole('button', { name: 'Export' })
    .tap();
  const sheet = page.getByRole('dialog', { name: /^Export \d photos?$/ });
  await expect(sheet).toBeVisible();
  return { dialog: sheet, queue };
}

test('a ZIP export opens Jobs on the job and downloads the ZIP when it finishes', async ({
  page
}) => {
  const { dialog, queue } = await openDesktopExport(page);
  await dialog.getByRole('button', { name: 'Download 2 as JPEG ZIP' }).click();

  await expect(dialog).toBeHidden();
  await expectJobsOn(page, 'Download ZIP');
  const download = page.waitForEvent('download');
  queue.finish();
  expect((await download).url()).toMatch(/\/api\/jobs\/job-queued\/download$/);
  await expect(
    page.getByRole('dialog', { name: 'Jobs' }).getByRole('link', { name: 'Download ZIP' })
  ).toBeVisible();
});

test('an export to Immich opens Jobs on the job', async ({ page }) => {
  const { dialog } = await openDesktopExport(page);
  await dialog.getByRole('radio', { name: 'To Immich' }).click();
  await dialog.getByRole('button', { name: 'Export 2 to Immich' }).click();

  await expect(dialog).toBeHidden();
  await expectJobsOn(page, 'Export to Immich');
});

test('@touch a ZIP export closes the sheet, opens Jobs and downloads the ZIP', async ({ page }) => {
  const { dialog, queue } = await openTouchExport(page);
  await dialog.getByRole('button', { name: 'Download 2 as JPEG ZIP' }).tap();

  await expect(dialog).toBeHidden();
  await expectJobsOn(page, 'Download ZIP');
  const download = page.waitForEvent('download');
  queue.finish();
  expect((await download).url()).toMatch(/\/api\/jobs\/job-queued\/download$/);
});

test('@touch an export to Immich closes the sheet and opens Jobs', async ({ page }) => {
  const { dialog } = await openTouchExport(page);
  await dialog.getByRole('radio', { name: 'To Immich' }).tap();
  await dialog.getByRole('button', { name: 'Export 2 to Immich' }).tap();

  await expect(dialog).toBeHidden();
  await expectJobsOn(page, 'Export to Immich');
});

function recordExports(): { opts: InstallOpts; requests: URL[] } {
  const requests: URL[] = [];
  return {
    requests,
    opts: {
      onExport: (route) => {
        requests.push(new URL(route.request().url()));
        return route.fulfill({
          status: 200,
          contentType: 'image/jpeg',
          headers: { 'content-disposition': 'attachment; filename="IMG_0001_edit.jpg"' },
          body: 'jpg'
        });
      }
    }
  };
}

function expectSingleExport(requests: URL[]): void {
  expect(requests).toHaveLength(1);
  expect(requests[0]?.pathname).toBe(`/api/assets/${ASSETS[0]?.id}/export`);
  expect(requests[0]?.searchParams.get('format')).toBe('png');
}

test('a single photo downloads directly in the chosen format', async ({ page }) => {
  const exports = recordExports();
  const { dialog } = await openDesktopExport(page, 1, exports.opts);
  await dialog.getByRole('combobox', { name: 'Format' }).click();
  await page.getByRole('option', { name: 'PNG' }).click();

  const download = page.waitForEvent('download');
  await dialog.getByRole('button', { name: 'Download PNG' }).click();
  expect((await download).suggestedFilename()).toBe('IMG_0001_edit.jpg');
  expectSingleExport(exports.requests);
  await expect(dialog).toBeHidden();
  await expect(page.getByRole('dialog', { name: 'Jobs' })).toHaveCount(0);
});

test('@touch a single photo downloads directly and closes the sheet', async ({ page }) => {
  const exports = recordExports();
  const { dialog } = await openTouchExport(page, 1, exports.opts);
  await expect(dialog).toHaveAccessibleName('Export 1 photo');
  await expect(dialog.getByText(/selected/)).toHaveCount(0);
  await dialog.getByRole('combobox', { name: 'Format' }).tap();
  await page.getByRole('option', { name: 'PNG' }).tap();

  const download = page.waitForEvent('download');
  await dialog.getByRole('button', { name: 'Download PNG' }).tap();
  expect((await download).suggestedFilename()).toBe('IMG_0001_edit.jpg');
  expectSingleExport(exports.requests);
  await expect(dialog).toBeHidden();
  await expect(page.getByRole('dialog', { name: 'Jobs' })).toHaveCount(0);
});
