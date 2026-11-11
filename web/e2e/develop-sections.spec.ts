import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_ID, installMocks, type InstallOpts } from './helpers';
import { drag } from './touch';
import { open } from './touchShell';

type Entry = { id: string; hidden: boolean };

function sectionsSaved(key: string, saved: unknown[]): InstallOpts['onPreferences'] {
  return (patch) => {
    if (key in patch) saved.push(patch[key]);
  };
}

function hiddenIds(saved: unknown[]): string[] {
  const last = saved.at(-1) as Entry[] | null;
  return (last ?? []).filter((entry) => entry.hidden).map((entry) => entry.id);
}

function orderOf(saved: unknown[]): string[] {
  return ((saved.at(-1) as Entry[] | null) ?? []).map((entry) => entry.id);
}

async function centre(target: Locator): Promise<{ x: number; y: number }> {
  let previous = Number.NaN;
  await expect
    .poll(async () => {
      const y = (await target.boundingBox())?.y ?? Number.NaN;
      const still = Math.abs(y - previous) < 0.5;
      previous = y;
      return still;
    })
    .toBe(true);
  const box = await target.boundingBox();
  if (!box) throw new Error('target has no box');
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}

test.describe('develop sections on desktop', () => {
  async function openEditor(page: Page, opts: InstallOpts = {}): Promise<void> {
    await installMocks(page, opts);
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.goto(`/assets/${ASSET_ID}`);
    await expect(page.getByRole('slider', { name: 'Exposure' })).toBeVisible();
  }

  async function openMenu(page: Page): Promise<Locator> {
    await page.getByRole('button', { name: 'Customize sections' }).click();
    const menu = page.locator('[data-popover-content]');
    await expect(menu).toContainText('Drag to reorder · the eye hides a section');
    return menu;
  }

  function panelOrder(page: Page): Promise<(string | null)[]> {
    return page
      .locator('[data-panel]:not([data-panel="quick"])')
      .evaluateAll((panels) => panels.map((panel) => panel.getAttribute('data-panel')));
  }

  test('hides, reorders and resets sections', async ({ page }) => {
    const saved: unknown[] = [];
    await openEditor(page, { onPreferences: sectionsSaved('desktop_sections', saved) });
    const menu = await openMenu(page);
    await expect(menu).toContainText(/Scopes\s+always first/);

    await menu.getByRole('button', { name: 'Hide LUT' }).click();
    await expect(page.locator('[data-panel="lut"]')).toHaveCount(0);
    await expect(menu.getByRole('button', { name: 'Show LUT' })).toBeVisible();
    await expect.poll(() => hiddenIds(saved)).toEqual(['lut']);

    await menu.getByRole('button', { name: 'Hide Lens Corrections' }).click();
    await expect(page.locator('[data-panel="lens"]')).toBeVisible();
    await expect.poll(() => hiddenIds(saved)).toEqual(['lut', 'lens']);

    await menu.getByRole('button', { name: 'Reorder Effects' }).focus();
    await page.keyboard.press('ArrowUp');
    await expect
      .poll(() => panelOrder(page))
      .toEqual([
        'dcp',
        'basic',
        'curves',
        'hsl',
        'color-grading',
        'bw',
        'detail',
        'lens',
        'effects',
        'lens-blur'
      ]);

    const grip = menu.getByRole('button', { name: 'Reorder Detail' });
    const start = await centre(grip);
    const row = await grip.locator('xpath=ancestor::li').boundingBox();
    await page.mouse.move(start.x, start.y);
    await page.mouse.down();
    await page.mouse.move(start.x, start.y - (row?.height ?? 32) * 3, { steps: 8 });
    await page.mouse.up();
    await expect
      .poll(() => orderOf(saved).slice(0, 5))
      .toEqual(['dcp', 'basic', 'curves', 'hsl', 'detail']);

    await menu.getByRole('button', { name: 'Reset to default' }).click();
    await expect.poll(() => saved.at(-1)).toBeNull();
    await expect(page.locator('[data-panel="lut"]')).toBeVisible();
    await expect(menu.getByRole('button', { name: 'Reset to default' })).toBeDisabled();
  });

  test('a hidden section still answers the filter and the palette', async ({ page }) => {
    await openEditor(page, { desktopSections: [{ id: 'detail', hidden: true }] });
    await expect(page.locator('[data-panel="detail"]')).toHaveCount(0);

    const filter = page.getByRole('textbox', { name: 'Search settings', exact: true });
    await filter.fill('sharpen amount');
    await expect(page.locator('[data-panel="detail"]')).toBeVisible();
    await filter.press('Escape');
    await expect(page.locator('[data-panel="detail"]')).toHaveCount(0);

    await page.keyboard.press('ControlOrMeta+k');
    const palette = page.getByRole('dialog', { name: 'Search settings and actions' });
    await palette.getByRole('combobox').fill('sharpen amount');
    await palette
      .getByRole('option', { name: /Sharpen amount/ })
      .first()
      .click();
    await expect(page.locator('[data-panel="detail"]')).toBeVisible();
  });
});

