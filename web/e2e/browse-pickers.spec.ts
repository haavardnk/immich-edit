import { expect, test } from '@playwright/test';
import { installMocks, numberedAssets } from './helpers';

const PAGED_ASSETS = numberedAssets(3);

const CAMERA_ASSETS = [
  ['SONY', 'ILCE-7M3', 'FE 35mm F1.8'],
  ['FUJIFILM', 'X-T5', 'XF35mmF1.4 R'],
  ['SONY', 'ILCE-7M4', 'FE 85mm F1.8']
].map(([make, model, lensModel], index) => ({
  ...PAGED_ASSETS[index],
  exifInfo: { make, model, lensModel }
}));

type ExifFilter = Partial<
  Record<'make' | 'model' | 'lensModel' | 'country' | 'state' | 'city', (string | null)[]>
>;

test('one camera pick narrows the lens list on flat search', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  await installMocks(page, {
    assets: CAMERA_ASSETS,
    onMetadata: (body) => void bodies.push(body)
  });
  await page.goto('/photos');

  await page.getByRole('button', { name: 'Filters' }).click();
  const make = page.getByRole('combobox', { name: 'Make' });
  await make.click();
  await page.getByRole('option', { name: 'SONY' }).click();

  await expect(make).toHaveValue('SONY');
  await expect.poll(() => (bodies.at(-1)?.filter as ExifFilter).make).toEqual(['SONY']);
  await expect(page.locator('div[title="IMG_0002.ARW"]')).toHaveCount(0);
  await page.getByRole('combobox', { name: 'Lens' }).click();
  await expect(page.getByRole('option')).toHaveText(['FE 35mm F1.8', 'FE 85mm F1.8']);
});

test('several cameras match together and a removed make prunes its models', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  const filter = () => bodies.at(-1)?.filter as ExifFilter;
  await installMocks(page, {
    structuredSearch: true,
    assets: CAMERA_ASSETS,
    onMetadata: (body) => void bodies.push(body)
  });
  await page.goto('/photos');

  await page.getByRole('button', { name: 'Filters' }).click();
  const make = page.getByRole('combobox', { name: 'Make' });
  await make.click();
  await page.getByRole('option', { name: 'SONY' }).click();
  await page.getByRole('option', { name: 'FUJIFILM' }).click();
  await expect.poll(() => filter().make).toEqual(['SONY', 'FUJIFILM']);
  await page.keyboard.press('Escape');
  await expect(make).toHaveAttribute('aria-expanded', 'false');

  await page.getByRole('combobox', { name: 'Model' }).click();
  await expect(page.getByRole('option')).toHaveText(['ILCE-7M3', 'ILCE-7M4', 'X-T5']);
  await page.getByRole('option', { name: 'ILCE-7M3' }).click();
  await expect.poll(() => filter().model).toEqual(['ILCE-7M3']);
  await page.keyboard.press('Escape');

  await make.click();
  await page.getByRole('option', { name: 'SONY' }).click();
  await expect.poll(() => filter().make).toEqual(['FUJIFILM']);
  await expect.poll(() => filter().model).toBeUndefined();
  await expect(page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')).toHaveText(
    ['Make: FUJIFILM']
  );
});

test('an unknown lens matches photos without one and excludes named lenses', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  const filter = () => bodies.at(-1)?.filter as ExifFilter;
  const bare = { ...numberedAssets(4)[3], exifInfo: { make: 'SONY', model: 'ILCE-7M3' } };
  await installMocks(page, {
    structuredSearch: true,
    assets: [...CAMERA_ASSETS, bare],
    onMetadata: (body) => void bodies.push(body)
  });
  await page.goto('/photos');

  await page.getByRole('button', { name: 'Filters' }).click();
  const lens = page.getByRole('combobox', { name: 'Lens' });
  await lens.click();
  await expect(page.getByRole('option')).toHaveText([
    'FE 35mm F1.8',
    'FE 85mm F1.8',
    'XF35mmF1.4 R',
    'Unknown'
  ]);
  await page.getByRole('option', { name: 'FE 35mm F1.8' }).click();
  await page.getByRole('option', { name: 'Unknown' }).click();
  await expect.poll(() => filter().lensModel).toEqual([null]);
  await expect(page.locator(`div[title="${bare.originalFileName}"]`)).toHaveCount(1);
  await expect(page.locator('div[title="IMG_0001.ARW"]')).toHaveCount(0);
  await page.getByRole('option', { name: 'FE 85mm F1.8' }).click();
  await expect.poll(() => filter().lensModel).toEqual(['FE 85mm F1.8']);
  await page.keyboard.press('Escape');
  await expect(page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')).toHaveText(
    ['Lens: FE 85mm F1.8']
  );
});

