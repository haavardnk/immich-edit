import { expect, test, type Locator, type Page } from '@playwright/test';
import { seriousViolations } from './axeHelpers';
import { ASSET_ID, numberedAssets } from './helpers';
import { tapCentre } from './touch';
import { avatar, backButton, open, openDrawer, tab } from './touchShell';

async function back(page: Page, to: string | RegExp): Promise<void> {
  await backButton(page).tap();
  await expect(page).toHaveURL(to);
}

async function hitTarget(chip: Locator, dy: number): Promise<boolean> {
  return chip.evaluate((element, offset) => {
    const box = element.getBoundingClientRect();
    const y = offset < 0 ? box.top + offset : box.bottom + offset;
    return document.elementFromPoint(box.left + box.width / 2, y)?.closest('button') === element;
  }, dy);
}

test.describe('touch back navigation on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('Back pops in-app history from pushed pages', async ({ page }) => {
    await open(page, '/library');
    await expect(avatar(page)).toBeVisible();
    await expect(backButton(page)).toHaveCount(0);

    await page.getByRole('link', { name: 'See all people' }).tap();
    await expect(page).toHaveURL('/people');
    await expect(avatar(page)).toHaveCount(0);
    await expect(backButton(page)).toBeVisible();
    await expect.poll(async () => (await backButton(page).boundingBox())?.height).toBe(44);
    await back(page, '/library');

    await page.getByRole('list', { name: 'People' }).getByRole('link', { name: 'Ingrid' }).tap();
    await expect(page).toHaveURL('/people/person-ingrid');
    await back(page, '/library');

    await tab(page, 'Albums').tap();
    await page
      .getByRole('list', { name: 'Albums' })
      .getByRole('link', { name: /^Family/ })
      .tap();
    await expect(page).toHaveURL('/albums/album-family');
    await back(page, '/albums');
    await expect(avatar(page)).toBeVisible();

    const drawer = await openDrawer(page);
    await drawer.getByRole('button', { name: 'Settings', exact: true }).tap();
    await expect(page).toHaveURL('/settings');
    await back(page, '/albums');
  });

  test('Back without history goes up to the logical parent', async ({ page }) => {
    await open(page, '/people/person-ingrid');
    await back(page, '/people');
    await back(page, '/library');
    await expect(avatar(page)).toBeVisible();
  });

  test('Back walks a folder drill-down up one level at a time', async ({ page }) => {
    await open(page, '/folders?path=%2Fphotos%2F2026%2FNorway');
    await expect(page.getByRole('heading', { name: 'Norway', level: 1 })).toBeVisible();

    await back(page, '/folders?path=%2Fphotos%2F2026');
    await back(page, '/folders?path=%2Fphotos');
    await back(page, '/folders');
    await back(page, '/library');

    await page.getByRole('link', { name: /^Folders/ }).tap();
    await page
      .getByRole('list', { name: 'Subfolders' })
      .getByRole('link', { name: /^photos/ })
      .tap();
    await expect(page).toHaveURL('/folders?path=%2Fphotos');
    await page
      .getByRole('navigation', { name: 'Folder path' })
      .getByRole('link', { name: 'Folders' })
      .tap();
    await expect(page).toHaveURL('/folders');
    await back(page, '/folders?path=%2Fphotos');
  });

  test('Back leaves settings for Photos', async ({ page }) => {
    await open(page, '/settings');
    await expect(page.getByRole('heading', { name: 'Settings', level: 1 })).toBeVisible();
    await back(page, '/photos');
    await expect(avatar(page)).toBeVisible();
  });

  test('reopening a tag does not look for a photo from another view', async ({ page }) => {
    const sky = { id: 'tag-sky', name: 'Sky', value: 'Sky' };
    const [first, second, third] = numberedAssets(3);
    await open(page, '/photos', {
      assets: [first, second, { ...third, tags: [sky] }].filter((asset) => asset !== undefined),
      structuredSearch: true
    });
    await tapCentre(page, page.locator(`[data-name="${first?.originalFileName}"]`));
    await page
      .getByRole('navigation', { name: 'Loupe toolbar' })
      .getByRole('button', { name: /^Back/ })
      .tap();

    const tag = page.getByRole('link', { name: 'Sky' });
    const tile = page.locator(`[data-name="${third?.originalFileName}"]`);
    await tab(page, 'Library').tap();
    await page.getByRole('link', { name: /^Tags/ }).first().tap();
    await expect(page).toHaveURL('/tags');
    await tag.tap();
    await expect(tile).toBeVisible();
    await back(page, '/tags');
    await tag.tap();
    await expect(tile).toBeVisible();
    await back(page, '/tags');
    await expect(page.getByText(/not in this view/)).toHaveCount(0);
  });

  test('the editor guard explains itself and goes back', async ({ page }) => {
    await open(page, '/library');
    await page.getByRole('list', { name: 'Recently edited' }).getByRole('link').first().tap();
    await expect(page).toHaveURL(/\/assets\//);
    await expect(page.getByRole('heading', { name: 'Larger screen needed' })).toBeVisible();
    await expect(page.getByText('Editing needs a larger screen for now.')).toBeVisible();
    expect(await seriousViolations(page)).toEqual([]);

    await page.getByRole('button', { name: 'Back', exact: true }).tap();
    await expect(page).toHaveURL('/library');
  });

  test('the editor guard Back returns after an in-app asset change without looping', async ({
    page
  }) => {
    const [first, second] = numberedAssets(2);
    await open(page, '/albums/album-family', { assets: numberedAssets(2) });
    await tapCentre(page, page.locator(`[data-name="${first.originalFileName}"]`));
    await page.getByRole('button', { name: 'Edit', exact: true }).tap();
    await expect(page).toHaveURL(new RegExp(`/assets/${first.id}\\?from=`));
    await expect(page.getByRole('heading', { name: 'Larger screen needed' })).toBeVisible();

    await page.keyboard.press('ArrowRight');
    await expect(page).toHaveURL(new RegExp(`/assets/${second.id}\\?from=`));

    await page.getByRole('button', { name: 'Back', exact: true }).tap();
    await expect(page).toHaveURL('/albums/album-family');
    await back(page, '/albums');
    await expect(avatar(page)).toBeVisible();
  });

  test('the editor guard Back on a deep-linked asset replaces it with the grid', async ({
    page
  }) => {
    const [first] = numberedAssets(1);
    const from = encodeURIComponent('/albums/album-family');
    await open(page, `/assets/${first.id}?from=${from}`, { assets: numberedAssets(2) });
    await expect(page.getByRole('heading', { name: 'Larger screen needed' })).toBeVisible();

    await page.getByRole('button', { name: 'Back', exact: true }).tap();
    await expect(page).toHaveURL('/albums/album-family');
    await back(page, '/albums');
  });

  test('the editor guard Back without a return path goes to photos', async ({ page }) => {
    await open(page, `/assets/${ASSET_ID}`);
    await page.getByRole('button', { name: 'Back', exact: true }).tap();
    await expect(page).toHaveURL('/photos');
  });
});

