import { expect, test } from '@playwright/test';
import {
  ASSET_ID,
  ASSET_SUMMARY,
  NEUTRAL_RECORD,
  installMocks,
  json,
  gotoAsset,
  numberedAssets,
  openPresetMenu,
  openPresetStrip,
  presetAction
} from './helpers';
import { editsToManifest } from '../src/lib/edits/manifest';
import { neutralEdits } from '../src/lib/types/edits';

test('editor toolbar shows the filename without a duplicate extension', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const toolbar = page.getByRole('navigation', { name: 'Editor toolbar' });
  await expect(toolbar.getByText('IMG_0001.ARW', { exact: true })).toBeVisible();
  const toolbarText = await toolbar.innerText();
  expect(toolbarText.match(/\bARW\b/g)).toHaveLength(1);
});

test('history popover expands details and restores a prior entry', async ({ page }) => {
  const latestEdits = neutralEdits();
  latestEdits.basic.exposure_ev = 1.25;
  latestEdits.geometry.rotate = 90;
  const entries = [
    {
      id: 2,
      manifest_hash: 'hash-neutral',
      deleted: false,
      manifest: editsToManifest(latestEdits),
      created_at: '2024-01-02T00:00:00Z',
      action: 'Latest'
    },
    {
      id: 1,
      manifest_hash: 'hash-prior',
      deleted: false,
      manifest: editsToManifest(neutralEdits()),
      created_at: '2024-01-01T00:00:00Z',
      action: 'Initial'
    }
  ];

  let restoreCalled = false;
  await installMocks(page, {
    onHistory: (route) => route.fulfill(json(entries)),
    onRestore: async (route) => {
      restoreCalled = true;
      const body = JSON.parse(route.request().postData() ?? '{}');
      expect(body.entry_id).toBe(1);
      await route.fulfill(json({ ...NEUTRAL_RECORD, hash: 'hash-restored' }));
    }
  });
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Edit history' }).click();
  await page.getByRole('button', { name: 'Expand changes for Latest' }).click();
  await expect(page.getByText('0.00 → +1.25')).toBeVisible();
  await expect(page.getByText('Rotation: 0° → 90°')).toBeVisible();
  expect(restoreCalled).toBe(false);
  await expect(page.getByText('Initial')).toBeVisible();
  await page.getByText('Initial').click();

  await expect.poll(() => restoreCalled).toBe(true);
});

test('export download triggers a file download', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('tab', { name: 'Export', exact: true }).click();

  const downloadPromise = page.waitForEvent('download');
  await page.getByRole('button', { name: /Export JPEG/ }).click();
  const download = await downloadPromise;
  expect(download.suggestedFilename()).toMatch(/IMG_0001.*\.jpg$/);
  await expect(page.getByText('Saved IMG_0001_edit.jpg')).toBeVisible();
});

test('export quality uses the shared range control', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  const quality = page.getByRole('slider', { name: 'Quality' });
  await quality.fill('82');

  await expect(quality).toHaveValue('82');
  await expect(page.getByText('82', { exact: true })).toBeVisible();
});

test('split view toggle reveals before/after slider', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const splitButton = page.getByRole('button', { name: /^Before \/ after split/ });
  await splitButton.click();

  await expect(page.getByRole('slider', { name: 'Before/after split' })).toBeVisible();
});

