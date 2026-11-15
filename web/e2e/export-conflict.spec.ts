import { expect, test, type Page } from '@playwright/test';
import {
  ASSET_ID,
  gotoAsset,
  installMocks,
  json,
  numberedAssets,
  type EarlierExportsFound,
  type EarlierExportsQuery,
  type InstallOpts
} from './helpers';
import { openBulkExport } from './browse';
import { mockLibrary } from './touchShell';

const TWO_HOURS_AGO = new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString();

const ONE_EARLIER: EarlierExportsFound = {
  photos: 1,
  files: [{ filename: 'IMG_0001_edit.jpg', exported_at: TWO_HOURS_AGO }]
};

function earlier(
  found: EarlierExportsFound
): Pick<InstallOpts, 'earlierExports'> & { queries: EarlierExportsQuery[] } {
  const queries: EarlierExportsQuery[] = [];
  return {
    queries,
    earlierExports: (query) => {
      queries.push(query);
      return found;
    }
  };
}

async function openImmichExport(page: Page, opts: InstallOpts = {}): Promise<void> {
  await installMocks(page, opts);
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  await page.getByRole('radio', { name: 'To Immich' }).click();
}

function answerUploads(page: Page, result: Record<string, unknown>): Record<string, unknown>[] {
  const bodies: Record<string, unknown>[] = [];
  void page.route('**/api/assets/*/export/immich', (route) => {
    bodies.push(route.request().postDataJSON() as Record<string, unknown>);
    return route.fulfill(
      json({ filename: 'IMG_0001_edit.jpg', warnings: [], original_trashed: false, ...result })
    );
  });
  return bodies;
}

test('the earlier-export choice stays hidden when the photo was never exported', async ({
  page
}) => {
  const lookup = earlier({ photos: 0, files: [] });
  await openImmichExport(page, lookup);
  const bodies = answerUploads(page, { asset_id: 'new', status: 'created' });
  const panel = page.getByRole('tabpanel');

  await expect.poll(() => lookup.queries).toEqual([{ asset_ids: [ASSET_ID], presets: [] }]);
  await expect(panel.getByText('Exported to Immich before')).toHaveCount(0);
  await expect(panel.getByRole('radio', { name: 'Keep both' })).toHaveCount(0);

  await panel.getByRole('button', { name: /Upload JPEG to Immich/ }).click();
  await expect(panel.getByText('Uploaded IMG_0001_edit.jpg to Immich')).toBeVisible();
  expect(bodies).toEqual([expect.objectContaining({ on_conflict: 'keep_both' })]);
});

test('replacing the earlier export of one photo names the file and is not remembered', async ({
  page
}) => {
  await openImmichExport(page, earlier(ONE_EARLIER));
  const kept = 'Previous export kept: it has its own edits in immich-edit';
  const bodies = answerUploads(page, {
    asset_id: 'new',
    status: 'created',
    warnings: [kept]
  });
  const panel = page.getByRole('tabpanel');
  const choice = panel.getByRole('radiogroup', { name: 'Exported to Immich before' });

  await expect(panel.getByText('IMG_0001_edit.jpg · 2 h ago')).toBeVisible();
  await expect(choice.getByRole('radio')).toHaveText(['Replace it', 'Keep both']);
  await expect(choice.getByRole('radio', { name: 'Keep both' })).toBeChecked();
  await expect(panel.getByText('The new export is added next to the earlier one.')).toBeVisible();

  await choice.getByRole('radio', { name: 'Replace it' }).click();
  await expect(
    panel.getByText(
      'IMG_0001_edit.jpg moves to the Immich trash; the new export takes its albums and stack.'
    )
  ).toBeVisible();

  await panel.getByRole('button', { name: /Upload JPEG to Immich/ }).click();
  await expect(panel.getByText('Uploaded IMG_0001_edit.jpg to Immich')).toBeVisible();
  await expect(panel.getByText(kept)).toBeVisible();
  expect(bodies).toEqual([expect.objectContaining({ on_conflict: 'replace' })]);
  expect(bodies[0]).not.toHaveProperty('preset');
  await expect(choice.getByRole('radio', { name: 'Keep both' })).toBeChecked();

  await page.reload();
  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  await expect(
    page
      .getByRole('radiogroup', { name: 'Exported to Immich before' })
      .getByRole('radio', { name: 'Keep both' })
  ).toBeChecked();
});

test('replacing an earlier export is unavailable while the Immich trash is off', async ({
  page
}) => {
  await openImmichExport(page, { trash: false, ...earlier(ONE_EARLIER) });
  const panel = page.getByRole('tabpanel');
  const choice = panel.getByRole('radiogroup', { name: 'Exported to Immich before' });

  await expect(choice.getByRole('radio', { name: 'Replace it' })).toBeDisabled();
  await expect(
    panel.getByText(
      'The Immich trash is turned off, so Replace would delete the earlier export for good.'
    )
  ).toBeVisible();
});

