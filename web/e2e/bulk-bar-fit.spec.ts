import { expect, test, type Page } from '@playwright/test';
import { installMocks, openFilters } from './helpers';
import { datedAssets, dayShots, smallPages } from './timeline';

const ASSETS = datedAssets(
  ['31', '30', '29', '28', '27', '26'].flatMap((day) => dayShots(`2026-10-${day}`, 20, 'Bergen'))
);

const ALWAYS = [
  /^Select all$/,
  /^Compare selected$/,
  /^Survey selected$/,
  /^(Un)?[Ff]avorite/,
  /^(Un)?[Rr]eject/,
  /^Colour label$/,
  /^Move to Immich trash$/,
  /^Clear selection$/
];

const LAYOUTS = [
  { width: 1440, filters: false, roomy: true },
  { width: 1280, filters: false, roomy: true },
  { width: 1024, filters: false, roomy: false },
  { width: 1024, filters: true, roomy: false }
] as const;

test.use({ timezoneId: 'UTC' });

type Held = { on: boolean; release: Array<() => void> };

async function openPhotos(page: Page, width: number, filters: boolean): Promise<Held> {
  const held: Held = { on: false, release: [] };
  await page.setViewportSize({ width, height: 900 });
  await page.clock.setFixedTime(new Date('2026-11-05T12:00:00Z'));
  await page.addInitScript(() => {
    localStorage.setItem('immich-edit:settings', JSON.stringify({ metadataPushConsented: true }));
  });
  await installMocks(page, {
    assets: ASSETS,
    total: 50401,
    onMetadata: async () => {
      if (held.on) await new Promise<void>((resolve) => held.release.push(resolve));
    }
  });
  await smallPages(page, 30);
  await page.goto('/photos');
  await expect(page.getByTestId('day-header').first()).toBeVisible();
  if (filters) await openFilters(page);
  return held;
}

function selectionBar(page: Page) {
  return page.getByRole('toolbar', { name: 'Selection actions' });
}

async function pick(page: Page, name: string): Promise<void> {
  const tile = page.locator(`[data-name="${name}"]`);
  await tile.hover();
  await tile.getByRole('button', { name: 'Select', exact: true }).click();
}

async function misfits(page: Page): Promise<string[]> {
  return selectionBar(page).evaluate((bar) => {
    const box = bar.getBoundingClientRect();
    const column = bar.closest('[data-testid="grid-column"]')?.getBoundingClientRect();
    const out: string[] = [];
    if (!column || box.left < column.left || box.right > column.right) out.push('bar');
    for (const node of bar.querySelectorAll<HTMLElement>('button, a[href]')) {
      if (!node.checkVisibility()) continue;
      const r = node.getBoundingClientRect();
      const inside =
        r.left >= box.left - 0.5 &&
        r.right <= box.right + 0.5 &&
        r.top >= box.top - 0.5 &&
        r.bottom <= box.bottom + 0.5;
      if (r.width < 16 || r.height < 16 || !inside)
        out.push(node.getAttribute('aria-label') ?? node.textContent?.trim() ?? '?');
    }
    return out;
  });
}

async function expectBarFits(page: Page, roomy: boolean): Promise<void> {
  const bar = selectionBar(page);
  for (const name of ALWAYS) await expect(bar.getByRole('button', { name })).toBeVisible();
  await expect(bar.getByText(/^\d+ selected$/)).toBeVisible();
  if (roomy) {
    await expect(bar.getByRole('radio', { name: '3 stars' })).toBeVisible();
    await expect(bar.getByRole('button', { name: 'Tags', exact: true })).toBeVisible();
    await expect(bar.getByRole('button', { name: 'Rate', exact: true })).toBeHidden();
  } else {
    await expect(bar.getByRole('button', { name: 'Rate', exact: true })).toBeVisible();
    await expect(bar.getByRole('button', { name: 'More actions' })).toBeVisible();
    await expect(bar.getByRole('button', { name: 'Tags', exact: true })).toBeHidden();
  }
  await expect.poll(() => misfits(page)).toEqual([]);
  await expect
    .poll(async () => {
      const compare = await bar.getByRole('button', { name: 'Compare selected' }).boundingBox();
      const trash = await bar.getByRole('button', { name: 'Move to Immich trash' }).boundingBox();
      return Math.abs((compare?.y ?? 0) - (trash?.y ?? -100));
    })
    .toBeLessThan(2);
}

for (const { width, filters, roomy } of LAYOUTS) {
  const where = `${width}${filters ? ' with the filter panel open' : ''}`;

  test(`every bar control stays visible inside the bar at ${where}`, async ({ page }) => {
    const held = await openPhotos(page, width, filters);
    await pick(page, 'IMG_0001.ARW');
    await pick(page, 'IMG_0002.ARW');
    const bar = selectionBar(page);
    await expect(bar.getByText('2 selected')).toBeVisible();
    await expect(bar).toContainText(/\d+ of 50401 loaded/);
    await expectBarFits(page, roomy);

    held.on = true;
    await bar.getByRole('button', { name: /^Select all$/ }).click();
    await expect.poll(() => held.release.length).toBeGreaterThan(0);
    await expect(bar.getByRole('button', { name: 'Clear selection' })).toBeDisabled();
    await expectBarFits(page, roomy);
    held.on = false;
    for (const release of held.release) release();
  });
}

test('a narrow column folds the stars and the organise group into menus', async ({ page }) => {
  await openPhotos(page, 1024, true);
  await pick(page, 'IMG_0001.ARW');
  await pick(page, 'IMG_0002.ARW');
  const bar = selectionBar(page);

  await bar.getByRole('button', { name: 'Rate', exact: true }).click();
  await page.locator('[data-popover-content]').getByRole('radio', { name: '4 stars' }).click();
  await expect(page.getByText('Updated 2 assets')).toBeVisible();
  await expect(page.locator('[data-popover-content]')).toHaveCount(0);

  await bar.getByRole('button', { name: 'More actions' }).click();
  const menu = page.locator('[data-popover-content]');
  for (const name of ['Create virtual copy', 'Edit and export selected', 'Tags']) {
    await expect(menu.getByRole('button', { name, exact: true })).toBeVisible();
  }
  await menu.getByRole('button', { name: 'Albums', exact: true }).click();
  await expect(menu).toHaveCount(0);
  await expect(bar.getByRole('button', { name: 'More actions' })).toHaveAttribute(
    'aria-expanded',
    'false'
  );
  await expect(bar.getByPlaceholder('Choose albums…')).toBeVisible();
});
