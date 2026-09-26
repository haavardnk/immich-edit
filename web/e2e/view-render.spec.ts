import { expect, test, type Page } from '@playwright/test';
import { gotoAsset, installMocks, makePng, type PreviewRequest } from './helpers';

const SOURCE_W = 12000;
const SOURCE_H = 8000;

function renderFor(req: PreviewRequest): Buffer {
  const roi = req.roi ?? [0, 0, 1, 1];
  const w = SOURCE_W * roi[2];
  const h = SOURCE_H * roi[3];
  const scale = Math.min(1, req.max_edge / Math.max(w, h));
  return makePng(Math.max(1, Math.round(w * scale)), Math.max(1, Math.round(h * scale)));
}

async function setZoom(page: Page, clicks: number): Promise<void> {
  const zoom = page.getByRole('button', { name: 'Zoom', exact: true });
  await zoom.click();
  const zoomIn = page.getByRole('button', { name: 'Zoom In', exact: true });
  for (let i = 0; i < clicks; i++) await zoomIn.click();
  await zoom.click();
}

async function deviceGeometry(page: Page) {
  return page.getByTestId('view-render').evaluate((el) => {
    const img = el as HTMLImageElement;
    const rect = img.getBoundingClientRect();
    return {
      dpr: window.devicePixelRatio,
      left: rect.left,
      top: rect.top,
      width: rect.width,
      height: rect.height,
      naturalWidth: img.naturalWidth,
      naturalHeight: img.naturalHeight
    };
  });
}

async function expectDeviceExact(page: Page): Promise<void> {
  await expect
    .poll(async () => {
      const g = await deviceGeometry(page);
      return `natural ${g.naturalWidth}x${g.naturalHeight} box ${Math.round(
        g.width * g.dpr
      )}x${Math.round(g.height * g.dpr)}`;
    })
    .toMatch(/^natural (\d+)x(\d+) box \1x\2$/);
}

test.use({ viewport: { width: 1280, height: 900 }, deviceScaleFactor: 2 });