test('editor and loupe keep independent filmstrip visibility', async ({ page }) => {
  await installMocks(page);
  await page.goto('/search?q=IMG');
  await expect(page.locator(`a[href^="/assets/${ASSET_ID}?"]`)).toBeVisible();
  await page.getByLabel('Quick review').first().click();
  await expect(page.getByRole('button', { name: 'Collapse filmstrip' })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
  await page.getByRole('button', { name: /^Edit/ }).click();

  await expect(page.getByRole('button', { name: 'Hide filmstrip' })).toBeVisible();
  await page.keyboard.press('Shift+S');
  await expect(page.getByRole('button', { name: 'Show filmstrip' })).toBeVisible();
  await page.getByRole('button', { name: /^Back/ }).click();

  await expect(page.getByRole('button', { name: 'Collapse filmstrip' })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
  await expect(page.getByRole('button', { name: 'IMG_0001.ARW' })).toBeVisible();
});

test('the filmstrip keeps a manual scroll away from the current photo', async ({ page }) => {
  const assets = numberedAssets(80);
  await installMocks(page, { assets });

  await page.goto(`/assets/${ASSET_ID}?from=%2Fphotos`);
  const strip = page.getByTestId('filmstrip-scroll');
  await expect(strip).toBeVisible();
  await expect
    .poll(() => strip.evaluate((el) => el.scrollWidth - el.clientWidth))
    .toBeGreaterThan(1000);

  await strip.evaluate((el) => {
    el.scrollLeft = 600;
    el.dispatchEvent(new Event('scroll'));
  });
  const settled = await strip.evaluate(async (el) => {
    await new Promise((resolve) => setTimeout(resolve, 600));
    return el.scrollLeft;
  });

  expect(settled).toBe(600);
});

test('editor and loupe layout choices survive reload', async ({ page }) => {
  await installMocks(page);
  await page.goto('/search?q=IMG');
  await page.getByLabel('Quick review').first().click();

  await page.getByRole('button', { name: 'Collapse filmstrip' }).click();
  await page.reload();
  await page.getByLabel('Quick review').first().click();
  await expect(page.getByRole('button', { name: 'Show filmstrip' })).toBeVisible();

  await page.getByRole('button', { name: /^Edit/ }).click();
  await page.getByRole('tab', { name: 'Develop' }).click();
  await page.getByRole('button', { name: 'Hide filmstrip' }).click();
  await page.reload();

  await expect(page.getByRole('complementary', { name: 'Editor controls' })).toBeHidden();
  await page.getByRole('button', { name: /^Back/ }).click();
  await page.locator(`a[href^="/assets/${ASSET_ID}?"]`).evaluate((element) => element.click());
  await expect(page.getByRole('button', { name: 'Show filmstrip' })).toBeVisible();
});

test('a popover closes on escape and on an outside click, returning focus', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const zoom = page.getByRole('button', { name: 'Zoom', exact: true });
  const zoomIn = page.getByRole('button', { name: 'Zoom In' });

  await zoom.click();
  await expect(zoomIn).toBeVisible();

  await page.keyboard.press('Escape');
  await expect(zoomIn).toBeHidden();
  await expect(zoom).toBeFocused();

  await zoom.click();
  await expect(zoomIn).toBeVisible();

  await page.getByRole('button', { name: /^Before \/ after split/ }).click();
  await expect(zoomIn).toBeHidden();

  await zoom.click();
  await expect(zoomIn).toBeVisible();
  const split = page.getByRole('button', { name: /^Before \/ after split/ });
  await split.hover();
  await expect(page.locator('.tooltip-content', { hasText: 'Before / after split' })).toBeVisible();
  await split.click();
  await expect(zoomIn).toBeHidden();
});

test('the copy dialog is a named modal that cancels back to the editor', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const slider = page
    .locator('div.group', { has: page.getByRole('button', { name: 'Exposure', exact: true }) })
    .getByRole('slider');
  await slider.fill('1');

  const copy = page.getByRole('button', { name: 'Copy edits' });
  await expect(copy).toBeEnabled();
  await copy.click();

  const dialog = page.getByRole('dialog', { name: 'Copy settings' });
  await expect(dialog).toBeVisible();

  await dialog.getByRole('button', { name: 'Cancel' }).click();
  await expect(dialog).toBeHidden();
});

test('narrow viewports show the desktop-required guard', async ({ page }) => {
  await page.setViewportSize({ width: 600, height: 800 });
  await installMocks(page);

  await page.goto(`/assets/${ASSET_ID}`);

  await expect(page.getByRole('heading', { name: 'Desktop required' })).toBeVisible();
  await expect(page.getByRole('button', { name: /^Back/ })).toHaveCount(0);
});

