import { expect, test, type Page } from '@playwright/test';
import { ASSET_ID, NEUTRAL_RECORD, type InstallOpts } from './helpers';
import { FULL_RETOUCH_RECORD } from './retouchRecords';
import { drag, fingersDown, fingersMove, tapAt, touchDown, touchMoveTo, touchUp } from './touch';
import { modes } from './touchMasks';
import { mockLibrary } from './touchShell';

type Box = { x: number; y: number; width: number; height: number };
type Saved = { manifest?: { ops?: { retouch?: { strokes?: { enabled?: boolean }[] } } } };

function spotsRecord(xs: number[]) {
  return {
    ...NEUTRAL_RECORD,
    hash: `hash-${xs.length}-strokes`,
    manifest: {
      schema_version: 3,
      ops: {
        retouch: {
          strokes: xs.map((x, i) => ({
            id: `spot-${i}`,
            mode: i % 2 === 0 ? 'heal' : 'clone',
            points: [{ x, y: 0.4 }],
            radius: 0.04,
            hardness: 0.5,
            opacity: 1,
            source: { x, y: 0.7 },
            enabled: true
          }))
        }
      }
    }
  };
}

const TWO_STROKES = spotsRecord([0.3, 0.6]);
const SIX_STROKES = spotsRecord([0.2, 0.3, 0.4, 0.5, 0.6, 0.7]);

async function openRetouch(page: Page, opts: InstallOpts = {}): Promise<void> {
  await mockLibrary(page, opts);
  await page.goto(`/assets/${ASSET_ID}`);
  await expect(page.locator('html')).toHaveAttribute('data-layout', 'touch');
  await modes(page).getByRole('tab', { name: 'Retouch' }).tap();
  await expect(tray(page)).toBeVisible();
}

const tray = (page: Page) => page.getByTestId('touch-retouch');
const banner = (page: Page) => page.getByTestId('retouch-source-banner');
const pill = (page: Page) => page.locator('[data-photo-pill]');

async function canvasBox(page: Page): Promise<Box> {
  const box = await page.getByLabel('retouch canvas').boundingBox();
  if (!box) throw new Error('retouch canvas has no box');
  return box;
}

function at(box: Box, u: number, v: number): { x: number; y: number } {
  return { x: box.x + box.width * u, y: box.y + box.height * v };
}

function strokesOf(saves: Array<Record<string, unknown>>): { enabled?: boolean }[] {
  return (saves.at(-1) as Saved | undefined)?.manifest?.ops?.retouch?.strokes ?? [];
}

async function setSource(page: Page, box: Box): Promise<void> {
  await expect(banner(page)).toContainText('Tap the photo to set the source');
  const source = at(box, 0.75, 0.3);
  await tapAt(page, source.x, source.y);
  await expect(banner(page)).toHaveCount(0);
  await expect(tray(page).getByText('Set', { exact: true })).toBeVisible();
}