test.describe('view rendering', () => {
  test('paints one device pixel per source pixel at every zoom', async ({ page }) => {
    const requests: PreviewRequest[] = [];
    await installMocks(page, {
      previewRender: renderFor,
      sourceSize: { w: SOURCE_W, h: SOURCE_H },
      onPreview: (req) => requests.push(req)
    });
    await gotoAsset(page);
    await expect(page.getByRole('img', { name: 'IMG_0001.ARW' })).toBeVisible();

    await expect(page.getByTestId('view-render')).toBeVisible();
    await expectDeviceExact(page);
    const fit = await deviceGeometry(page);

    await setZoom(page, 4);
    await expectDeviceExact(page);

    const zoomed = await deviceGeometry(page);
    expect(zoomed.naturalWidth).toBeGreaterThan(fit.naturalWidth);

    const roiRequest = requests.filter((r) => r.lane === 'roi').at(-1);
    if (!roiRequest) throw new Error('no view request was sent');
    const roi = roiRequest.roi;
    if (!roi) throw new Error('zoomed view request had no roi');
    expect(roi[2]).toBeGreaterThan(0);
    expect(roi[2]).toBeLessThan(1);
    expect(roi[0] + roi[2]).toBeLessThanOrEqual(1);
  });

  test('renders the full frame without a roi when fitted', async ({ page }) => {
    const requests: PreviewRequest[] = [];
    await installMocks(page, {
      previewRender: renderFor,
      sourceSize: { w: SOURCE_W, h: SOURCE_H },
      onPreview: (req) => requests.push(req)
    });
    await gotoAsset(page);
    await expect(page.getByTestId('view-render')).toBeVisible();

    const viewRequest = requests.filter((r) => r.lane === 'roi').at(-1);
    if (!viewRequest) throw new Error('no view request was sent');
    expect(viewRequest.roi ?? null).toBeNull();
    expect(await page.getByTestId('view-render').count()).toBe(1);
  });

  test('keeps a single painted image after returning to fit', async ({ page }) => {
    await installMocks(page, {
      previewRender: renderFor,
      sourceSize: { w: SOURCE_W, h: SOURCE_H }
    });
    await gotoAsset(page);
    await expect(page.getByTestId('view-render')).toBeVisible();

    await setZoom(page, 4);
    await expect(page.getByTestId('view-render')).toBeVisible();

    await page.getByRole('button', { name: 'Zoom', exact: true }).click();
    await page.getByRole('button', { name: /Fit to screen/ }).click();

    await expect(page.getByTestId('view-render')).toHaveCount(1);
    await expectDeviceExact(page);
  });

  test('zoom returns to the level the photographer last picked', async ({ page }) => {
    await installMocks(page, {
      previewRender: renderFor,
      sourceSize: { w: SOURCE_W, h: SOURCE_H }
    });
    await gotoAsset(page);
    await expect(page.getByTestId('view-render')).toBeVisible();

    const zoom = page.getByRole('button', { name: 'Zoom', exact: true });
    await expect(zoom).toHaveText('Fit');

    await zoom.click();
    await page.getByRole('button', { name: '1:1', exact: true }).click();
    await page.getByRole('button', { name: 'Zoom In', exact: true }).click();
    await expect(zoom).toHaveText('200%');
    await page.getByRole('button', { name: /Fit to screen/ }).click();
    await expect(zoom).toHaveText('Fit');

    await page.keyboard.press('z');
    await expect(zoom).toHaveText('200%');

    await gotoAsset(page);
    await expect(page.getByTestId('view-render')).toBeVisible();
    await expect(zoom).toHaveText('Fit');

    await page.keyboard.press('z');
    await expect(zoom).toHaveText('200%');
  });

  test('shift and an arrow nudge the zoomed view but not a focused split handle', async ({
    page
  }) => {
    await installMocks(page, {
      previewRender: renderFor,
      sourceSize: { w: SOURCE_W, h: SOURCE_H }
    });
    await gotoAsset(page);
    await expect(page.getByTestId('view-render')).toBeVisible();
    await setZoom(page, 4);

    const image = page.getByRole('img', { name: 'IMG_0001.ARW' }).first();
    const left = async (): Promise<number> => (await image.boundingBox())?.x ?? Number.NaN;
    const start = await left();
    await page.keyboard.press('Shift+ArrowRight');
    await expect.poll(left).toBeLessThan(start);
    const moved = await left();
    await page.keyboard.press('Shift+ArrowLeft');
    await expect.poll(left).toBeGreaterThan(moved);
    await expect(page).toHaveURL(/\/assets\//);

    await page.keyboard.press('y');
    const split = page.getByRole('slider', { name: 'Before/after split' });
    await split.focus();
    const before = await left();
    await page.keyboard.press('Shift+ArrowRight');
    await expect(split).toHaveAttribute('aria-valuenow', '60');
    expect(await left()).toBe(before);
  });

  test('a held section bypass is not replaced by the view render', async ({ page }) => {
    const requests: PreviewRequest[] = [];
    await installMocks(page, {
      previewRender: renderFor,
      sourceSize: { w: SOURCE_W, h: SOURCE_H },
      onPreview: (req) => requests.push(req)
    });
    await gotoAsset(page);
    await expect(page.getByTestId('view-render')).toBeVisible();

    const exposure = (req: PreviewRequest): number =>
      (req.edits as { basic: { exposure_ev: number } }).basic.exposure_ev;
    await page
      .locator('div.group', { has: page.getByRole('button', { name: 'Exposure', exact: true }) })
      .getByRole('slider')
      .fill('1');
    await expect.poll(() => requests.some((r) => r.lane === 'roi' && exposure(r) === 1)).toBe(true);

    await page.getByRole('button', { name: 'Bypass Tone' }).hover();
    const held = requests.length;
    await page.mouse.down();
    await expect
      .poll(() => requests.slice(held).some((r) => r.lane === 'base' && exposure(r) === 0))
      .toBe(true);
    await page.waitForTimeout(1000);

    expect(requests.slice(held).filter((r) => exposure(r) !== 0)).toEqual([]);
    await expect(page.getByTestId('view-render')).toHaveCount(0);

    await page.mouse.up();
    await expect
      .poll(() => requests.slice(held).some((r) => r.lane === 'roi' && exposure(r) === 1))
      .toBe(true);
  });

  test('a paused alt-drag preview is not replaced by the view render', async ({ page }) => {
    const requests: PreviewRequest[] = [];
    await installMocks(page, {
      previewRender: renderFor,
      sourceSize: { w: SOURCE_W, h: SOURCE_H },
      onPreview: (req) => requests.push(req)
    });
    await gotoAsset(page);
    await expect(page.getByTestId('view-render')).toBeVisible();

    await page.getByRole('button', { name: 'Detail', exact: true }).click();
    const slider = page
      .locator('div.group', { has: page.getByRole('button', { name: 'Masking', exact: true }) })
      .getByRole('slider');
    await slider.scrollIntoViewIfNeeded();
    const box = await slider.boundingBox();
    if (!box) throw new Error('masking slider has no box');
    const y = box.y + box.height / 2;
    await page.mouse.move(box.x + 4, y);
    await page.keyboard.down('Alt');
    const held = requests.length;
    await page.mouse.down();
    await page.mouse.move(box.x + box.width / 2, y, { steps: 8 });
    await expect
      .poll(() => requests.slice(held).some((r) => r.preview_mode === 'sharpen_mask'))
      .toBe(true);
    await page.waitForTimeout(1000);

    expect(requests.slice(held).filter((r) => r.preview_mode !== 'sharpen_mask')).toEqual([]);
    await expect(page.getByTestId('view-render')).toHaveCount(0);

    await page.mouse.up();
    await page.keyboard.up('Alt');
    await expect
      .poll(() => requests.slice(held).some((r) => r.lane === 'roi' && r.preview_mode === 'none'))
      .toBe(true);
  });

  test('a slider drag keeps one 1x render in flight and settles at full DPR', async ({ page }) => {
    const requests: PreviewRequest[] = [];
    await installMocks(page, {
      previewRender: renderFor,
      sourceSize: { w: SOURCE_W, h: SOURCE_H },
      onPreview: (req) => requests.push(req)
    });
    await page.route('**/api/assets/*/preview', async (route) => {
      if (route.request().method() !== 'POST') return route.fallback();
      await new Promise((done) => setTimeout(done, 120));
      await route.fallback().catch(() => undefined);
    });
    await page.addInitScript(() => {
      const renders = { open: 0, peak: 0 };
      Object.assign(window, { renders });
      const send = window.fetch.bind(window);
      window.fetch = (input, init) => {
        const target = input instanceof Request ? input.url : String(input);
        const post = (init?.method ?? (input instanceof Request ? input.method : 'GET')) === 'POST';
        if (!post || !new URL(target, location.href).pathname.endsWith('/preview')) {
          return send(input, init);
        }
        renders.open += 1;
        renders.peak = Math.max(renders.peak, renders.open);
        return send(input, init).finally(() => {
          renders.open -= 1;
        });
      };
    });
    const renderCount = (): Promise<{ open: number; peak: number }> =>
      page.evaluate(
        () => (window as unknown as { renders: { open: number; peak: number } }).renders
      );

    await gotoAsset(page);
    await expect(page.getByTestId('view-render')).toBeVisible();
    await expect.poll(async () => (await renderCount()).open).toBe(0);
    await page.evaluate(() => {
      (window as unknown as { renders: { peak: number } }).renders.peak = 0;
    });

    const slider = page
      .locator('div.group', { has: page.getByRole('button', { name: 'Exposure', exact: true }) })
      .getByRole('slider');
    const box = await slider.boundingBox();
    if (!box) throw new Error('exposure slider has no box');
    const y = box.y + box.height / 2;
    await page.mouse.move(box.x + box.width / 2, y);
    await page.mouse.down();
    const dragStart = requests.length;
    for (let step = 1; step <= 6; step++) {
      await page.mouse.move(box.x + box.width / 2 + step * 6, y, { steps: 2 });
      await page.waitForTimeout(250);
    }
    const peakDuringDrag = (await renderCount()).peak;
    const dragEnd = requests.length;
    await page.mouse.up();

    const during = requests.slice(dragStart, dragEnd);
    const stageLong = await page
      .locator('.editor-stage')
      .evaluate((el) => Math.max(el.clientWidth, el.clientHeight));
    const dragEdge = Math.min(1600, Math.round(stageLong));
    expect(during.length).toBeGreaterThan(0);
    expect(during.filter((r) => r.lane !== 'base' || r.max_edge !== dragEdge)).toEqual([]);
    expect(peakDuringDrag).toBe(1);

    await expect.poll(() => requests.slice(dragEnd).some((r) => r.lane === 'roi')).toBe(true);
    const after = requests.slice(dragEnd);
    const settled = after.findLast((r) => r.lane === 'base');
    const view = after.find((r) => r.lane === 'roi');
    if (!settled || !view) throw new Error('release did not render both lanes');
    expect(settled.max_edge).toBe(view.max_edge);
    expect(settled.max_edge).toBeGreaterThan(dragEdge);
    expect(after.indexOf(settled)).toBeLessThan(after.indexOf(view));
    await expectDeviceExact(page);
  });
});
