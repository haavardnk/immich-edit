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
});
