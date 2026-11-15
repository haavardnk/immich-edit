import { expect, test, type Page, type Request } from '@playwright/test';
import {
  ASSET_ID,
  gotoAsset,
  installMocks,
  jobsPanel,
  numberedAssets,
  type InstallOpts
} from './helpers';
import { tapCentre } from './touch';
import { ALBUMS, mockLibrary, open } from './touchShell';

const ASSETS = numberedAssets(2);

function exportPreset(
  name: string,
  destination: 'download' | 'immich',
  form: Record<string, unknown> = {}
): Record<string, unknown> {
  return {
    id: `preset-${name.toLowerCase().replace(/\W+/g, '-')}`,
    name,
    destination,
    form,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z'
  };
}

const PRESETS = [
  exportPreset('Archive', 'download', { format: 'jxl', bitDepth: '16', colorSpace: 'displayp3' }),
  exportPreset('Family album', 'immich', {
    resizeMode: 'dimensions',
    albumIds: [ALBUMS[1]?.id]
  }),
  exportPreset('Web 2048', 'download', {
    resizeMode: 'dimensions',
    sharpenMedia: 'screen',
    filenameTemplate: '{name}_web'
  })
];

interface JobBody {
  kind: string;
  asset_ids: string[];
  params: { variants?: { preset: string; params: Record<string, unknown> }[] };
}

function recordJobs(page: Page): JobBody[] {
  const bodies: JobBody[] = [];
  page.on('request', (request: Request) => {
    if (request.method() === 'POST' && request.url().endsWith('/api/jobs')) {
      bodies.push(request.postDataJSON() as JobBody);
    }
  });
  return bodies;
}

function opts(): InstallOpts {
  return { exportPresets: structuredClone(PRESETS), albums: ALBUMS };
}

async function stubShare(page: Page): Promise<void> {
  await page.addInitScript(() => {
    navigator.canShare = () => true;
    navigator.share = async () => {};
  });
}

test('the preset menu sums up each preset and exports several from the photo', async ({ page }) => {
  await installMocks(page, opts());
  const jobs = recordJobs(page);
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  await page.getByRole('button', { name: 'Export presets' }).click();

  const list = page.getByRole('list', { name: 'Export presets' });
  await expect(list.getByRole('listitem')).toHaveCount(3);
  await expect(list.getByRole('listitem').nth(0)).toContainText(
    'JPEG XL 16-bit · Full size · Display P3'
  );
  await expect(list.getByRole('listitem').nth(1)).toContainText(
    'JPEG 90 · 2048 × 2048 · sRGB · to Immich, Family'
  );
  await expect(list.getByRole('img')).toHaveCount(0);

  await list.getByText('Family album', { exact: true }).click();
  await expect(list.getByRole('checkbox', { name: 'Export with Family album' })).toBeChecked();
  await list.getByRole('checkbox', { name: 'Export with Web 2048' }).click();
  await list.getByRole('checkbox', { name: 'Export with Archive' }).click();
  await expect(page.getByText('3 selected · runs as 2 jobs')).toBeVisible();
  await page.getByRole('button', { name: 'Clear' }).click();
  await expect(page.getByRole('button', { name: /^Export with \d presets?$/ })).toHaveCount(0);

  await list.getByRole('checkbox', { name: 'Export with Web 2048' }).click();
  await list.getByRole('checkbox', { name: 'Export with Archive' }).click();
  await expect(page.getByText('2 selected · runs as one job')).toBeVisible();
  await list.getByRole('checkbox', { name: 'Export with Family album' }).click();
  await page.getByRole('button', { name: 'Export with 3 presets' }).click();

  await expect(page.getByRole('complementary', { name: 'Jobs' })).toBeVisible();
  await expect(list).toBeHidden();
  expect(jobs.map((job) => [job.kind, job.asset_ids])).toEqual([
    ['download_zip', [ASSET_ID]],
    ['export_immich', [ASSET_ID]]
  ]);
  const [zip, upload] = jobs;
  expect(
    zip?.params.variants?.map((v) => [v.preset, v.params.format, v.params.filename_template])
  ).toEqual([
    ['Archive', 'jxl', '{name}_{preset}'],
    ['Web 2048', 'jpeg', '{name}_web']
  ]);
  expect(upload?.params.variants?.map((v) => [v.preset, v.params.album_ids])).toEqual([
    ['Family album', [ALBUMS[1]?.id]]
  ]);
});

test('a preset keeps its destination when saved and applied', async ({ page }) => {
  const saved: Record<string, unknown>[] = [];
  page.on('request', (request) => {
    if (request.method() === 'POST' && request.url().endsWith('/api/export-presets')) {
      saved.push(request.postDataJSON() as Record<string, unknown>);
    }
  });
  await installMocks(page, opts());
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Export', exact: true }).click();

  await page.getByRole('radio', { name: 'To Immich' }).click();
  await page.getByRole('button', { name: 'Export presets' }).click();
  await page.getByRole('textbox', { name: 'Export preset name' }).fill('Upload');
  await page.getByRole('button', { name: 'Save export preset' }).click();
  await expect(page.getByRole('button', { name: 'Apply Upload', exact: true })).toBeVisible();
  expect(saved.map((body) => [body.name, body.destination])).toEqual([['Upload', 'immich']]);

  await page.getByRole('button', { name: 'Apply Web 2048', exact: true }).click();
  await expect(page.getByRole('radio', { name: 'Download' })).toBeChecked();
  await expect(page.getByRole('button', { name: 'Export JPEG' })).toBeVisible();
});