test('deleting a preset requires explicit confirmation', async ({ page }) => {
  const deleted: string[] = [];
  await installMocks(page, {
    presets: [
      {
        id: 'preset-1',
        name: 'Warm film',
        group_name: null,
        manifest: { schema_version: 1, ops: {} },
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z'
      }
    ],
    onPresetDelete: (id) => deleted.push(id)
  });
  await gotoAsset(page);

  await openPresetStrip(page);
  const actions = page.getByRole('button', { name: 'Actions for Warm film' });
  await openPresetMenu(page, 'Warm film');
  await actions.click();
  await page.mouse.move(0, 0);
  await expect(actions).toHaveCSS('opacity', '0');

  const menu = await openPresetMenu(page, 'Warm film');

  await menu.getByRole('button', { name: 'Delete', exact: true }).click();
  const confirm = menu.getByRole('button', { name: 'Confirm delete' });
  await expect(confirm).toBeVisible();
  expect(deleted).toEqual([]);

  await menu.getByRole('button', { name: 'Cancel' }).click();
  await expect(confirm).toHaveCount(0);
  expect(deleted).toEqual([]);

  await menu.getByRole('button', { name: 'Delete', exact: true }).click();
  await confirm.click();
  await expect.poll(() => deleted).toEqual(['preset-1']);
  await expect(page.getByRole('button', { name: 'Apply Warm film' })).toHaveCount(0);
});

test('editing a preset from its tile menu renames and regroups it', async ({ page }) => {
  const updates: Array<[string, Record<string, unknown>]> = [];
  const manifest = { schema_version: 1, ops: {} };
  await installMocks(page, {
    presets: [
      {
        id: 'preset-1',
        name: 'Warm film',
        group_name: null,
        manifest,
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z'
      }
    ],
    onPresetUpdate: (id, body) => updates.push([id, body])
  });
  await gotoAsset(page);

  const strip = await openPresetStrip(page);
  await strip.getByRole('button', { name: 'Apply Warm film' }).click({ button: 'right' });
  await page.locator('[data-popover-content]').getByRole('button', { name: 'Edit…' }).click();
  const dialog = page.getByRole('dialog', { name: 'Edit preset' });
  const name = dialog.getByRole('textbox', { name: 'Preset name' });
  await expect(name).toHaveValue('Warm film');
  await name.fill('Cool film');
  const group = dialog.getByRole('combobox', { name: 'Preset group' });
  await group.click();
  await group.fill('Film');
  await group.press('Enter');
  await expect(group).toHaveValue('Film');
  await dialog.getByRole('button', { name: 'Save', exact: true }).click();

  await expect(dialog).toBeHidden();
  expect(updates).toEqual([['preset-1', { name: 'Cool film', group_name: 'Film', manifest }]]);
  await expect(strip.getByRole('button', { name: 'Apply Cool film' })).toHaveCount(1);
});

