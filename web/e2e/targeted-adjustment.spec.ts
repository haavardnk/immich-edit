import { expect, test, type Locator, type Page } from '@playwright/test';
import {
  ASSET_ID,
  installMocks,
  makeStripesPng,
  type InstallOpts,
  type PreviewRequest
} from './helpers';
import { tapAt, touchDown, touchMoveTo, touchUp } from './touch';
import { open } from './touchShell';

const HUES = makeStripesPng(64, 48, [
  [230, 40, 40],
  [240, 150, 30],
  [230, 220, 40],
  [40, 200, 60],
  [40, 200, 200],
  [40, 70, 230],
  [150, 40, 230],
  [230, 40, 230]
]);
const TONES = makeStripesPng(48, 48, [
  [20, 20, 20],
  [128, 128, 128],
  [235, 235, 235]
]);

function rendering(image: Buffer, previews: PreviewRequest[] = []): InstallOpts {
  return {
    onPreview: (req) => previews.push(req),
    previewRender: () => image
  };
}

async function openDesktop(page: Page, opts: InstallOpts): Promise<void> {
  await installMocks(page, opts);
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto(`/assets/${ASSET_ID}`);
  await expect(page.getByRole('slider', { name: 'Exposure' })).toBeVisible();
}

async function readySurface(page: Page): Promise<Locator> {
  const surface = page.getByTestId('target-surface');
  await expect(surface).toHaveAttribute('data-target-ready', '');
  return surface;
}

async function column(
  surface: Locator,
  index: number,
  count: number
): Promise<{ x: number; y: number }> {
  const box = await surface.boundingBox();
  if (!box) throw new Error('no surface');
  return { x: box.x + (box.width * (index + 0.5)) / count, y: box.y + box.height * 0.5 };
}

async function dragUp(page: Page, at: { x: number; y: number }, by: number): Promise<void> {
  await page.mouse.move(at.x, at.y);
  await page.mouse.down();
  await page.mouse.move(at.x, at.y - by, { steps: 6 });
  await page.mouse.up();
}

function panel(page: Page, id: string): Locator {
  return page.locator(`[data-panel="${id}"]`);
}

test.describe('targeted adjustment on desktop', () => {
  test('T names the HSL band under the pointer and drags its saturation, hue and luminance', async ({
    page
  }) => {
    const previews: PreviewRequest[] = [];
    await openDesktop(page, rendering(HUES, previews));

    await page.keyboard.press('t');
    const status = page.getByRole('status').filter({ hasText: 'Targeted adjustment: HSL' });
    await expect(status).toBeVisible();
    await expect(page.getByRole('button', { name: 'Adjust HSL on the photo' })).toHaveAttribute(
      'aria-pressed',
      'true'
    );
    const surface = await readySurface(page);
    expect(previews.some((req) => req.lane === 'reference')).toBe(true);

    const orange = await column(surface, 1, 8);
    await page.mouse.move(orange.x, orange.y);
    await expect(page.getByTestId('target-callout')).toContainText('Orange');
    await expect(page.getByTestId('target-callout')).toContainText('drag ↕ Saturation');
    const blue = await column(surface, 5, 8);
    await page.mouse.move(blue.x, blue.y);
    await expect(page.getByTestId('target-callout')).toContainText('Blue');

    await dragUp(page, orange, 75);
    const hsl = panel(page, 'hsl');
    await expect(hsl.getByRole('radio', { name: 'Orange' })).toHaveAttribute(
      'aria-checked',
      'true'
    );
    await expect(hsl.getByRole('button', { name: 'Edit Saturation value' })).toHaveText(/50/);

    await page.keyboard.down('Alt');
    await dragUp(page, orange, 30);
    await page.keyboard.up('Alt');
    await expect(hsl.getByRole('button', { name: 'Edit Hue value' })).toHaveText(/20/);

    await page.keyboard.down('Shift');
    await dragUp(page, orange, -15);
    await page.keyboard.up('Shift');
    await expect(hsl.getByRole('button', { name: 'Edit Luminance value' })).toHaveText(/-10|−10/);
    await expect(hsl.getByRole('button', { name: 'Edit Saturation value' })).toHaveText(/50/);

    await page.keyboard.press('Escape');
    await expect(status).toHaveCount(0);
    await expect(page.getByTestId('target-surface')).toHaveCount(0);
  });

  test('picks and drags a curve point at the tone under the pointer', async ({ page }) => {
    await openDesktop(page, rendering(TONES));
    await page.getByRole('button', { name: 'Adjust Curves on the photo' }).click();
    const surface = await readySurface(page);

    const grey = await column(surface, 1, 3);
    await page.mouse.move(grey.x, grey.y);
    await expect(page.getByTestId('target-callout')).toContainText('In 128');

    await page.mouse.click(grey.x, grey.y);
    const curves = panel(page, 'curves');
    await expect(curves.getByRole('button', { name: 'Edit In value' })).toHaveText('128');
    await expect(curves.getByRole('button', { name: 'Edit Out value' })).toHaveText('128');
    await expect(
      curves.getByRole('application', { name: 'Curves – RGB' }).locator('circle')
    ).toHaveCount(4);

    await dragUp(page, grey, 60);
    await expect(curves.getByRole('button', { name: 'Edit Out value' })).toHaveText('179');
    await expect(
      curves.getByRole('application', { name: 'Curves – RGB' }).locator('circle')
    ).toHaveCount(4);
  });

  test('samples B&W channels from a colour reference', async ({ page }) => {
    const previews: PreviewRequest[] = [];
    await openDesktop(page, rendering(HUES, previews));
    await page.getByRole('button', { name: 'Black & White', exact: true }).click();
    const target = page.getByRole('button', { name: 'Adjust Black & White on the photo' });
    await expect(target).toBeDisabled();
    const convert = page.getByRole('checkbox', { name: 'Convert to Black & White' });
    await convert.click();
    await expect(target).toBeEnabled();

    await target.click();
    const surface = await readySurface(page);
    const reference = previews.filter((req) => req.lane === 'reference').at(-1);
    const color = (reference?.edits as { color?: { bw?: { enabled?: boolean } } }).color;
    expect(color?.bw?.enabled).toBe(false);

    const red = await column(surface, 0, 8);
    await page.mouse.move(red.x, red.y);
    await expect(page.getByTestId('target-callout')).toContainText('Red');
    await dragUp(page, red, 45);
    await expect(panel(page, 'bw').getByRole('button', { name: 'Edit Red value' })).toHaveText(
      /30/
    );

    await convert.click();
    await expect(page.getByTestId('target-surface')).toHaveCount(0);
    await expect(page.getByRole('status').filter({ hasText: 'Targeted adjustment' })).toHaveCount(
      0
    );
    await expect(target).toBeDisabled();
    await expect(target).toHaveAttribute('aria-pressed', 'false');
  });

  test('T falls back to HSL when the last tool is unavailable', async ({ page }) => {
    await openDesktop(page, rendering(HUES));
    await page.getByRole('button', { name: 'Black & White', exact: true }).click();
    const convert = page.getByRole('checkbox', { name: 'Convert to Black & White' });
    await convert.click();
    await page.getByRole('button', { name: 'Adjust Black & White on the photo' }).click();
    await page.keyboard.press('Escape');
    await convert.click();

    await page.keyboard.press('t');
    await expect(
      page.getByRole('status').filter({ hasText: 'Targeted adjustment: HSL' })
    ).toBeVisible();
  });

  test('picks the colour grading region by tone', async ({ page }) => {
    await openDesktop(page, rendering(TONES));
    await page.getByRole('button', { name: 'Adjust Color Grading on the photo' }).click();
    const surface = await readySurface(page);

    const dark = await column(surface, 0, 3);
    await page.mouse.move(dark.x, dark.y);
    await expect(page.getByTestId('target-callout')).toContainText('Shadows');
    const light = await column(surface, 2, 3);
    await page.mouse.move(light.x, light.y);
    await expect(page.getByTestId('target-callout')).toContainText('Highlights');

    await dragUp(page, dark, 60);
    const grading = panel(page, 'color-grading');
    await expect(grading.getByText('Adjust — Shadows')).toBeVisible();
    await expect(grading.getByRole('button', { name: 'Edit Saturation value' })).toHaveText(/20/);
  });
});

