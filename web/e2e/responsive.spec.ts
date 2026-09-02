import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';
import { ASSET_ID, ASSET_SUMMARY, gotoAsset, installMocks, makePng } from './helpers';

const SIZES = [
  { width: 1600, height: 832 },
  { width: 1440, height: 900 },
  { width: 1280, height: 720 },
  { width: 1024, height: 768 }
];

async function sidewaysScrollers(page: Page): Promise<string[]> {
  return page.evaluate(() => {
    const slack = 2;
    const root = document.documentElement;
    const out: string[] = [];
    if (root.scrollWidth > window.innerWidth + slack) out.push('document');
    if (root.scrollHeight > window.innerHeight + slack) out.push('document (vertical)');
    for (const el of Array.from(document.querySelectorAll<HTMLElement>('body *'))) {
      const overflowX = getComputedStyle(el).overflowX;
      if (['auto', 'scroll', 'hidden', 'clip'].includes(overflowX)) continue;
      if (el.scrollWidth <= el.clientWidth + slack) continue;
      out.push(`${el.tagName.toLowerCase()}.${el.className || '(no class)'}`);
    }
    return out;
  });
}

for (const size of SIZES) {
  test(`the photo grid fits ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await installMocks(page);

    await page.goto('/photos');
    await expect(page.getByRole('link').first()).toBeVisible();

    await expect(page.getByRole('link', { name: /^Photos/ })).toBeVisible();
    await expect(page.getByRole('textbox', { name: 'Search your photos' })).toBeVisible();
    expect(await sidewaysScrollers(page)).toEqual([]);
  });

  test(`the editor fits ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await installMocks(page);

    await gotoAsset(page);

    await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();
    await expect(page.getByRole('tab', { name: 'Export', exact: true })).toBeVisible();
    await expect(page.getByRole('slider', { name: 'Exposure' })).toBeVisible();
    await expect(page.getByRole('navigation', { name: 'Editor toolbar' })).toHaveCSS(
      'height',
      '48px'
    );
    await expect(
      page.getByRole('navigation', { name: 'Editor status and view controls' })
    ).toHaveCSS('height', '36px');
    expect(await sidewaysScrollers(page)).toEqual([]);
  });
}

test('the editor resize handles persist once per committed gesture', async ({ page }) => {
  await page.addInitScript(() => {
    const state = window as Window & { editorUiWriteCount: number };
    const setItem = Storage.prototype.setItem;
    state.editorUiWriteCount = 0;
    Storage.prototype.setItem = function (key: string, value: string): void {
      if (key === 'immich-edit:editorUi') state.editorUiWriteCount += 1;
      setItem.call(this, key, value);
    };
  });
  await installMocks(page);
  await page.goto('/photos');
  await page
    .getByRole('link', { name: ASSET_SUMMARY.originalFileName })
    .evaluate((element) => element.click());
  await expect(page.getByText(ASSET_SUMMARY.originalFileName)).toBeVisible();

  const cases = [
    { name: 'Resize editor controls', axis: 'x', key: 'ArrowLeft', keyDelta: 16 },
    { name: 'Resize filmstrip', axis: 'y', key: 'ArrowUp', keyDelta: 8 }
  ] as const;
  for (const resizeCase of cases) {
    const handle = page.getByRole('slider', { name: resizeCase.name });
    const bounds = await handle.boundingBox();
    if (!bounds) throw new Error(`${resizeCase.name} has no bounds`);
    if (resizeCase.name === 'Resize editor controls') {
      expect(bounds.width).toBeGreaterThanOrEqual(8);
      expect(await handle.evaluate((element) => getComputedStyle(element, '::after').width)).toBe(
        '1px'
      );
    }
    const startValue = Number(await handle.getAttribute('aria-valuenow'));
    const centerX = bounds.x + bounds.width / 2;
    const centerY = bounds.y + bounds.height / 2;
    await page.evaluate(() => {
      (window as Window & { editorUiWriteCount: number }).editorUiWriteCount = 0;
    });
    await page.mouse.move(centerX, centerY);
    await page.mouse.down();
    await page.mouse.move(
      resizeCase.axis === 'x' ? centerX - 20 : centerX,
      resizeCase.axis === 'y' ? centerY - 20 : centerY,
      { steps: 5 }
    );
    expect(
      await page.evaluate(
        () => (window as Window & { editorUiWriteCount: number }).editorUiWriteCount
      )
    ).toBe(0);
    await page.mouse.up();
    await expect(handle).toHaveAttribute('aria-valuenow', String(startValue + 20));
    expect(
      await page.evaluate(
        () => (window as Window & { editorUiWriteCount: number }).editorUiWriteCount
      )
    ).toBe(1);

    await page.evaluate(() => {
      (window as Window & { editorUiWriteCount: number }).editorUiWriteCount = 0;
    });
    await handle.press(resizeCase.key);
    await expect(handle).toHaveAttribute(
      'aria-valuenow',
      String(startValue + 20 + resizeCase.keyDelta)
    );
    expect(
      await page.evaluate(
        () => (window as Window & { editorUiWriteCount: number }).editorUiWriteCount
      )
    ).toBe(1);
  }
});