test('the preset strip shares the filmstrip height and filters by group', async ({ page }) => {
  const preset = (id: string, name: string, group_name: string | null) => ({
    id,
    name,
    group_name,
    manifest: { schema_version: 1, ops: {} },
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z'
  });
  await installMocks(page, {
    presets: [preset('1', 'Loose', null), preset('2', 'Warm film', 'Film')]
  });
  await gotoAsset(page);
  const filmstrip = page.getByRole('slider', { name: 'Resize filmstrip' }).locator('..');
  const height = async (panel: typeof filmstrip): Promise<number | undefined> =>
    (await panel.boundingBox())?.height;
  const filmHeight = await height(filmstrip);

  const strip = await openPresetStrip(page);
  expect(await height(strip)).toBe(filmHeight);
  const resize = strip.getByRole('slider', { name: 'Resize presets' });
  await resize.focus();
  await resize.press('Home');
  const shortHeight = await height(strip);
  expect(shortHeight).toBeLessThan(filmHeight ?? 0);
  await page.getByRole('button', { name: 'Show filmstrip' }).click();
  expect(await height(filmstrip)).toBe(shortHeight);

  await openPresetStrip(page);
  await resize.focus();
  await resize.press('End');
  const tallHeight = await height(strip);
  expect(tallHeight).toBeGreaterThan(filmHeight ?? 0);
  await page.getByRole('button', { name: 'Show filmstrip' }).click();
  expect(await height(filmstrip)).toBe(tallHeight);

  await openPresetStrip(page);
  const groups = strip.getByRole('navigation', { name: 'Preset groups' });
  await expect(groups.getByRole('button')).toHaveText(['All', 'Film', 'Ungrouped']);
  await groups.getByRole('button', { name: 'Film' }).click();
  await expect(groups.getByRole('button', { name: 'Film' })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
  await expect(strip.getByRole('button', { name: /^Apply / })).toHaveText(['Warm film']);
});

test('a preset saved from one asset applies to another', async ({ page }) => {
  const second = {
    ...ASSET_SUMMARY,
    id: '00000000-0000-0000-0000-000000000002',
    originalFileName: 'IMG_0002.ARW'
  };
  const created: Array<Record<string, unknown>> = [];
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, {
    assets: [ASSET_SUMMARY, second],
    onPresetCreate: (body) => created.push(body),
    onSave: (body) => saves.push(body)
  });
  await gotoAsset(page);

  const exposure = () =>
    page
      .locator('div.group', { has: page.getByRole('button', { name: 'Exposure', exact: true }) })
      .getByRole('slider');
  await exposure().fill('1');

  await openPresetStrip(page);
  await presetAction(page, 'Save current as preset');
  const dialog = page.getByRole('dialog', { name: 'Save preset' });
  await expect(dialog.getByRole('textbox', { name: 'Preset name' })).toBeFocused();
  await page.keyboard.type('Warm');
  await dialog.getByRole('button', { name: 'Save', exact: true }).click();

  await expect.poll(() => created.map((body) => body.name)).toEqual(['Warm']);
  await expect(dialog).toBeHidden();
  const manifest = created[0]?.manifest as { ops: Record<string, unknown> };
  expect(Object.keys(manifest.ops).length).toBeGreaterThan(0);

  await page.goto(`/assets/${second.id}`);
  await expect(exposure()).toHaveValue('0');
  await openPresetStrip(page);
  await page.getByRole('button', { name: 'Apply Warm' }).click();

  await expect(exposure()).toHaveValue('1');
  await expect.poll(() => saves.some((body) => body.action === 'Preset: Warm')).toBe(true);
});

for (const { title, group, pick } of [
  { title: 'an existing group', group: 'Film', pick: 'option' },
  { title: 'a new group', group: 'Portrait', pick: 'enter' },
  { title: 'a typed group without pressing Enter', group: 'Portrait', pick: 'none' }
]) {
  test(`saving a preset into ${title}`, async ({ page }) => {
    const created: Array<Record<string, unknown>> = [];
    await installMocks(page, {
      presets: [
        {
          id: 'preset-film',
          name: 'Kodak',
          group_name: 'Film',
          manifest: editsToManifest(neutralEdits()),
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z'
        }
      ],
      onPresetCreate: (body) => created.push(body)
    });
    await gotoAsset(page);
    await page.getByRole('slider', { name: 'Exposure' }).fill('1');

    await openPresetStrip(page);
    await presetAction(page, 'Save current as preset');
    const dialog = page.getByRole('dialog', { name: 'Save preset' });
    await dialog.getByRole('textbox', { name: 'Preset name' }).fill('Warm');
    const groupInput = dialog.getByRole('combobox', { name: 'Preset group' });
    await groupInput.click();
    await groupInput.fill(group);
    if (pick === 'option') await page.getByRole('option', { name: group }).click();
    else if (pick === 'enter') await groupInput.press('Enter');
    else await dialog.getByRole('textbox', { name: 'Preset name' }).click();
    await expect(groupInput).toHaveValue(group);
    await dialog.getByRole('button', { name: 'Save', exact: true }).click();

    await expect.poll(() => created.map((body) => body.group_name)).toEqual([group]);
  });
}

test('the preset strip explains a disabled save and keeps failed thumbnails visible', async ({
  page
}) => {
  await installMocks(page, {
    presets: [
      {
        id: 'preset-test',
        name: 'test',
        group_name: null,
        manifest: editsToManifest(neutralEdits()),
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z'
      }
    ]
  });
  await page.route('**/api/assets/*/presets/*/thumb*', (route) => route.fulfill({ status: 500 }));
  await gotoAsset(page);
  await page.getByRole('button', { name: 'Reset develop settings' }).click();
  const strip = await openPresetStrip(page);

  const save = strip.getByRole('button', { name: 'Save current as preset' });
  await expect(save).toBeDisabled();
  await expect(save).toContainText('No edits yet');
  const tile = strip.getByRole('button', { name: 'Apply test' });
  await expect(tile).toContainText('T');
  expect((await tile.boundingBox())?.height).toBeGreaterThan(60);
});