test.describe('pick on photo', { tag: '@touch' }, () => {
  async function openView(page: Page, group: string, view: string): Promise<Locator> {
    await open(page, `/assets/${ASSET_ID}`, { previewRender: () => HUES });
    await expect(page.getByRole('slider', { name: 'Exposure' })).toBeVisible();
    await page
      .getByRole('tablist', { name: 'Develop groups' })
      .getByRole('tab', { name: group })
      .tap();
    await page
      .getByRole('tablist', { name: `${group} views` })
      .getByRole('tab', { name: view, exact: true })
      .tap();
    const region = page.getByRole('region', { name: view, exact: true });
    await expect(region).toBeVisible();
    return region;
  }

  test('selects the Mix band under the finger through the magnifier', async ({ page }) => {
    const view = await openView(page, 'Color', 'Mix');
    const pick = view.getByRole('button', { name: 'Pick band on the photo' });
    await pick.tap();
    await expect(pick).toHaveAttribute('aria-pressed', 'true');
    const surface = await readySurface(page);

    const blue = await column(surface, 5, 8);
    await touchDown(page, blue);
    await expect(page.getByTestId('touch-magnifier')).toBeVisible();
    await expect(page.getByTestId('target-callout')).toHaveText('Blue');
    const orange = await column(surface, 1, 8);
    await touchMoveTo(page, orange);
    await expect(page.getByTestId('target-callout')).toHaveText('Orange');
    await touchUp(page);

    await expect(page.getByTestId('target-surface')).toHaveCount(0);
    await expect(pick).toHaveAttribute('aria-pressed', 'false');
    await expect(view.getByRole('radio', { name: 'Orange' })).toHaveAttribute(
      'aria-checked',
      'true'
    );
    await expect(view.getByRole('button', { name: 'Edit Saturation value' })).toHaveText('0');
  });

  test('adds or selects the curve point for the tone under the finger', async ({ page }) => {
    const view = await openView(page, 'Light', 'Curve');
    await view.getByRole('button', { name: 'Pick point on the photo' }).tap();
    const surface = await readySurface(page);
    const box = await surface.boundingBox();
    if (!box) throw new Error('no surface');

    await tapAt(page, box.x + box.width * 0.5, box.y + box.height * 0.5);
    await expect(page.getByTestId('target-surface')).toHaveCount(0);
    await expect(view.getByRole('button', { name: 'Edit In value' })).toBeVisible();
    const plot = view.getByRole('application', { name: 'Curves – RGB' });
    await expect(plot.locator('circle')).toHaveCount(4);
  });
});
