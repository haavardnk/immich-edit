import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_ID, gotoAsset, installMocks, type PreviewRequest } from './helpers';
import { tapCentre, touchDown, touchUp } from './touch';
import { mockLibrary } from './touchShell';

const LUTS = [
  { id: 'lut-kodak', name: 'Kodak 2383', lut_size: 33, shaper_size: 0, size: 1, created_at: '' },
  { id: 'lut-teal', name: 'Teal Orange', lut_size: 33, shaper_size: 0, size: 1, created_at: '' },
  { id: 'lut-fuji', name: 'Fuji 3510', lut_size: 33, shaper_size: 0, size: 1, created_at: '' }
];

interface Recorded {
  previews: PreviewRequest[];
  saves: Array<Record<string, unknown>>;
}

function record(): Recorded & {
  opts: Parameters<typeof installMocks>[1];
} {
  const previews: PreviewRequest[] = [];
  const saves: Array<Record<string, unknown>> = [];
  return {
    previews,
    saves,
    opts: {
      luts: LUTS,
      onPreview: (req) => previews.push(req),
      onSave: (body) => saves.push(body)
    }
  };
}

function lutOf(req: PreviewRequest | undefined): unknown {
  return (req?.edits as { color?: { lut_3d?: { lut_id?: unknown } } } | undefined)?.color?.lut_3d
    ?.lut_id;
}

function lastLut(previews: PreviewRequest[]): unknown {
  return lutOf(previews.filter((req) => req.lane !== 'reference').at(-1));
}

function pill(page: Page): Locator {
  return page.locator('[data-photo-pill]');
}

function selections(saves: Array<Record<string, unknown>>): number {
  return saves.filter((body) => body.action === 'Select LUT').length;
}

test.describe('LUT hover preview on desktop', () => {
  async function openList(page: Page, recorded: ReturnType<typeof record>): Promise<Locator> {
    await installMocks(page, recorded.opts);
    await gotoAsset(page);
    const section = page.getByRole('button', { name: 'LUT', exact: true });
    if ((await section.getAttribute('aria-expanded')) !== 'true') await section.click();
    await page.getByRole('combobox', { name: 'Select LUT…' }).click();
    const list = page.getByRole('listbox');
    await expect(list.getByRole('option')).toHaveCount(LUTS.length);
    return list;
  }

  test('hovering a LUT previews it and leaving the list restores the photo', async ({ page }) => {
    const recorded = record();
    const list = await openList(page, recorded);

    const teal = list.getByRole('option', { name: /Teal Orange/ });
    await teal.hover();
    await expect(pill(page)).toContainText('Previewing LUT · Teal Orange');
    await expect(pill(page)).toContainText('Click to apply · move away to restore');
    await expect(teal.getByText('previewing', { exact: true })).toBeVisible();
    await expect.poll(() => lastLut(recorded.previews)).toBe('lut-teal');
    await expect(page.getByText('preview only, nothing committed')).toBeVisible();

    await list.getByRole('option', { name: /Fuji 3510/ }).hover();
    await expect(pill(page)).toContainText('Fuji 3510');
    await expect.poll(() => lastLut(recorded.previews)).toBe('lut-fuji');

    const box = await list.boundingBox();
    if (!box) throw new Error('no list');
    await page.mouse.move(box.x + box.width / 2, box.y + box.height + 120);
    await expect(pill(page)).toBeHidden();
    await expect(list).toBeVisible();
    await expect(list.getByText('previewing', { exact: true })).toHaveCount(0);
    await expect.poll(() => lastLut(recorded.previews)).toBeNull();
    expect(selections(recorded.saves)).toBe(0);
  });

  test('arrow keys preview the highlighted LUT and Escape restores', async ({ page }) => {
    const recorded = record();
    const list = await openList(page, recorded);
    await expect(pill(page)).toBeHidden();

    await page.keyboard.press('ArrowDown');
    const highlighted = list.locator('[data-highlighted]');
    await expect(highlighted).toContainText('previewing');
    await expect(pill(page)).toContainText('Previewing LUT ·');
    const first = lastLut(recorded.previews);
    await expect.poll(() => lastLut(recorded.previews)).not.toBeNull();

    await page.keyboard.press('ArrowDown');
    await expect(highlighted).toContainText('previewing');
    await expect.poll(() => lastLut(recorded.previews)).not.toBe(first);
    await expect(list.getByText('previewing', { exact: true })).toHaveCount(1);

    await page.keyboard.press('Escape');
    await expect(list).toBeHidden();
    await expect(pill(page)).toBeHidden();
    await expect.poll(() => lastLut(recorded.previews)).toBeNull();
    expect(selections(recorded.saves)).toBe(0);
  });

  test('a click applies the hovered LUT as one history step', async ({ page }) => {
    const recorded = record();
    const list = await openList(page, recorded);

    const teal = list.getByRole('option', { name: /Teal Orange/ });
    await teal.hover();
    await expect(pill(page)).toBeVisible();
    await expect.poll(() => lastLut(recorded.previews)).toBe('lut-teal');
    const before = recorded.previews.length;

    await teal.click();
    await expect(list).toBeHidden();
    await expect(pill(page)).toBeHidden();
    await expect.poll(() => selections(recorded.saves)).toBe(1);
    await expect(page.getByRole('combobox', { name: 'Select LUT…' })).toHaveValue('Teal Orange');
    const after = recorded.previews.slice(before).filter((req) => req.lane !== 'reference');
    expect(after.every((req) => lutOf(req) === 'lut-teal')).toBe(true);
    expect(recorded.saves.length).toBe(1);

    await page.getByRole('combobox', { name: 'Select LUT…' }).click();
    const applied = list.getByRole('option', { name: /Teal Orange/ });
    await applied.hover();
    await expect(pill(page)).toBeHidden();
    await expect(applied.getByText('previewing', { exact: true })).toHaveCount(0);
  });
});