test('history restore waits for a save that is still in flight', async ({ page }) => {
  const entries = [
    {
      id: 1,
      manifest_hash: 'hash-prior',
      deleted: false,
      manifest: editsToManifest(neutralEdits()),
      created_at: '2024-01-01T00:00:00Z',
      action: 'Initial'
    }
  ];
  let releaseSave: (() => void) | null = null;
  let held = false;
  let restoreCalled = false;
  await installMocks(page, {
    onHistory: (route) => route.fulfill(json(entries)),
    onRestore: async (route) => {
      restoreCalled = true;
      await route.fulfill(json({ ...NEUTRAL_RECORD, hash: 'hash-restored' }));
    }
  });
  await page.route('**/api/assets/*/edits', async (route) => {
    if (route.request().method() !== 'PUT' || held) return route.fallback();
    held = true;
    await new Promise<void>((resolve) => {
      releaseSave = resolve;
    });
    return route.fallback();
  });
  await gotoAsset(page);

  const exposure = page
    .locator('div.group', { has: page.getByRole('button', { name: 'Exposure', exact: true }) })
    .getByRole('slider');
  await exposure.fill('1');
  await expect.poll(() => releaseSave !== null).toBe(true);

  await page.getByRole('button', { name: 'Edit history' }).click();
  await page.getByText('Initial').click();
  expect(restoreCalled).toBe(false);

  releaseSave?.();
  await expect.poll(() => restoreCalled).toBe(true);
  await expect(exposure).toHaveValue('0');
});

test('stack primary is a keyboard radio group', async ({ page }) => {
  await installMocks(page, {});
  await gotoAsset(page);

  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  await page.getByRole('radio', { name: 'To Immich' }).click();
  await page.getByRole('checkbox', { name: 'Stack with original' }).click();

  const original = page.getByRole('radio', { name: 'Original primary' });
  await expect(page.getByRole('radiogroup', { name: 'Stack primary' })).toHaveCSS('height', '28px');
  await expect(page.getByRole('radio', { name: 'Edit primary' })).toBeChecked();
  await original.click();
  await expect(original).toBeChecked();
});

test("another user's photo is read-only and says why", async ({ page }) => {
  await installMocks(page, { assets: [{ ...ASSET_SUMMARY, ownerId: 'partner' }] });
  await gotoAsset(page);

  const status = page.getByRole('navigation', { name: 'Editor status and view controls' });
  const star = status.getByRole('radio', { name: '3 stars', exact: true });
  await expect(star).toBeDisabled();
  await expect(status.getByRole('button', { name: /^Favorite/ })).toBeDisabled();
  await star.hover({ force: true });
  await expect(page.getByText('Shared by another Immich user.')).toBeVisible();

  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  await page.getByRole('radio', { name: 'To Immich' }).click();

  const stack = page.getByRole('checkbox', { name: 'Stack with original' });
  await expect(stack).toBeDisabled();
  await expect(stack).not.toBeChecked();
  await expect(page.getByRole('checkbox', { name: 'Mark as favorite' })).toBeEnabled();
  await stack.hover({ force: true });
  await expect(page.getByText('Only the owner can stack it.')).toBeVisible();
});

test('editor toggles report their pressed state', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const split = page.getByRole('button', { name: /^Before \/ after split/ });
  await expect(split).toHaveAttribute('aria-pressed', 'false');
  await split.click();
  await expect(split).toHaveAttribute('aria-pressed', 'true');
});

test('a failed preview render offers a retry that recovers', async ({ page }) => {
  await installMocks(page);
  let previewCalls = 0;
  await page.route('**/api/assets/*/preview', async (route) => {
    if (route.request().method() !== 'POST') return route.fallback();
    previewCalls += 1;
    if (previewCalls > 1) return route.fallback();
    await route.fulfill({
      status: 408,
      contentType: 'application/json',
      body: JSON.stringify({ code: 'timeout', message: 'render timed out' })
    });
  });
  await gotoAsset(page);

  const retry = page.getByRole('button', { name: 'Retry' });
  await expect(retry).toBeVisible();
  await retry.click();

  await expect(retry).toBeHidden();
  await expect(page.getByRole('img', { name: ASSET_SUMMARY.originalFileName })).toBeVisible();
});

