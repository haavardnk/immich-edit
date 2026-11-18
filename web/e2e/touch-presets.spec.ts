import { expect, test, type Locator, type Page } from '@playwright/test';
import { editsToManifest } from '../src/lib/edits/manifest';
import { neutralEdits } from '../src/lib/types/edits';
import { ASSET_ID, type PreviewRequest } from './helpers';
import { tapCentre, touchDown, touchMoveTo, touchUp } from './touch';
import { open } from './touchShell';

function preset(id: string, name: string, group_name: string | null, exposure = 0) {
  const edits = neutralEdits();
  edits.basic.exposure_ev = exposure;
  return {
    id,
    name,
    group_name,
    manifest: editsToManifest(edits),
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z'
  };
}

const PRESETS = [
  preset('preset-warm', 'Warm', 'Film', 1),
  preset('preset-kodak', 'Kodak 400', 'Film'),
  preset('preset-mono', 'Mono', 'B&W')
];

type Opened = {
  strip: Locator;
  saves: Array<Record<string, unknown>>;
  previews: PreviewRequest[];
  deleted: string[];
};

async function openPresets(page: Page): Promise<Opened> {
  const saves: Array<Record<string, unknown>> = [];
  const previews: PreviewRequest[] = [];
  const deleted: string[] = [];
  await open(page, `/assets/${ASSET_ID}`, {
    presets: PRESETS.map((p) => ({ ...p })),
    onSave: (body) => saves.push(body),
    onPreview: (req) => previews.push(req),
    onPresetDelete: (id) => deleted.push(id)
  });
  await page
    .getByRole('tablist', { name: 'Editor modes' })
    .getByRole('tab', { name: 'Presets' })
    .tap();
  const strip = page.getByRole('region', { name: 'Presets' });
  await expect(tile(strip, 'Warm')).toBeEnabled();
  return { strip, saves, previews, deleted };
}

function tile(strip: Locator, name: string): Locator {
  return strip.getByRole('button', { name: `Apply ${name}`, exact: true });
}

function pill(page: Page): Locator {
  return page.locator('[data-photo-pill]');
}

function previewExposure(requests: PreviewRequest[]): unknown {
  return (requests.at(-1)?.edits as { basic?: { exposure_ev?: number } } | undefined)?.basic
    ?.exposure_ev;
}

async function centre(target: Locator): Promise<{ x: number; y: number }> {
  const box = await target.boundingBox();
  if (!box) throw new Error('target has no box');
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}