test('the photo grid preserves mixed source orientations', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 720 });
  const fixtures = [
    { id: '00000000-0000-0000-0000-000000000011', name: 'LANDSCAPE.ARW', width: 600, height: 400 },
    { id: '00000000-0000-0000-0000-000000000012', name: 'PORTRAIT.ARW', width: 400, height: 600 },
    {
      id: '00000000-0000-0000-0000-000000000013',
      name: 'ROTATED.ARW',
      width: 600,
      height: 400,
      orientation: '6'
    },
    { id: '00000000-0000-0000-0000-000000000014', name: 'SQUARE.ARW', width: 400, height: 400 },
    { id: '00000000-0000-0000-0000-000000000015', name: 'PANORAMA.ARW', width: 800, height: 300 }
  ];
  const assets = fixtures.map((fixture) => ({
    ...ASSET_SUMMARY,
    id: fixture.id,
    originalFileName: fixture.name,
    exifInfo: {
      exifImageWidth: fixture.width,
      exifImageHeight: fixture.height,
      orientation: fixture.orientation ?? null
    }
  }));
  const thumbnails = new Map(
    fixtures.map((fixture) => [
      fixture.id,
      makePng(
        fixture.orientation ? fixture.height : fixture.width,
        fixture.orientation ? fixture.width : fixture.height
      )
    ])
  );
  await installMocks(page, {
    assets,
    thumbnailRender: (id) => thumbnails.get(id) ?? makePng(1, 1)
  });

  await page.goto('/photos');

  for (const fixture of fixtures) {
    const link = page.getByRole('link', { name: fixture.name });
    const image = link.locator('img');
    await expect(link).toBeVisible();
    await expect(link.locator('..')).toHaveCSS('border-radius', '0px');
    const ratios = await image.evaluate((element) => ({
      rendered: element.clientWidth / element.clientHeight,
      source: element.naturalWidth / element.naturalHeight
    }));
    expect(ratios.rendered).toBeCloseTo(ratios.source, 2);
  }
  expect(await sidewaysScrollers(page)).toEqual([]);
});

test('the editor filmstrip preserves mixed source orientations', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 720 });
  const fixtures = [
    { id: ASSET_ID, name: 'LANDSCAPE.ARW', width: 600, height: 400 },
    { id: '00000000-0000-0000-0000-000000000012', name: 'PORTRAIT.ARW', width: 400, height: 600 },
    {
      id: '00000000-0000-0000-0000-000000000013',
      name: 'ROTATED.ARW',
      width: 600,
      height: 400,
      orientation: '6'
    }
  ];
  const assets = fixtures.map((fixture) => ({
    ...ASSET_SUMMARY,
    id: fixture.id,
    originalFileName: fixture.name,
    exifInfo: {
      exifImageWidth: fixture.width,
      exifImageHeight: fixture.height,
      orientation: fixture.orientation ?? null
    }
  }));
  const thumbnails = new Map(
    fixtures.map((fixture) => [
      fixture.id,
      makePng(
        fixture.orientation ? fixture.height : fixture.width,
        fixture.orientation ? fixture.width : fixture.height
      )
    ])
  );
  await installMocks(page, {
    assets,
    thumbnailRender: (id) => thumbnails.get(id) ?? makePng(1, 1)
  });

  await page.goto('/photos');
  await page.getByRole('link', { name: 'LANDSCAPE.ARW' }).evaluate((element) => element.click());

  for (const fixture of fixtures) {
    const thumbnail = page.getByRole('link', { name: fixture.name });
    const image = thumbnail.locator('img');
    await expect(thumbnail).toBeVisible();
    await expect(image).toHaveJSProperty('complete', true);
    await expect
      .poll(() =>
        image.evaluate((element) => element.naturalWidth > 0 && element.naturalHeight > 0)
      )
      .toBe(true);
    const rendered = await thumbnail.evaluate((element) => {
      const bounds = element.getBoundingClientRect();
      return bounds.width / bounds.height;
    });
    const source = await image.evaluate((element) => element.naturalWidth / element.naturalHeight);
    const ratios = { rendered, source };
    expect(ratios.rendered).toBeCloseTo(ratios.source, 2);
  }
});

