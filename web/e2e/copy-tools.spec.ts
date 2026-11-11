import { expect, test, type Locator, type Page } from '@playwright/test';
import {
  ASSET_ID,
  NEUTRAL_RECORD,
  gotoAsset,
  installMocks,
  numberedAssets,
  type MockAssetSummary
} from './helpers';
import { exposureManifest } from './snapshotMocks';
import { bulkBar } from './browse';
import { mockLibrary, selectTwo } from './touchShell';

interface PasteJob {
  kind: string;
  asset_ids: string[];
  params: { tools: Record<string, boolean> };
}

function exposure(page: Page): Locator {
  return page
    .locator('div.group', { has: page.getByRole('button', { name: 'Exposure', exact: true }) })
    .getByRole('slider');
}

function copyDialog(page: Page): Locator {
  return page.getByRole('dialog', { name: 'Copy settings' });
}

function pasteDialog(page: Page): Locator {
  return page.getByRole('dialog', { name: 'Paste settings' });
}

async function pasteReady(page: Page): Promise<void> {
  await expect(page.getByRole('button', { name: 'Paste edits', exact: true })).toBeEnabled();
}

function checkedTools(job: PasteJob): string[] {
  return Object.entries(job.params.tools)
    .filter(([, on]) => on)
    .map(([tool]) => tool);
}

test.describe('copy and paste per section on desktop', () => {
  test('quick picks check the last touched section or the modified sections', async ({ page }) => {
    await installMocks(page);
    await gotoAsset(page);
    await exposure(page).fill('1');

    await page.getByRole('button', { name: 'Copy edits', exact: true }).click();
    const dialog = copyDialog(page);
    await expect(dialog).toBeVisible();
    await expect(dialog.getByRole('button', { name: 'Copy', exact: true })).toBeEnabled();
    await expect(dialog).toContainText('15 of 15');
    await expect(dialog.getByRole('checkbox', { name: 'Masks', exact: true })).not.toBeChecked();
    await expect(dialog).toContainText('pastes these');

    const lastTouched = dialog.getByRole('button', { name: /^Last touched · Tone$/ });
    await lastTouched.click();
    await expect(lastTouched).toHaveAttribute('aria-pressed', 'true');
    await expect(dialog.getByRole('checkbox', { name: 'Tone', exact: true })).toBeChecked();
    await expect(dialog.getByRole('checkbox', { name: 'HSL', exact: true })).not.toBeChecked();
    await expect(dialog.getByRole('checkbox', { name: 'Basic', exact: true })).toHaveAttribute(
      'data-state',
      'indeterminate'
    );
    await expect(dialog).toContainText(/(^|\D)1 of 15/);

    const modified = dialog.getByRole('button', { name: 'Modified only', exact: true });
    await modified.click();
    await expect(modified).toHaveAttribute('aria-pressed', 'true');
    await expect(lastTouched).toHaveAttribute('aria-pressed', 'false');
    await expect(dialog.getByRole('checkbox', { name: 'Tone', exact: true })).toBeChecked();
    await expect(dialog.getByRole('checkbox', { name: 'Curves', exact: true })).not.toBeChecked();

    await dialog.getByRole('button', { name: 'Check none', exact: true }).click();
    await expect(dialog.getByRole('button', { name: 'Copy', exact: true })).toBeDisabled();
    await dialog.getByRole('button', { name: 'Cancel', exact: true }).click();
    await expect(dialog).toBeHidden();
  });

  test('pastes only the copied section in one step, and Paste only narrows it', async ({
    page
  }) => {
    await installMocks(page);
    await gotoAsset(page);
    await exposure(page).fill('1');

    await page.getByRole('button', { name: 'Copy edits', exact: true }).click();
    const dialog = copyDialog(page);
    await dialog.getByRole('button', { name: /^Last touched/ }).click();
    await dialog.getByRole('button', { name: 'Copy', exact: true }).click();
    await expect(dialog).toBeHidden();

    await exposure(page).fill('0.3');
    await exposure(page).blur();
    await pasteReady(page);
    await page.keyboard.press('ControlOrMeta+v');
    await expect(exposure(page)).toHaveValue('1');
    await page.keyboard.press('ControlOrMeta+z');
    await expect(exposure(page)).toHaveValue('0.3');

    await pasteReady(page);
    await page.keyboard.press('ControlOrMeta+Shift+v');
    const paste = pasteDialog(page);
    await expect(paste).toBeVisible();
    await expect(paste.getByRole('checkbox')).toHaveCount(3);
    await expect(paste.getByRole('checkbox', { name: 'HSL', exact: true })).toHaveCount(0);
    await paste.getByRole('button', { name: 'Check none', exact: true }).click();
    await expect(paste.getByRole('button', { name: 'Paste', exact: true })).toBeDisabled();
    await paste.getByRole('button', { name: 'Cancel', exact: true }).click();
    await expect(paste).toBeHidden();
    await expect(exposure(page)).toHaveValue('0.3');

    await pasteReady(page);
    await page.keyboard.press('ControlOrMeta+Shift+v');
    await expect(paste.getByRole('checkbox', { name: 'Tone', exact: true })).toBeChecked();
    await paste.getByRole('button', { name: 'Paste', exact: true }).click();
    await expect(paste).toBeHidden();
    await expect(exposure(page)).toHaveValue('1');
  });

  test('grid paste and Paste only send just the checked sections', async ({ page }) => {
    const [source, second, third] = numberedAssets(3);
    if (!source || !second || !third) throw new Error('missing assets');
    await installMocks(page, {
      assets: [source, second, third],
      edits: [{ id: source.id, hash: 'h1', updated_at: '2024-05-01T00:00:00Z' }]
    });
    await page.goto('/photos');
    const tile = (name: string) => page.locator(`[data-name="${name}"]`);
    await expect(
      tile(source.originalFileName).getByRole('img', { name: 'Edited', exact: true })
    ).toBeVisible();

    await page.keyboard.press('ArrowRight');
    await page.keyboard.press('ControlOrMeta+c');
    const dialog = copyDialog(page);
    await expect(dialog.getByRole('button', { name: /^Last touched/ })).toBeDisabled();
    await dialog.getByRole('button', { name: 'Check none', exact: true }).click();
    await dialog.getByRole('checkbox', { name: 'HSL', exact: true }).click();
    await dialog.getByRole('checkbox', { name: 'Curves', exact: true }).click();
    await dialog.getByRole('button', { name: 'Copy', exact: true }).click();
    await expect(dialog).toBeHidden();
    await page.keyboard.press('Escape');

    await tile(second.originalFileName).getByRole('button', { name: 'Select' }).click();
    await tile(third.originalFileName).getByRole('button', { name: 'Select' }).click();
    await expect(page.getByText('2 selected')).toBeVisible();

    const pasted = page.waitForRequest(
      (r) => r.method() === 'POST' && r.url().endsWith('/api/jobs')
    );
    await page.keyboard.press('ControlOrMeta+v');
    const job = (await pasted).postDataJSON() as PasteJob;
    expect(job.kind).toBe('paste_edits');
    expect(checkedTools(job)).toEqual(['curves', 'hsl']);

    await page.keyboard.press('ControlOrMeta+Shift+v');
    const paste = pasteDialog(page);
    await expect(paste).toContainText(`From ${source.originalFileName}`);
    await expect(paste.getByRole('checkbox')).toHaveCount(3);
    await paste.getByRole('checkbox', { name: 'Curves', exact: true }).click();
    const narrowed = page.waitForRequest(
      (r) => r.method() === 'POST' && r.url().endsWith('/api/jobs')
    );
    await paste.getByRole('button', { name: 'Paste', exact: true }).click();
    const only = (await narrowed).postDataJSON() as PasteJob;
    expect(only.asset_ids.sort()).toEqual([second.id, third.id].sort());
    expect(checkedTools(only)).toEqual(['hsl']);
    await expect(paste).toBeHidden();

    await page.getByRole('button', { name: 'Edit and export selected' }).click();
    const bulk = page.getByRole('dialog', { name: 'Edit and export selected' });
    await bulk.getByRole('button', { name: 'Paste only…', exact: true }).click();
    await expect(paste.getByRole('checkbox')).toHaveCount(3);
    await paste.getByRole('checkbox', { name: 'HSL', exact: true }).click();
    const fromBulk = page.waitForRequest(
      (r) => r.method() === 'POST' && r.url().endsWith('/api/jobs')
    );
    await paste.getByRole('button', { name: 'Paste', exact: true }).click();
    const curves = (await fromBulk).postDataJSON() as PasteJob;
    expect(curves.asset_ids.sort()).toEqual([second.id, third.id].sort());
    expect(checkedTools(curves)).toEqual(['curves']);
  });
});