test.describe('touch retouch on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('the first tap sets the source and one finger paints a stroke', async ({ page }) => {
    const saves: Array<Record<string, unknown>> = [];
    await openRetouch(page, { onSave: (body) => saves.push(body) });
    await expect(tray(page)).toContainText('No strokes yet');
    await expect(tray(page).locator('header')).toHaveCount(0);
    await expect(tray(page).getByRole('button', { name: 'Set source' })).toHaveAttribute(
      'aria-pressed',
      'true'
    );
    const box = await canvasBox(page);
    await setSource(page, box);

    const from = at(box, 0.3, 0.5);
    await touchDown(page, from);
    for (let step = 1; step <= 8; step += 1) {
      await page.waitForTimeout(16);
      await touchMoveTo(page, { x: from.x + step * 6, y: from.y });
    }
    await expect(page.getByTestId('touch-magnifier')).toBeVisible();
    await expect(pill(page)).toContainText('Heal');
    await page.waitForTimeout(150);
    await touchUp(page);

    await expect.poll(() => strokesOf(saves).length).toBe(1);
    await expect(page.getByTestId('touch-magnifier')).toHaveCount(0);
    await expect(tray(page).getByRole('button', { name: 'Stroke 1', exact: true })).toBeVisible();
    await expect(tray(page)).toContainText('Heal · drag the ring to move it');
    await tray(page).getByRole('button', { name: 'Done', exact: true }).tap();
    await expect(tray(page).locator('header')).toHaveCount(0);
    await expect(tray(page).getByRole('button', { name: 'Stroke 1', exact: true })).toHaveAttribute(
      'aria-pressed',
      'false'
    );
    await expect(tray(page).getByRole('button', { name: 'Retouch actions' })).toBeVisible();
    await expect(tray(page).getByRole('button', { name: 'Done', exact: true })).toHaveCount(0);
  });

  test('a second finger within 120 ms drops the stroke', async ({ page }) => {
    const saves: Array<Record<string, unknown>> = [];
    await openRetouch(page, { onSave: (body) => saves.push(body) });
    const box = await canvasBox(page);
    await setSource(page, box);

    const first = at(box, 0.3, 0.5);
    const second = at(box, 0.6, 0.6);
    await fingersDown(page, [first]);
    await fingersDown(page, [first, second]);
    await fingersMove(page, [
      { x: first.x - 20, y: first.y - 20 },
      { x: second.x + 20, y: second.y + 20 }
    ]);
    await touchUp(page);
    await page.waitForTimeout(400);
    expect(strokesOf(saves)).toHaveLength(0);
    await expect(page.getByTestId('retouch-strokes')).toHaveCount(0);
  });

  test('painting without a source says so in the banner', async ({ page }) => {
    await openRetouch(page);
    await banner(page).getByRole('button', { name: 'Stop picking the source' }).tap();
    await expect(banner(page)).toHaveCount(0);
    const box = await canvasBox(page);
    await drag(page, at(box, 0.3, 0.5), at(box, 0.5, 0.5), { settleMs: 150 });

    await expect(banner(page)).toContainText('Set a source before painting');
    await banner(page).getByRole('button', { name: 'Set source' }).tap();
    await expect(banner(page)).toContainText('Tap the photo to set the source');
  });

  test('a tap on the photo at 64 strokes says so', async ({ page }) => {
    await openRetouch(page, { editRecord: FULL_RETOUCH_RECORD });
    await expect(page.getByTestId('retouch-limit')).toContainText(
      'Limit of 64 retouch strokes reached.'
    );
    const box = await canvasBox(page);
    await expect(banner(page)).toHaveCount(0);
    await tray(page).getByRole('button', { name: 'Set source' }).tap();
    await setSource(page, box);
    const empty = at(box, 0.6, 0.7);
    await tapAt(page, empty.x, empty.y);
    await expect(pill(page)).toHaveText('Limit of 64 retouch strokes reached.');
  });

  test('stroke chips and the photo select strokes, and the menu removes all', async ({ page }) => {
    const saves: Array<Record<string, unknown>> = [];
    await openRetouch(page, { editRecord: TWO_STROKES, onSave: (body) => saves.push(body) });
    await expect(banner(page)).toHaveCount(0);
    const box = await canvasBox(page);
    const spot = at(box, 0.6, 0.4);
    await tapAt(page, spot.x, spot.y);
    const chip = tray(page).getByRole('button', { name: 'Stroke 2', exact: true });
    await expect(chip).toHaveAttribute('aria-pressed', 'true');
    await tray(page).getByRole('button', { name: 'Done', exact: true }).tap();
    await expect(chip).toHaveAttribute('aria-pressed', 'false');
    await chip.tap();
    await expect(chip).toHaveAttribute('aria-pressed', 'true');
    await expect(tray(page)).toContainText('Clone · drag the ring to move it');
    await expect(tray(page).getByRole('radio', { name: 'Clone' })).toBeChecked();

    await tray(page).getByRole('button', { name: 'Hide Stroke 2' }).tap();
    await expect.poll(() => strokesOf(saves)[1]?.enabled).toBe(false);
    await tray(page).getByRole('button', { name: 'Delete Stroke 1' }).tap();
    await expect.poll(() => strokesOf(saves).length).toBe(1);

    await tray(page).getByRole('button', { name: 'Retouch actions' }).tap();
    await page.getByRole('dialog').getByRole('button', { name: 'Remove all' }).tap();
    await expect.poll(() => strokesOf(saves).length).toBe(0);
    await expect(tray(page)).toContainText('No strokes yet');
  });

  test('earlier stroke chips come back by swipe, wheel and photo tap', async ({ page }) => {
    await openRetouch(page, { editRecord: SIX_STROKES });
    const chip = (n: number) =>
      tray(page).getByRole('button', { name: `Stroke ${n}`, exact: true });
    await expect(chip(6)).toBeInViewport({ ratio: 1 });
    await expect(chip(1)).not.toBeInViewport();

    const box = await canvasBox(page);
    const first = at(box, 0.2, 0.4);
    await tapAt(page, first.x, first.y);
    await expect(chip(1)).toHaveAttribute('aria-pressed', 'true');
    await expect(chip(1)).toBeInViewport({ ratio: 1 });

    const strip = page.getByTestId('retouch-strokes');
    const row = await strip.boundingBox();
    if (!row) throw new Error('stroke strip has no box');
    const y = row.y + row.height / 2;
    await drag(page, { x: row.x + row.width - 40, y }, { x: row.x + 40, y }, { settleMs: 150 });
    await expect(chip(1)).not.toBeInViewport();

    await page.mouse.move(row.x + row.width / 2, y);
    await page.mouse.wheel(0, -2000);
    await expect(chip(1)).toBeInViewport({ ratio: 1 });
  });

  test('size sits on the photo edge and the menu turns on Visualize spots', async ({ page }) => {
    await openRetouch(page);
    await expect(tray(page).getByRole('slider', { name: 'Size' })).toHaveCount(0);
    await expect(tray(page).getByRole('slider', { name: 'Hardness' })).toBeVisible();
    const rail = page.getByTestId('brush-size-rail');
    await expect(rail).toBeVisible();
    const before = await rail.textContent();
    const thumb = await rail.getByRole('slider', { name: 'Brush size' }).boundingBox();
    if (!thumb) throw new Error('size thumb has no box');
    const centre = { x: thumb.x + thumb.width / 2, y: thumb.y + thumb.height / 2 };
    await drag(page, centre, { x: centre.x, y: centre.y - 40 }, { settleMs: 150 });
    await expect(rail).not.toHaveText(before ?? '');

    await tray(page).getByRole('button', { name: 'Retouch actions' }).tap();
    await page.getByRole('dialog').getByRole('button', { name: 'Visualize spots' }).tap();
    await expect(page.getByTestId('preview-image')).toHaveAttribute(
      'style',
      /filter: url\("?#retouch-visualize"?\)/
    );
  });
});

test.describe('touch retouch on a tablet', { tag: ['@touch', '@tablet'] }, () => {
  test('the inspector keeps the size row and one finger paints', async ({ page }) => {
    const saves: Array<Record<string, unknown>> = [];
    await openRetouch(page, { onSave: (body) => saves.push(body) });
    await expect(tray(page).getByRole('slider', { name: 'Size' })).toBeVisible();
    await expect(page.getByTestId('brush-size-rail')).toHaveCount(0);
    const box = await canvasBox(page);
    await setSource(page, box);
    await drag(page, at(box, 0.3, 0.5), at(box, 0.45, 0.55), { settleMs: 150 });
    await expect.poll(() => strokesOf(saves).length).toBe(1);
  });
});