test.describe('LUT hold preview on touch', { tag: ['@touch'] }, () => {
  async function openSheet(page: Page, recorded: ReturnType<typeof record>): Promise<Locator> {
    await mockLibrary(page, recorded.opts);
    await page.goto(`/assets/${ASSET_ID}`);
    await expect(page.locator('html')).toHaveAttribute('data-layout', 'touch');
    await expect(page.getByRole('slider', { name: 'Exposure' })).toBeVisible();
    await page
      .getByRole('tablist', { name: 'Develop groups' })
      .getByRole('tab', { name: 'Profile' })
      .tap();
    await page
      .getByRole('tablist', { name: 'Profile views' })
      .getByRole('tab', { name: 'LUT', exact: true })
      .tap();
    await page.getByRole('button', { name: 'Select LUT…' }).tap();
    const sheet = page.locator('[data-searchable-select-sheet]');
    await expect(sheet.getByRole('option')).toHaveCount(LUTS.length);
    return sheet;
  }

  test('holding a LUT previews it, releasing restores, and a tap applies it', async ({ page }) => {
    const recorded = record();
    const sheet = await openSheet(page, recorded);
    const teal = sheet.getByRole('option', { name: /Teal Orange/ });

    await teal.scrollIntoViewIfNeeded();
    const box = await teal.boundingBox();
    if (!box) throw new Error('no row');
    await touchDown(page, { x: box.x + box.width / 2, y: box.y + box.height / 2 });
    try {
      await expect(pill(page)).toContainText('Previewing LUT · Teal Orange');
      await expect(pill(page)).toContainText('Release to return');
      expect(selections(recorded.saves)).toBe(0);
    } finally {
      await touchUp(page);
    }
    await expect(pill(page)).toBeHidden();
    await expect(sheet).toBeVisible();
    await expect.poll(() => recorded.previews.some((req) => lutOf(req) === 'lut-teal')).toBe(true);
    await expect.poll(() => lastLut(recorded.previews)).toBeNull();
    expect(selections(recorded.saves)).toBe(0);

    await tapCentre(page, teal);
    await expect(sheet).toBeHidden();
    await expect.poll(() => selections(recorded.saves)).toBe(1);
    await expect(pill(page)).toBeHidden();
  });
});