test('a country narrows states and a state narrows cities', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  const filter = () => bodies.at(-1)?.filter as ExifFilter;
  const assets = [
    ['Norway', 'Vestland', 'Bergen'],
    ['Norway', 'Oslo', 'Oslo'],
    ['Japan', 'Tokyo', 'Tokyo']
  ].map(([country, state, city], index) => ({
    ...PAGED_ASSETS[index],
    exifInfo: { country, state, city }
  }));
  await installMocks(page, { assets, onMetadata: (body) => void bodies.push(body) });
  await page.goto('/photos');

  await page.getByRole('button', { name: 'Filters' }).click();
  await page.getByRole('combobox', { name: 'Country' }).click();
  await page.getByRole('option', { name: 'Norway' }).click();
  await expect.poll(() => filter().country).toEqual(['Norway']);
  await expect(page.locator('div[title="IMG_0003.ARW"]')).toHaveCount(0);

  await page.getByRole('combobox', { name: 'State' }).click();
  await expect(page.getByRole('option')).toHaveText(['Oslo', 'Vestland']);
  await page.getByRole('option', { name: 'Vestland' }).click();
  await expect.poll(() => filter().state).toEqual(['Vestland']);

  await page.getByRole('combobox', { name: 'City' }).click();
  await expect(page.getByRole('option')).toHaveText(['Bergen']);
});

test('filter pickers join the list edge it opens on and keep the popover size', async ({
  page
}) => {
  await installMocks(page, { structuredSearch: true, assets: CAMERA_ASSETS });
  await page.goto('/photos');
  await page.getByRole('button', { name: 'Filters' }).click();
  const make = page.getByRole('combobox', { name: 'Make' });
  await make.click();
  await page.getByRole('option', { name: 'SONY' }).click();
  await page.keyboard.press('Escape');
  await expect(make).toHaveAttribute('aria-expanded', 'false');

  await page.evaluate(() => {
    const popover = document.querySelector('[data-popover-content]') as HTMLElement;
    const widths: number[] = [];
    Object.assign(window, { popoverWidths: widths });
    new ResizeObserver(() => widths.push(popover.offsetWidth)).observe(popover);
  });
  const sides: string[] = [];
  for (const name of ['Make', 'City']) {
    const picker = page.getByRole('combobox', { name });
    await picker.click();
    const side = (await page.locator('[data-combobox-content]').getAttribute('data-side')) ?? '';
    sides.push(side);
    const [joined, free] =
      side === 'top'
        ? ['border-top-left-radius', 'border-bottom-left-radius']
        : ['border-bottom-left-radius', 'border-top-left-radius'];
    const radius = (corner: string) =>
      picker.evaluate((el, key) => getComputedStyle(el).getPropertyValue(key), corner);
    await expect.poll(() => radius(joined)).toBe('0px');
    expect(await radius(free)).not.toBe('0px');
    await page.keyboard.press('Escape');
    await expect(picker).toHaveAttribute('aria-expanded', 'false');
  }
  const widths = await page.evaluate(
    () => (window as unknown as { popoverWidths: number[] }).popoverWidths
  );
  expect(new Set(widths).size).toBe(1);
  expect(sides).toEqual(['top', 'bottom']);
});
