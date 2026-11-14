import { expect, test, type Page } from '@playwright/test';
import { seriousViolations } from './axeHelpers';
import { json, numberedAssets } from './helpers';
import { avatar, mockLibrary, open, openDrawer, tabBar } from './touchShell';

const GREEN = { id: 'tag-green', name: 'green', value: 'immich-edit/label/green' };

function job(id: string, kind: string, status: string, failed = 0) {
  return {
    id,
    kind,
    status,
    target: null,
    params: null,
    total: 12,
    completed: status === 'running' ? 3 : 11,
    failed,
    cancelled_at: null,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z'
  };
}

const FINISHED = job('job-paste', 'paste_edits', 'completed', 1);
const RUNNING = job('job-export', 'export_immich', 'running');

async function stubJobs(page: Page): Promise<void> {
  await page.route('**/api/jobs/**', (route) => {
    const url = new URL(route.request().url());
    if (url.pathname !== `/api/jobs/${FINISHED.id}`) return route.fulfill({ status: 404 });
    return route.fulfill(
      json({
        job: FINISHED,
        items: [
          {
            id: 'item-1',
            job_id: FINISHED.id,
            asset_id: 'asset-0001',
            status: 'failed',
            error: 'Unsupported raw format',
            result: { filename: 'IMG_3990.CR3' },
            idempotency_key: null,
            attempts: 1,
            created_at: '2026-01-01T00:00:00Z',
            updated_at: '2026-01-01T00:00:00Z'
          }
        ]
      })
    );
  });
  await page.route('**/api/jobs', (route) => route.fulfill(json([FINISHED, RUNNING])));
}

async function openWithJobs(page: Page): Promise<void> {
  await mockLibrary(page);
  await stubJobs(page);
  await page.goto('/photos');
  await expect(page.locator('html')).toHaveAttribute('data-layout', 'touch');
}

async function expectLargeDetent(page: Page, name: string): Promise<void> {
  const sheet = page.getByRole('dialog', { name });
  await expect(sheet).toBeVisible();
  const height = page.viewportSize()?.height ?? 0;
  await expect
    .poll(async () => (await sheet.boundingBox())?.height ?? 0)
    .toBeGreaterThan(height * 0.8);
}

test.describe('touch sheets on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('filters open in a sheet that applies live, resets and shows the count', async ({
    page
  }) => {
    const assets = numberedAssets(3);
    const [first] = assets;
    if (first) Object.assign(first, { tags: [GREEN] });
    await open(page, '/photos', { assets, tags: [GREEN] });
    const count = page.locator('header[data-app-bar]').getByText(/^\d+ photos?$/);
    await expect(count).toHaveText('3 photos');

    const toggle = page.getByRole('button', { name: /^View & filters/ });
    await toggle.tap();
    await expectLargeDetent(page, 'View & filters');
    const sheet = page.getByRole('dialog', { name: 'View & filters' });
    await expect(page.getByRole('complementary', { name: /filters/i })).toHaveCount(0);
    await expect(sheet.getByRole('button', { name: 'Show 3 photos' })).toBeVisible();
    await expect(sheet.getByRole('button', { name: 'Reset all filters' })).toBeDisabled();

    await sheet.getByRole('button', { name: 'Rating & label', exact: true }).tap();
    await sheet.getByRole('button', { name: 'Green label' }).tap();
    await expect(sheet.getByRole('button', { name: 'Show 1 photo' })).toBeVisible();
    await expect(count).toHaveText('1 photo');
    expect(await seriousViolations(page)).toEqual([]);

    await sheet.getByRole('button', { name: 'Reset all filters' }).tap();
    await expect(sheet.getByRole('button', { name: 'Show 3 photos' })).toBeVisible();
    await expect(count).toHaveText('3 photos');

    await sheet.getByRole('button', { name: 'Red label' }).tap();
    const none = sheet.getByRole('button', { name: 'Show 0 photos', exact: true });
    await expect(none).toBeEnabled();
    await none.tap();
    await expect(sheet).toBeHidden();
    await expect(page.getByRole('button', { name: 'View & filters (1 active)' })).toBeFocused();

    await page.getByRole('button', { name: 'View & filters (1 active)' }).tap();
    await expect(
      page
        .getByRole('dialog', { name: 'View & filters' })
        .getByRole('button', { name: 'Rating & label', exact: true })
    ).toHaveAttribute('aria-expanded', 'true');
    await page
      .getByRole('dialog', { name: 'View & filters' })
      .getByRole('button', { name: 'Close view & filters' })
      .tap();
    await expect(page.getByRole('dialog', { name: 'View & filters' })).toBeHidden();
  });

  test('the jobs pill opens a sheet with running jobs first', async ({ page }) => {
    await openWithJobs(page);

    const pill = page.getByRole('button', { name: /^Jobs: Exporting/ });
    await expect(pill).toBeVisible();
    const [pillBox, navBox] = await Promise.all([pill.boundingBox(), tabBar(page).boundingBox()]);
    expect((pillBox?.y ?? 0) + (pillBox?.height ?? 0)).toBeLessThanOrEqual(navBox?.y ?? 0);
    await pill.tap();
    await expectLargeDetent(page, 'Jobs');
    const sheet = page.getByRole('dialog', { name: 'Jobs' });
    await expect(sheet.getByText('1 active', { exact: true })).toBeVisible();
    await expect(sheet.getByRole('heading', { level: 3 })).toHaveText(['In progress', 'Finished']);
    await expect(sheet.getByRole('progressbar')).toHaveCount(2);
    await expect(sheet.getByRole('progressbar').first()).toHaveAccessibleName(
      'Export to Immich progress'
    );
    await expect(sheet.getByRole('button', { name: 'Clear finished' })).toBeVisible();

    await sheet.getByRole('button', { name: 'Toggle details' }).nth(1).tap();
    await expect(sheet.getByText('IMG_3990.CR3')).toBeVisible();
    await expect(sheet.getByText('Unsupported raw format')).toBeVisible();
    expect(await seriousViolations(page)).toEqual([]);

    await sheet.getByRole('button', { name: 'Close jobs' }).tap();
    await expect(sheet).toBeHidden();
    await expect(pill).toBeFocused();
  });

  test('the drawer Jobs row opens the jobs sheet and returns focus to the avatar', async ({
    page
  }) => {
    await openWithJobs(page);

    const drawer = await openDrawer(page);
    await drawer.getByRole('button', { name: /^Jobs/ }).tap();
    await expect(drawer).toBeHidden();
    await expectLargeDetent(page, 'Jobs');
    const sheet = page.getByRole('dialog', { name: 'Jobs' });
    await expect(sheet.getByRole('heading', { level: 3 })).toHaveText(['In progress', 'Finished']);

    await page.keyboard.press('Escape');
    await expect(sheet).toBeHidden();
    await expect(avatar(page)).toBeFocused();
  });
});