test('one ticked Immich preset asks about its own earlier export and sends its name', async ({
  page
}) => {
  const lookup = earlier(ONE_EARLIER);
  await installMocks(page, {
    ...lookup,
    exportPresets: [
      {
        id: 'preset-family',
        name: 'Family',
        destination: 'immich',
        form: {},
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z'
      }
    ]
  });
  const bodies = answerUploads(page, { asset_id: 'new', status: 'created' });
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  await page.getByRole('button', { name: 'Export presets' }).click();
  await page
    .getByRole('list', { name: 'Export presets' })
    .getByRole('checkbox', { name: 'Export with Family' })
    .click();

  const choice = page.getByRole('radiogroup', {
    name: 'Exported to Immich before with this preset'
  });
  await choice.getByRole('radio', { name: 'Replace it' }).click();
  expect(lookup.queries).toContainEqual({ asset_ids: [ASSET_ID], presets: ['Family'] });
  await page.getByRole('button', { name: 'Export with 1 preset' }).click();

  await expect.poll(() => bodies).toHaveLength(1);
  expect(bodies[0]).toMatchObject({ preset: 'Family', on_conflict: 'replace' });
});

test('a batch counts earlier uploads, can skip them, and Jobs marks skipped and replaced', async ({
  page
}) => {
  const assets = numberedAssets(2);
  const lookup = earlier({ photos: 1, files: [] });
  await installMocks(page, { assets, ...lookup });
  const bodies: Record<string, unknown>[] = [];
  const job = {
    id: 'job-conflict',
    kind: 'export_immich',
    status: 'completed',
    target: {},
    params: {},
    total: 2,
    completed: 2,
    failed: 0,
    cancelled_at: null,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z'
  };
  const item = (index: number, result: Record<string, unknown>) => ({
    id: `item-${index}`,
    job_id: job.id,
    asset_id: assets[index]?.id,
    position: index + 1,
    variant: 0,
    status: 'completed',
    error: null,
    result: { filename: `IMG_000${index + 1}_edit.jpg`, warnings: [], ...result },
    idempotency_key: null,
    attempts: 1,
    created_at: job.created_at,
    updated_at: job.updated_at
  });
  await page.route('**/api/jobs', (route) => {
    if (route.request().method() !== 'POST') return route.fulfill(json([job]));
    bodies.push(route.request().postDataJSON() as Record<string, unknown>);
    return route.fulfill(json(job));
  });
  await page.route('**/api/jobs/job-conflict', (route) =>
    route.fulfill(
      json({
        job,
        items: [item(0, { status: 'skipped' }), item(1, { replaced_asset_id: 'old' })]
      })
    )
  );
  await page.goto('/photos');
  const select = page.getByRole('button', { name: 'Select', exact: true });
  await select.nth(0).click();
  await select.nth(0).click();
  const dialog = await openBulkExport(page);
  await expect(dialog.getByText(/exported to Immich before/)).toHaveCount(0);
  await dialog.getByRole('radio', { name: 'To Immich' }).click();

  const choice = dialog.getByRole('radiogroup', {
    name: '1 of 2 photos was exported to Immich before'
  });
  await expect(choice.getByRole('radio')).toHaveText(['Replace them', 'Skip them', 'Keep both']);
  expect(lookup.queries.at(-1)?.asset_ids).toHaveLength(2);
  await choice.getByRole('radio', { name: 'Skip them' }).click();
  await expect(dialog.getByText('Only the other photo is uploaded.')).toBeVisible();
  await dialog.getByRole('button', { name: 'Export 2 to Immich' }).click();

  const drawer = page.getByRole('complementary', { name: 'Jobs' });
  await expect(drawer).toBeVisible();
  expect(bodies).toEqual([
    expect.objectContaining({ params: expect.objectContaining({ on_conflict: 'skip' }) })
  ]);
  await expect(drawer.getByText('IMG_0001_edit.jpg')).toBeVisible();
  await expect(drawer.getByText('skipped', { exact: true })).toBeVisible();
  await expect(drawer.getByText('replaced', { exact: true })).toBeVisible();
});

for (const device of ['phone', 'tablet'] as const) {
  test(
    `the touch sheet asks about an earlier export on a ${device}`,
    {
      tag: ['@touch', `@${device}`]
    },
    async ({ page }) => {
      await mockLibrary(page, earlier(ONE_EARLIER));
      const bodies = answerUploads(page, { asset_id: 'new', status: 'created' });
      await page.goto(`/assets/${ASSET_ID}`);
      await page.getByRole('button', { name: 'Share', exact: true }).tap();
      const sheet = page.getByRole('dialog', { name: 'Export', exact: true });
      await sheet.getByRole('radio', { name: 'Immich' }).tap();
      const choice = sheet.getByRole('radiogroup', { name: 'Exported to Immich before' });
      await expect(sheet.getByText('IMG_0001_edit.jpg · 2 h ago')).toBeVisible();
      const replace = choice.getByRole('radio', { name: 'Replace it' });
      expect((await replace.boundingBox())?.height).toBeGreaterThanOrEqual(44);
      await replace.tap();
      await expect(sheet.getByText(/IMG_0001_edit\.jpg moves to the Immich trash/)).toBeVisible();
      await sheet.getByRole('button', { name: /Upload JPEG to Immich/ }).tap();
      await expect.poll(() => bodies).toHaveLength(1);
      expect(bodies[0]).toMatchObject({ on_conflict: 'replace' });
    }
  );
}
