import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_SUMMARY, installMocks, gotoAsset, json } from './helpers';
import { editsToManifest } from '../src/lib/edits/manifest';
import { neutralEdits } from '../src/lib/types/edits';

function warmPreset(): Record<string, unknown> {
  const edits = neutralEdits();
  edits.basic.exposure_ev = 1;
  return {
    id: 'preset-warm',
    name: 'Warm',
    group_name: 'Film',
    manifest: editsToManifest(edits),
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z'
  };
}

async function openPresets(scope: Page | Locator): Promise<void> {
  const section = scope.getByRole('button', { name: 'Presets', exact: true });
  if ((await section.getAttribute('aria-expanded')) !== 'true') await section.click();
}

async function pickWarm(page: Page): Promise<void> {
  await page.getByRole('combobox', { name: 'Select a preset…' }).click();
  await page.getByRole('option', { name: 'Warm', exact: true }).click();
}

test('the preset amount scales the look it applies', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, { presets: [warmPreset()], onSave: (body) => saves.push(body) });
  await gotoAsset(page);
  await openPresets(page);
  const amount = page.getByRole('slider', { name: 'Preset amount' });
  await expect(page.getByRole('combobox', { name: 'Select a preset…' })).toBeVisible();
  await expect(amount).toHaveCount(0);
  await expect(page.getByRole('button', { name: /^Apply/ })).toHaveCount(0);
  await pickWarm(page);

  await expect(amount).toHaveValue('100');
  await amount.fill('50');
  await expect(page.getByText('50%', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Apply Warm' }).click();

  const exposure = page
    .locator('div.group', { has: page.getByRole('button', { name: 'Exposure', exact: true }) })
    .getByRole('slider');
  await expect(exposure).toHaveValue('0.5');
  await expect.poll(() => saves.map((body) => body.action)).toContain('Preset: Warm (50%)');

  await amount.dblclick();
  await expect(amount).toHaveValue('100');
});

test('the bulk preset job carries the chosen amount', async ({ page }) => {
  const jobs: Array<Record<string, unknown>> = [];
  await installMocks(page, { presets: [warmPreset()] });
  await page.route('**/api/jobs', (route) => {
    if (route.request().method() !== 'POST') return route.fallback();
    jobs.push(route.request().postDataJSON() as Record<string, unknown>);
    return route.fulfill(json({ id: 'job-1', kind: 'apply_preset', status: 'queued' }));
  });
  await page.goto('/photos');
  await page.getByRole('button', { name: 'Select', exact: true }).click();
  await page.getByRole('button', { name: 'Edit and export selected' }).click();
  const dialog = page.getByRole('dialog', { name: 'Edit and export selected' });
  await openPresets(dialog);
  await expect(dialog.getByRole('slider', { name: 'Preset amount' })).toHaveCount(0);
  await pickWarm(page);
  await dialog.getByRole('slider', { name: 'Preset amount' }).fill('150');
  await dialog.getByRole('button', { name: 'Apply Warm to 1' }).click();

  await expect.poll(() => jobs.length).toBe(1);
  expect(jobs[0]).toMatchObject({
    kind: 'apply_preset',
    asset_ids: [ASSET_SUMMARY.id],
    params: { preset_id: 'preset-warm', amount: 150 }
  });
});