test.describe('copy and paste per section on touch', { tag: ['@touch'] }, () => {
  async function openCopySheet(page: Page, assets?: MockAssetSummary[]): Promise<Locator> {
    await mockLibrary(page, {
      assets,
      editRecord: { ...NEUTRAL_RECORD, manifest: exposureManifest(0.5), hash: 'hash-exposure' }
    });
    await page.goto(`/assets/${ASSET_ID}`);
    await expect(page.locator('html')).toHaveAttribute('data-layout', 'touch');
    await page.getByRole('button', { name: 'More editor actions' }).tap();
    await page.getByRole('button', { name: 'Copy edits…', exact: true }).tap();
    const sheet = copyDialog(page);
    await expect(sheet).toBeVisible();
    return sheet;
  }

  async function edge(locator: Locator, axis: 'x' | 'y'): Promise<number> {
    const box = await locator.boundingBox();
    return Math.round(box?.[axis] ?? Number.NaN);
  }

  async function expectLayout(sheet: Locator, wide: boolean): Promise<void> {
    const picks = sheet.getByRole('group', { name: 'Quick pick' });
    const last = picks.getByRole('button', { name: /^Last touched/ });
    const modified = picks.getByRole('button', { name: 'Modified only', exact: true });
    await expect.poll(async () => (await edge(modified, 'y')) - (await edge(last, 'y'))).toBe(0);
    await expect(picks.getByText('Quick pick', { exact: true })).toBeVisible({ visible: wide });
    const develop = sheet.getByRole('checkbox', { name: 'Develop', exact: true });
    const masks = sheet.getByRole('checkbox', { name: 'Masks', exact: true });
    const offset = async () => (await edge(masks, 'x')) - (await edge(develop, 'x'));
    if (wide) {
      await expect.poll(offset).toBeGreaterThan(200);
    } else {
      await expect.poll(offset).toBe(0);
    }
  }

  test('quick picks stay on one line and the columns follow the sheet width', async ({ page }) => {
    const sheet = await openCopySheet(page);
    await expectLayout(sheet, (page.viewportSize()?.width ?? 0) >= 700);
  });

  test('the copy sheet lists every section in 44 px rows and pastes a chosen subset', async ({
    page
  }) => {
    const sheet = await openCopySheet(page);
    for (const [name, text] of [
      ['Check all', 'All'],
      ['Check none', 'None']
    ]) {
      const action = sheet.getByRole('button', { name, exact: true });
      await expect(action.getByText(text, { exact: true })).toBeVisible();
      await expect(action.getByText(name, { exact: true })).toBeHidden();
    }
    await expect(sheet).toContainText('Then open another photo and use More › Paste edits');
    const tone = sheet.getByRole('checkbox', { name: 'Tone', exact: true });
    const row = tone.locator('xpath=../..');
    const box = await row.boundingBox();
    expect(Math.round(box?.height ?? 0)).toBeGreaterThanOrEqual(44);

    await sheet.getByRole('button', { name: 'Check none', exact: true }).tap();
    await expect(sheet.getByRole('button', { name: 'Copy', exact: true })).toBeDisabled();
    await tone.tap();
    await sheet.getByRole('checkbox', { name: 'Vignette', exact: true }).tap();
    await sheet.getByRole('button', { name: 'Copy', exact: true }).tap();
    await expect(sheet).toBeHidden();

    await page.getByRole('button', { name: 'More editor actions' }).tap();
    await expect(page.getByRole('button', { name: 'Paste edits', exact: true })).toContainText(
      'from IMG_'
    );
    await page.getByRole('button', { name: 'Paste only…', exact: true }).tap();
    const paste = pasteDialog(page);
    await expect(paste).toBeVisible();
    await expect(paste.getByRole('checkbox')).toHaveCount(5);
    await paste.getByRole('checkbox', { name: 'Vignette', exact: true }).tap();
    await paste.getByRole('button', { name: 'Paste', exact: true }).tap();
    await expect(paste).toBeHidden();
    await expect(page.getByRole('status').filter({ hasText: 'Pasted settings' })).toBeVisible();
  });

  test('bulk More pastes only the checked sections onto the selection', async ({ page }) => {
    const assets = numberedAssets(3);
    const sheet = await openCopySheet(page, assets);
    await sheet.getByRole('button', { name: 'Check none', exact: true }).tap();
    await sheet.getByRole('checkbox', { name: 'Tone', exact: true }).tap();
    await sheet.getByRole('checkbox', { name: 'Vignette', exact: true }).tap();
    await sheet.getByRole('button', { name: 'Copy', exact: true }).tap();
    await expect(sheet).toBeHidden();
    await page.getByRole('button', { name: /^Back/ }).tap();
    await expect(page).toHaveURL('/photos');

    await selectTwo(page);
    await bulkBar(page).getByRole('button', { name: 'More', exact: true }).tap();
    await page
      .getByRole('dialog', { name: 'More actions' })
      .getByRole('button', { name: 'Paste only…', exact: true })
      .tap();
    const paste = pasteDialog(page);
    await expect(paste).toBeVisible();
    await expect(paste.getByRole('checkbox')).toHaveCount(5);
    await paste.getByRole('checkbox', { name: 'Vignette', exact: true }).tap();
    const job = page.waitForRequest((r) => r.method() === 'POST' && r.url().endsWith('/api/jobs'));
    await paste.getByRole('button', { name: 'Paste', exact: true }).tap();
    const body = (await job).postDataJSON() as PasteJob;
    expect(body.asset_ids.sort()).toEqual(
      assets
        .slice(0, 2)
        .map((a) => a.id)
        .sort()
    );
    expect(checkedTools(body)).toEqual(['tone']);
    await expect(paste).toBeHidden();
  });

  test('@phone the copy sheet works in landscape', async ({ page }) => {
    await page.setViewportSize({ width: 844, height: 390 });
    const sheet = await openCopySheet(page);
    await expectLayout(sheet, true);
    const confirm = sheet.getByRole('button', { name: 'Copy', exact: true });
    await expect(confirm).toBeInViewport();
    await sheet.getByRole('checkbox', { name: 'Masks', exact: true }).scrollIntoViewIfNeeded();
    await expect(sheet.getByRole('checkbox', { name: 'Masks', exact: true })).toBeInViewport();
    await confirm.tap();
    await expect(sheet).toBeHidden();
  });
});