test.describe('touch presets', { tag: '@touch' }, () => {
  test('the tray is a header row over a square grid led by None', async ({ page }) => {
    const { strip } = await openPresets(page);
    const chips = strip.getByRole('tablist', { name: 'Preset groups' });
    await expect(chips.getByRole('tab')).toHaveText(['All', 'B&W', 'Film']);
    await expect(strip.getByRole('button', { name: 'Search presets' })).toBeVisible();
    await expect(strip.getByRole('button', { name: 'Save current as preset' })).toBeVisible();
    await expect(strip.getByRole('button', { name: 'Import presets' })).toHaveCount(0);
    await expect(strip.getByRole('button', { name: /^Apply / })).toHaveText([
      'Warm',
      'Kodak 400',
      'Mono'
    ]);
    const none = strip.getByRole('button', { name: 'No preset' });
    await expect(none).toHaveAttribute('aria-current', 'true');
    const thumb = await tile(strip, 'Warm').locator('span').first().boundingBox();
    expect(Math.round(thumb?.width ?? 0)).toBe(Math.round(thumb?.height ?? -1));
    await expect(strip.getByRole('slider', { name: 'Amount' })).toHaveCount(0);

    await strip.getByRole('button', { name: 'Preset actions' }).tap();
    await expect(page.getByRole('dialog', { name: 'Presets' }).getByRole('button')).toContainText([
      'Import presets',
      'Export all presets'
    ]);
    await page.keyboard.press('Escape');

    await chips.getByRole('tab', { name: 'Film' }).tap();
    await expect(strip.getByRole('button', { name: /^Apply / })).toHaveText(['Warm', 'Kodak 400']);
  });

  test('an applied preset shows its card and None removes it', async ({ page }) => {
    const { strip, saves } = await openPresets(page);
    await tile(strip, 'Warm').tap();
    const card = strip.getByRole('region', { name: 'Applied preset' });
    await expect(card).toContainText('Warm');
    await expect(card.getByRole('slider', { name: 'Amount' })).toHaveValue('100');
    await expect(card.getByText('Crop', { exact: true })).toHaveCount(0);
    await expect(tile(strip, 'Warm')).toHaveAttribute('aria-current', 'true');

    const saved = saves.length;
    await strip.getByRole('button', { name: 'No preset' }).tap();
    await expect(card).toHaveCount(0);
    await expect(strip.getByRole('button', { name: 'No preset' })).toHaveAttribute(
      'aria-current',
      'true'
    );
    await expect.poll(() => saves.length).toBeGreaterThan(saved);
  });

  test('a tap applies the preset as one history step', async ({ page }) => {
    const { strip, saves } = await openPresets(page);
    await tapCentre(page, tile(strip, 'Warm'));
    await expect.poll(() => saves.map((body) => body.action)).toEqual(['Preset: Warm']);
  });

  test('holding previews, release cancels and sliding off cancels', async ({ page }) => {
    const { strip, saves, previews } = await openPresets(page);
    const warm = tile(strip, 'Warm');
    const point = await centre(warm);

    await touchDown(page, point);
    await expect(pill(page)).toHaveText('Previewing Warm · release to cancel');
    await expect.poll(() => previewExposure(previews)).toBe(1);
    await touchUp(page);
    await expect(pill(page)).toHaveCount(0);
    await expect.poll(() => previewExposure(previews)).toBe(0);

    await touchDown(page, point);
    await expect(pill(page)).toBeVisible();
    await touchMoveTo(page, { x: point.x, y: point.y - 120 });
    await expect(pill(page)).toHaveCount(0);
    await touchUp(page);
    await page.waitForTimeout(300);
    expect(saves).toEqual([]);
  });

  test('the tile menu opens as an action sheet and deletes after confirming', async ({ page }) => {
    const { strip, deleted } = await openPresets(page);
    await strip.getByRole('button', { name: 'Actions for Mono' }).tap();
    const sheet = page.getByRole('dialog', { name: 'Mono' });
    await expect(sheet.getByRole('button')).toContainText(['Edit…', 'Export', 'Delete']);
    await sheet.getByRole('button', { name: 'Delete', exact: true }).tap();
    await sheet.getByRole('button', { name: 'Confirm delete' }).tap();
    await expect.poll(() => deleted).toEqual(['preset-mono']);
    await expect(tile(strip, 'Mono')).toHaveCount(0);
  });
});

test.describe('touch presets in phone landscape', { tag: ['@touch', '@phone'] }, () => {
  test.use({ viewport: { width: 915, height: 412 } });

  test('the side panel wraps tiles into a grid', async ({ page }) => {
    const { strip } = await openPresets(page);
    const panel = page.getByRole('complementary', { name: 'Adjustments' });
    await expect(panel.getByRole('region', { name: 'Presets' })).toBeVisible();
    const warm = await tile(strip, 'Warm').boundingBox();
    const mono = await tile(strip, 'Mono').boundingBox();
    expect(mono?.y ?? 0).toBeGreaterThan((warm?.y ?? 0) + (warm?.height ?? 0));
    const box = await panel.boundingBox();
    expect((mono?.x ?? 0) + (mono?.width ?? 0)).toBeLessThanOrEqual(
      (box?.x ?? 0) + (box?.width ?? 0)
    );
  });
});