test('the bulk dialog keeps presets in the menu and exports them from it', async ({ page }) => {
  await installMocks(page, { ...opts(), assets: ASSETS });
  const jobs = recordJobs(page);
  await page.goto('/photos');
  const select = page.getByRole('button', { name: 'Select', exact: true });
  await select.nth(0).click();
  await select.nth(0).click();
  await page.getByRole('button', { name: 'Edit and export selected' }).click();
  const dialog = page.getByRole('dialog', { name: 'Edit and export selected' });
  await dialog.getByRole('tab', { name: 'Export' }).click();

  await expect(page.getByRole('checkbox', { name: 'Export with Family album' })).toHaveCount(0);
  await expect(dialog.getByRole('heading', { name: 'Settings', exact: true })).toHaveCount(0);
  await expect(dialog.getByText('2 photos · ZIP', { exact: true })).toBeVisible();

  const trigger = dialog.getByRole('button', { name: 'Export presets' });
  const beside = await dialog.getByRole('button', { name: 'close', exact: true }).boundingBox();
  const box = await trigger.boundingBox();
  expect(
    Math.abs((box?.y ?? 0) + (box?.height ?? 0) / 2 - (beside?.y ?? 0) - (beside?.height ?? 0) / 2)
  ).toBeLessThan(4);
  expect((box?.x ?? 0) + (box?.width ?? 0)).toBeLessThanOrEqual(beside?.x ?? 0);
  await expect(
    dialog.getByRole('tabpanel').getByRole('button', { name: 'Download 2 as JPEG ZIP' })
  ).toHaveCount(0);
  await trigger.click();
  const list = page.getByRole('list', { name: 'Export presets' });
  await list.getByRole('checkbox', { name: 'Export with Family album' }).click();
  await list.getByRole('checkbox', { name: 'Export with Web 2048' }).click();
  await expect(page.getByText('2 selected · runs as 2 jobs')).toBeVisible();
  await expect(list.getByText(/Save only/)).toHaveCount(0);

  await trigger.click();
  await expect(list).toBeHidden();
  await expect(dialog.getByRole('radio', { name: 'Download', exact: true })).toBeChecked();
  await expect(dialog.getByText('2 photos · ZIP', { exact: true })).toBeVisible();
  await expect(dialog.getByRole('button', { name: 'Download 2 as JPEG ZIP' })).toBeEnabled();

  await trigger.click();
  await expect(list.getByRole('checkbox', { name: 'Export with Web 2048' })).toBeChecked();
  await page.getByRole('button', { name: 'Export with 2 presets' }).click();

  await expect(dialog).toBeHidden();
  await expect(page.getByRole('complementary', { name: 'Jobs' })).toBeVisible();
  expect(jobs.map((job) => [job.kind, job.asset_ids.length])).toEqual([
    ['download_zip', 2],
    ['export_immich', 2]
  ]);
  expect(jobs.map((job) => job.params.variants?.map((v) => v.preset))).toEqual([
    ['Web 2048'],
    ['Family album']
  ]);
});

test('the bulk form ignores ticked presets', async ({ page }) => {
  await installMocks(page, { ...opts(), assets: ASSETS });
  const jobs = recordJobs(page);
  await page.goto('/photos');
  const select = page.getByRole('button', { name: 'Select', exact: true });
  await select.nth(0).click();
  await select.nth(0).click();
  await page.getByRole('button', { name: 'Edit and export selected' }).click();
  const dialog = page.getByRole('dialog', { name: 'Edit and export selected' });
  await dialog.getByRole('tab', { name: 'Export' }).click();

  const trigger = dialog.getByRole('button', { name: 'Export presets' });
  await trigger.click();
  await page.getByRole('checkbox', { name: 'Export with Family album' }).click();
  await trigger.click();
  await dialog.getByRole('button', { name: 'Download 2 as JPEG ZIP' }).click();

  await expect(dialog).toBeHidden();
  expect(jobs.map((job) => [job.kind, job.params.variants])).toEqual([['download_zip', undefined]]);
});