test('develop reset keeps masks, retouch and the crop', async ({ page }) => {
  const edits = neutralEdits();
  edits.basic.exposure_ev = 1;
  edits.geometry.rotate = 90;
  edits.masks = [
    {
      id: 'layer',
      name: 'Sky',
      enabled: true,
      amount: 1,
      invert: false,
      components: [
        {
          id: 'luma',
          enabled: true,
          mode: 'add',
          invert: false,
          kind: { kind: 'luma_range', min: 0.25, max: 0.75, softness: 0.1 },
          source: 'manual'
        }
      ],
      edits: { exposure_ev: 0.5 }
    }
  ];
  edits.retouch = [
    {
      id: 'spot',
      mode: 'heal',
      points: [{ x: 0.4, y: 0.5 }],
      radius: 0.05,
      hardness: 0.5,
      opacity: 1,
      source: { x: 0.6, y: 0.5 },
      enabled: true
    }
  ];
  await installMocks(page, {
    editRecord: { ...NEUTRAL_RECORD, manifest: editsToManifest(edits), hash: 'hash-edited' }
  });
  await gotoAsset(page);

  const saved = page.waitForRequest(
    (request) => request.url().endsWith('/edits') && request.method() === 'PUT'
  );
  await page.getByRole('button', { name: 'Reset develop settings' }).click();
  const body = (await saved).postDataJSON() as {
    manifest: { ops: Record<string, { layers?: unknown[]; strokes?: unknown[] } | undefined> };
    action: string | null;
  };
  expect(body.action).toBe('Reset Develop');
  expect(body.manifest.ops.exposure).toBeUndefined();
  expect(body.manifest.ops.masks?.layers).toHaveLength(1);
  expect(body.manifest.ops.retouch?.strokes).toHaveLength(1);
  expect(body.manifest.ops.transform).toBeDefined();
  await expect(page.getByRole('button', { name: 'Reset develop settings' })).toBeDisabled();
});

test('geometry action buttons rotate, flip and reset', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Geometry' }).click();
  const reset = page.getByRole('button', { name: 'Reset geometry', exact: true });
  await expect(reset).toBeDisabled();

  const steps: [string, string][] = [
    ['Rotate right 90°', 'Rotate'],
    ['Flip horizontal', 'Flip'],
    ['Reset geometry', 'Reset Geometry']
  ];
  for (const [name, action] of steps) {
    const saved = page.waitForRequest(
      (request) => request.url().endsWith('/edits') && request.method() === 'PUT'
    );
    await page.getByRole('button', { name, exact: true }).click();
    expect(((await saved).postDataJSON() as { action: string | null }).action).toBe(action);
    if (action === 'Rotate') await expect(reset).toBeEnabled();
  }
  await expect(reset).toBeDisabled();
});

test('the reset shortcut on the Geometry tab resets only geometry', async ({ page }) => {
  const edits = neutralEdits();
  edits.basic.exposure_ev = 1;
  edits.geometry.rotate = 90;
  edits.geometry.flip_h = true;
  await installMocks(page, {
    editRecord: { ...NEUTRAL_RECORD, manifest: editsToManifest(edits), hash: 'hash-edited' }
  });
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Geometry' }).click();
  await expect(page.getByRole('button', { name: 'Flip horizontal' })).toHaveAttribute(
    'aria-pressed',
    'true'
  );

  const saved = page.waitForRequest(
    (request) => request.url().endsWith('/edits') && request.method() === 'PUT'
  );
  await page.keyboard.press('ControlOrMeta+Shift+r');
  const body = (await saved).postDataJSON() as {
    manifest: { ops: Record<string, unknown> };
    action: string | null;
  };
  expect(body.action).toBe('Reset Geometry');
  expect(body.manifest.ops.exposure).toBeDefined();
  expect(body.manifest.ops.transform).toBeUndefined();
  await expect(page.getByRole('button', { name: 'Reset geometry', exact: true })).toBeDisabled();
  await expect(page.getByRole('button', { name: 'Flip horizontal' })).toHaveAttribute(
    'aria-pressed',
    'false'
  );
});
