import { readFile } from 'node:fs/promises';
import { expect, test, type Page } from '@playwright/test';
import { installMocks, gotoAsset, openPresetMenu, openPresetStrip, presetAction } from './helpers';
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

async function chooseFile(page: Page, name: string, body: string): Promise<void> {
  const chooser = page.waitForEvent('filechooser');
  await presetAction(page, 'Import presets');
  await (await chooser).setFiles({ name, mimeType: 'application/json', buffer: Buffer.from(body) });
}

test('presets export to a versioned file', async ({ page }) => {
  await installMocks(page, { presets: [warmPreset()] });
  await gotoAsset(page);
  await openPresetStrip(page);

  const all = page.waitForEvent('download');
  await (
    await openPresetMenu(page, 'Warm')
  )
    .getByRole('button', { name: 'Export all presets' })
    .click();
  const allFile = await all;
  expect(allFile.suggestedFilename()).toBe('immich-edit-presets.json');
  const body = JSON.parse(await readFile(await allFile.path(), 'utf8'));
  expect(body).toMatchObject({
    kind: 'immich-edit-presets',
    version: 1,
    presets: [{ name: 'Warm', group_name: 'Film', manifest: warmPreset().manifest }]
  });
  expect(body.presets[0]).not.toHaveProperty('id');

  const one = page.waitForEvent('download');
  await (
    await openPresetMenu(page, 'Warm')
  )
    .getByRole('button', { name: 'Export', exact: true })
    .click();
  expect((await one).suggestedFilename()).toBe('Warm.json');
});

test('importing presets skips names already in the library', async ({ page }) => {
  const created: Array<Record<string, unknown>> = [];
  await installMocks(page, {
    presets: [warmPreset()],
    onPresetCreate: (body) => created.push(body)
  });
  await gotoAsset(page);
  await openPresetStrip(page);

  const cool = { ...warmPreset(), name: 'Cool', group_name: null };
  const file = {
    kind: 'immich-edit-presets',
    version: 1,
    presets: [warmPreset(), cool].map(({ name, group_name, manifest }) => ({
      name,
      group_name,
      manifest
    }))
  };
  await chooseFile(page, 'presets.json', JSON.stringify(file));

  await expect(
    page.getByText('Imported 1 preset, skipped 1 with a name already in use')
  ).toBeVisible();
  expect(created.map((body) => body.name)).toEqual(['Cool']);
  await expect(page.getByRole('button', { name: 'Apply Cool' })).toBeVisible();
});

test('a file that is not a preset export is refused', async ({ page }) => {
  const created: Array<Record<string, unknown>> = [];
  await installMocks(page, { onPresetCreate: (body) => created.push(body) });
  await gotoAsset(page);
  await openPresetStrip(page);

  await chooseFile(page, 'other.json', '{"kind":"something-else"}');

  await expect(
    page.getByText('Failed to import presets: Not an immich-edit preset file')
  ).toBeVisible();
  expect(created).toEqual([]);
});