for (const device of ['phone', 'tablet'] as const) {
  test.describe(`touch presets on a ${device}`, { tag: ['@touch', `@${device}`] }, () => {
    test('the preset sheet ticks presets, flags save-only formats and exports', async ({
      page
    }) => {
      await stubShare(page);
      await mockLibrary(page, opts());
      const jobs = recordJobs(page);
      await page.goto(`/assets/${ASSET_ID}`);
      await page.getByRole('button', { name: 'Share', exact: true }).tap();
      const sheet = page.getByRole('dialog', { name: 'Export', exact: true });
      await sheet.getByRole('button', { name: 'Export presets' }).tap();

      const presets = page.getByRole('dialog', { name: 'Export presets' });
      await expect(presets).toBeVisible();
      const archive = presets.getByRole('listitem').filter({ hasText: 'Archive' });
      await expect(archive.getByText('Save only')).toBeVisible();
      await expect(presets.getByText(/HEIC and JPEG XL can't be shared/)).toBeVisible();

      await presets.getByText('Web 2048', { exact: true }).tap();
      await expect(presets.getByRole('checkbox', { name: 'Export with Web 2048' })).toBeChecked();
      await presets.getByRole('checkbox', { name: 'Export with Archive' }).tap();
      await expect(presets.getByText('2 selected · runs as one job')).toBeVisible();
      await presets.getByRole('button', { name: 'Export with 2 presets' }).tap();
      await expect(jobsPanel(page)).toBeVisible();
      expect(jobs.map((job) => job.params.variants?.map((v) => v.preset))).toEqual([
        ['Archive', 'Web 2048']
      ]);
    });

    test('one ticked preset saves the file directly without a job', async ({ page }) => {
      await stubShare(page);
      await mockLibrary(page, opts());
      const jobs = recordJobs(page);
      await page.goto(`/assets/${ASSET_ID}`);
      await page.getByRole('button', { name: 'Share', exact: true }).tap();
      const sheet = page.getByRole('dialog', { name: 'Export', exact: true });
      await sheet.getByRole('button', { name: 'Export presets' }).tap();

      const presets = page.getByRole('dialog', { name: 'Export presets' });
      await presets.getByRole('checkbox', { name: 'Export with Web 2048' }).tap();
      await expect(presets.getByText(/runs as/)).toHaveCount(0);
      const download = page.waitForEvent('download');
      await presets.getByRole('button', { name: 'Export with 1 preset' }).tap();
      await expect(presets).toBeHidden();
      expect((await download).suggestedFilename()).toBe('IMG_0001_edit.jpg');
      await expect(sheet.getByText('Saved', { exact: true }).first()).toBeVisible();
      await expect(sheet.getByRole('group', { name: 'Exported file' })).toContainText(
        '2048 × 1365'
      );
      expect(jobs).toEqual([]);
    });

    test('the sheet folds the file sections for both destinations', async ({ page }) => {
      await mockLibrary(page, opts());
      await page.goto(`/assets/${ASSET_ID}`);
      await page.getByRole('button', { name: 'Share', exact: true }).tap();
      const sheet = page.getByRole('dialog', { name: 'Export', exact: true });
      for (const destination of ['Device', 'Immich']) {
        await sheet.getByRole('radio', { name: destination }).tap();
        await expect(sheet.getByRole('combobox', { name: 'Format' })).toHaveCount(0);
        for (const title of ['File', 'Size', 'Watermark', 'Name']) {
          await expect(sheet.getByRole('button', { name: new RegExp(`^${title}`) })).toBeVisible();
        }
      }
      await expect(sheet.getByRole('region', { name: 'Immich' })).toBeVisible();
      await sheet.getByRole('button', { name: /^File/ }).tap();
      await expect(sheet.getByRole('combobox', { name: 'Format' })).toBeVisible();
    });

    test('a batch export keeps presets in a menu sheet and says it saves a ZIP', async ({
      page
    }) => {
      await stubShare(page);
      const jobs = recordJobs(page);
      await open(page, '/photos', { ...opts(), assets: ASSETS });
      await page.getByRole('button', { name: 'Select photos' }).tap();
      for (const asset of ASSETS) {
        await tapCentre(page, page.locator(`[data-name="${asset.originalFileName}"]`));
      }
      await page
        .getByRole('toolbar', { name: 'Selection actions' })
        .getByRole('button', { name: 'Export' })
        .tap();
      const sheet = page.getByRole('dialog', { name: 'Export 2 photos' });
      await expect(
        sheet.getByText('Saves one ZIP instead of sharing.', { exact: false })
      ).toBeVisible();
      await expect(sheet.getByRole('combobox', { name: 'Format' })).toHaveCount(0);
      await expect(page.getByRole('checkbox', { name: 'Export with Family album' })).toHaveCount(0);

      const trigger = sheet.getByRole('button', { name: 'Export presets' });
      expect((await trigger.boundingBox())?.height ?? 0).toBeGreaterThanOrEqual(44);
      await trigger.tap();
      const presets = page.getByRole('dialog', { name: 'Export presets' });
      await expect(presets).toBeVisible();
      await expect(presets.getByText('Save only')).toHaveCount(0);
      await presets.getByRole('checkbox', { name: 'Export with Family album' }).tap();
      await presets.getByRole('button', { name: 'Export with 1 preset' }).tap();

      await expect(presets).toBeHidden();
      await expect(jobsPanel(page)).toBeVisible();
      expect(jobs.map((job) => [job.kind, job.params.variants?.map((v) => v.preset)])).toEqual([
        ['export_immich', ['Family album']]
      ]);
    });
  });
}