async function openTouchSheet(page: Page, saved: unknown[]): Promise<Locator> {
  await open(page, `/assets/${ASSET_ID}`, {
    onPreferences: sectionsSaved('touch_subviews', saved)
  });
  await expect(page.getByRole('slider', { name: 'Exposure' })).toBeVisible();
  await page.getByRole('button', { name: 'More editor actions' }).tap();
  await page.getByRole('button', { name: 'Customize Develop', exact: true }).tap();
  const sheet = page.getByRole('dialog', { name: 'Develop sections' });
  await expect(sheet).toBeVisible();
  return sheet;
}

function views(page: Page, group: string): Locator {
  return page.getByRole('tablist', { name: `${group} views` }).getByRole('tab');
}

async function pickGroup(page: Page, name: string): Promise<void> {
  await page.getByRole('tablist', { name: 'Develop groups' }).getByRole('tab', { name }).tap();
}

for (const tag of ['@phone', '@tablet']) {
  test.describe(`develop sections on touch ${tag}`, { tag: ['@touch', tag] }, () => {
    test('hiding a view removes its tab and reordering stays in its group', async ({ page }) => {
      const saved: unknown[] = [];
      const sheet = await openTouchSheet(page, saved);
      await expect(sheet.getByRole('button', { name: 'Reorder Corrections' })).toBeDisabled();

      await sheet.getByRole('switch', { name: 'Show LUT' }).tap();
      await expect.poll(() => hiddenIds(saved)).toEqual(['profile.lut']);

      await sheet.getByRole('button', { name: 'Reorder Curve' }).focus();
      await page.keyboard.press('ArrowUp');
      await expect.poll(() => orderOf(saved).slice(0, 2)).toEqual(['light.curve', 'light.tone']);

      await sheet.getByRole('button', { name: 'Done' }).tap();
      await expect(sheet).toHaveCount(0);
      await expect(views(page, 'Light')).toHaveText(['Curve', 'Tone']);
      await pickGroup(page, 'Profile');
      await expect(views(page, 'Profile')).toHaveText(['Camera']);
    });

    test('a drag on the handle reorders a group', async ({ page }) => {
      const saved: unknown[] = [];
      const sheet = await openTouchSheet(page, saved);
      const grip = sheet.getByRole('button', { name: 'Reorder Grading' });
      const row = await grip.locator('xpath=ancestor::li').boundingBox();
      const from = await centre(grip);
      await drag(page, from, { x: from.x, y: from.y - (row?.height ?? 44) * 2 });
      await expect
        .poll(() => orderOf(saved).filter((id) => id.startsWith('color.')))
        .toEqual(['color.grading', 'color.basic', 'color.mix', 'color.bw']);

      await sheet.getByRole('button', { name: 'Reset to default' }).tap();
      await expect.poll(() => saved.at(-1)).toBeNull();
    });
  });
}