test.describe('touch search field on a small phone', { tag: ['@touch', '@phone'] }, () => {
  test('the field fits beside Cancel and the mode chips scroll on one row', async ({ page }) => {
    await page.setViewportSize({ width: 360, height: 780 });
    await open(page, '/search');

    const field = page.getByRole('textbox', { name: 'Search your photos' });
    const cancel = page.getByRole('button', { name: 'Cancel', exact: true });
    await expect(field).toHaveAttribute('placeholder', 'Search');
    const [fieldBox, cancelBox] = await Promise.all([field.boundingBox(), cancel.boundingBox()]);
    expect((fieldBox?.x ?? 0) + (fieldBox?.width ?? 0)).toBeLessThanOrEqual(cancelBox?.x ?? 0);
    expect((cancelBox?.x ?? 0) + (cancelBox?.width ?? 0)).toBeLessThanOrEqual(360);
    const options = await page
      .locator('header[data-app-bar]')
      .getByRole('button', { name: 'Select photos' })
      .boundingBox();
    expect((options?.x ?? 0) + (options?.width ?? 0)).toBeLessThanOrEqual(360);
    const room = await field.evaluate((input) => {
      if (!(input instanceof HTMLInputElement)) return -1;
      const style = getComputedStyle(input);
      const context = document.createElement('canvas').getContext('2d');
      if (!context) return -1;
      context.font = style.font;
      const inner =
        input.clientWidth - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight);
      return inner - context.measureText(input.placeholder).width;
    });
    expect(room).toBeGreaterThan(0);

    await field.pressSequentially('harbour');
    await expect(page).toHaveURL(/\/search\?q=harbour/);
    const modes = page.getByTestId('search-modes');
    await expect(modes.getByText('Search by')).toBeHidden();
    const chips = modes.getByRole('button');
    await expect(chips).toHaveText([
      'Context',
      'File name',
      'Description',
      'Full path or folder',
      'OCR'
    ]);
    await expect(chips.first()).toHaveAttribute('aria-pressed', 'true');
    const tops = await chips.evaluateAll((elements) =>
      elements.map((element) => Math.round(element.getBoundingClientRect().top))
    );
    expect(new Set(tops).size).toBe(1);
    const first = chips.first();
    expect(await hitTarget(first, -5)).toBe(true);
    expect(await hitTarget(first, 5)).toBe(true);
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)
    ).toBe(true);

    const last = chips.last();
    await last.scrollIntoViewIfNeeded();
    await last.tap();
    await expect(page).toHaveURL(/mode=ocr/);
    await expect(last).toHaveAttribute('aria-pressed', 'true');
    await expect(first).toHaveAttribute('aria-pressed', 'false');
  });
});