test('the mobile browse shell uses a toggleable 256px sidebar', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installMocks(page);

  await page.goto('/photos');

  const sidebar = page.getByRole('complementary');
  const main = page.getByRole('main');
  const visibleMainWidth = (): Promise<number> =>
    main.evaluate((element) => element.parentElement?.clientWidth ?? 0);
  const sidebarWidth = (): Promise<number> =>
    main.evaluate((element) => element.parentElement?.previousElementSibling?.clientWidth ?? 0);
  await expect(page.getByRole('textbox', { name: 'Search your photos' })).toBeVisible();
  await expect(sidebar).toBeHidden();
  await expect.poll(visibleMainWidth).toBeGreaterThan(380);
  await expect.poll(sidebarWidth).toBe(0);
  await expect
    .poll(() => page.evaluate(() => document.elementFromPoint(100, 500)?.closest('main') !== null))
    .toBe(true);

  await page.getByRole('button', { name: 'Library' }).click();
  await expect(sidebar).toBeVisible();
  await expect(sidebar).toHaveCSS('width', '256px');
  await expect.poll(sidebarWidth).toBeGreaterThan(254);
  expect(await sidewaysScrollers(page)).toEqual([]);

  await page.getByRole('button', { name: 'Library' }).click();
  await expect(sidebar).toBeHidden();
  await expect(page.getByRole('button', { name: 'Library' })).toBeFocused();
  await expect.poll(visibleMainWidth).toBeGreaterThan(380);
  await expect.poll(sidebarWidth).toBe(0);
  await expect
    .poll(() => page.evaluate(() => document.elementFromPoint(100, 500)?.closest('main') !== null))
    .toBe(true);

  await page.getByRole('button', { name: 'Library' }).click();
  await page.getByRole('link', { name: /^Favorites/ }).click();
  await expect(page).toHaveURL('/favorites');
  await expect(sidebar).toBeHidden();
});

test('the mobile bulk action bar keeps every action reachable', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installMocks(page);

  await page.goto('/photos');
  await page.getByRole('button', { name: 'Select', exact: true }).click();

  const bulkBar = page.getByRole('navigation').filter({ hasText: '1 selected' });
  const bulkSurfaceFits = (): Promise<boolean> =>
    bulkBar.evaluate((element) => {
      const surface = element.parentElement;
      if (!surface) return false;
      const bounds = surface.getBoundingClientRect();
      return bounds.left >= 0 && bounds.right <= window.innerWidth;
    });
  const clearSelection = page.getByRole('button', { name: 'Clear selection' });
  const tags = page.getByRole('button', { name: 'Tags', exact: true });
  await expect.poll(bulkSurfaceFits).toBe(true);
  await expect(clearSelection).toBeInViewport();
  await tags.scrollIntoViewIfNeeded();
  await expect(tags).toBeInViewport();
  await expect(clearSelection).toBeInViewport();

  await tags.click();
  await expect(page.getByRole('button', { name: 'Add to selected' })).toBeInViewport();
  await expect(page.getByRole('button', { name: 'Remove from selected' })).toBeInViewport();
  await expect(clearSelection).toBeInViewport();
  await expect.poll(bulkSurfaceFits).toBe(true);
});

for (const size of [
  { width: 390, height: 844 },
  { width: 1024, height: 768 }
]) {
  test(`the loupe chrome fits ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await installMocks(page);
    await page.goto('/photos');

    await page.getByLabel('Quick review').click();
    await expect(page.getByRole('button', { name: /^Back/ })).toBeInViewport();
    await expect(page.getByRole('navigation', { name: 'Photo actions' })).toBeInViewport();
    await expect(page.getByRole('button', { name: 'Collapse filmstrip' })).toBeInViewport();

    if (size.width === 390) {
      await page.getByRole('button', { name: 'More loupe actions' }).click();
      const clipping = page.getByRole('button', { name: 'Clipping overlay', exact: true });
      await expect(clipping).toBeInViewport();
      await clipping.click();
      await expect(page.getByRole('button', { name: 'More loupe actions' })).toBeFocused();
    }

    expect(await sidewaysScrollers(page)).toEqual([]);
  });
}

for (const size of [
  { width: 767, height: 800 },
  { width: 390, height: 844 }
]) {
  test(`the editor guard fits ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await installMocks(page);

    await page.goto(`/assets/${ASSET_ID}`);

    await expect(page.getByRole('heading', { name: 'Desktop required' })).toBeVisible();
    await expect(page.getByRole('tab', { name: 'Export', exact: true })).toBeHidden();
    expect(await sidewaysScrollers(page)).toEqual([]);
  });
}

test('the editor fits at the desktop breakpoint', async ({ page }) => {
  await page.setViewportSize({ width: 768, height: 768 });
  await installMocks(page);

  await gotoAsset(page);

  await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();
  await expect(page.getByRole('tab', { name: 'Export', exact: true })).toBeVisible();
  await expect(page.getByRole('slider', { name: 'Exposure' })).toBeVisible();
  await page.getByRole('button', { name: 'More editor actions' }).click();
  for (const name of [
    'Create virtual copy',
    'View original',
    'Before/After split',
    'Clipping overlay'
  ]) {
    await expect(page.getByRole('button', { name, exact: true })).toBeInViewport();
  }
  expect(await sidewaysScrollers(page)).toEqual([]);
});
