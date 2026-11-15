import { expect, test, type Page } from '@playwright/test';
import { installMocks, jobsPanel, json, numberedAssets, type InstallOpts } from './helpers';
import { openBulkExport } from './browse';
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
  const drawer = jobsPanel(page);
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
  const dialog = await openBulkExport(page);
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
  await expect(jobsPanel(page).getByRole('link', { name: 'Download ZIP' })).toBeVisible();
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
  await dialog.getByRole('radio', { name: 'Immich', exact: true }).tap();
  await dialog.getByRole('button', { name: 'Export 2 to Immich' }).tap();

  await expect(dialog).toBeHidden();
  await expectJobsOn(page, 'Export to Immich');
});

test('@touch the sheet opens at full height with presets beside the close button', async ({
  page
}) => {
  const { dialog } = await openTouchExport(page);
  const viewport = page.viewportSize()?.height ?? 0;
  await expect(dialog.getByRole('radio', { name: 'Device' })).toBeChecked();
  await expect
    .poll(async () => (await dialog.boundingBox())?.height ?? 0)
    .toBeGreaterThan(viewport * 0.8);
  const presets = dialog.getByRole('button', { name: 'Export presets' });
  const close = dialog.getByRole('button', { name: 'Close export' });
  await expect
    .poll(async () => {
      const [a, b] = await Promise.all([presets.boundingBox(), close.boundingBox()]);
      return Math.abs((a?.y ?? 0) + (a?.height ?? 0) / 2 - (b?.y ?? 0) - (b?.height ?? 0) / 2);
    })
    .toBeLessThan(4);
  const full = (await dialog.boundingBox())?.height;

  await dialog.getByRole('region', { name: 'File' }).getByRole('button').first().tap();
  await expect(dialog.getByRole('combobox', { name: 'Format' })).toBeVisible();
  expect((await dialog.boundingBox())?.height).toBe(full);

  await dialog.getByRole('button', { name: 'Close export' }).tap();
  await expect(dialog).toBeHidden();
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
  await expect(jobsPanel(page)).toHaveCount(0);
});

test('@touch a single photo downloads directly and closes the sheet', async ({ page }) => {
  const exports = recordExports();
  const { dialog } = await openTouchExport(page, 1, exports.opts);
  await expect(dialog).toHaveAccessibleName('Export 1 photo');
  await expect(dialog.getByText(/selected/)).toHaveCount(0);
  await expect(dialog.getByRole('combobox', { name: 'Format' })).toHaveCount(0);
  await dialog.getByRole('button', { name: /^File/ }).tap();
  await dialog.getByRole('combobox', { name: 'Format' }).tap();
  await page.getByRole('option', { name: 'PNG' }).tap();

  const download = page.waitForEvent('download');
  await dialog.getByRole('button', { name: 'Download PNG' }).tap();
  expect((await download).suggestedFilename()).toBe('IMG_0001_edit.jpg');
  expectSingleExport(exports.requests);
  await expect(dialog).toBeHidden();
  await expect(jobsPanel(page)).toHaveCount(0);
});
