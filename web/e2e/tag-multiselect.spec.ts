import { expect, test } from '@playwright/test';
import { ASSET_ID, ASSET_SUMMARY, installMocks, json } from './helpers';

const ASSETS = [
  ASSET_SUMMARY,
  {
    ...ASSET_SUMMARY,
    id: '00000000-0000-0000-0000-000000000002',
    originalFileName: 'IMG_0002.ARW'
  }
];

const TAGS = [
  { id: 'tag-1', value: 'Landscape' },
  { id: 'tag-2', value: 'Portrait' },
  { id: 'tag-3', value: 'Landmark' }
];

test('the tag multi-select filters, picks and clears its query', async ({ page }) => {
  await installMocks(page, { assets: ASSETS });
  await page.route('**/api/tags', (route) => route.fulfill(json(TAGS)));

  await page.goto('/search?q=IMG');
  await expect(page.locator(`a[href^="/assets/${ASSET_ID}?"]`)).toBeVisible();
  await page.locator('div[title="IMG_0001.ARW"]').getByLabel('Select', { exact: true }).click();
  await page.getByRole('button', { name: 'Tags', exact: true }).click();

  const search = page.getByLabel('Choose tags…');
  await expect(search).toHaveCSS('height', '28px');
  const add = page.getByRole('button', { name: 'Add to selected' });
  const removeSelected = page.getByRole('button', { name: 'Remove from selected' });
  const controlHeights = await Promise.all(
    [search, add, removeSelected].map((control) =>
      control.evaluate((element) => Math.round(element.getBoundingClientRect().height))
    )
  );
  expect(controlHeights).toEqual([28, 28, 28]);
  await search.fill('land');

  const listbox = page.getByRole('listbox');
  const menuBackground = await listbox.evaluate(
    (element) => getComputedStyle(element).backgroundColor
  );
  const neutralBackground = await page.evaluate(() => {
    const probe = document.createElement('span');
    probe.style.color = 'var(--color-neutral-800)';
    document.body.append(probe);
    const color = getComputedStyle(probe).color;
    probe.remove();
    return color;
  });
  expect(menuBackground).toBe(neutralBackground);
  await expect(listbox).toHaveAttribute('data-side', 'top');
  const inputRadii = await search.locator('..').evaluate((element) => {
    const style = getComputedStyle(element);
    return {
      topLeft: Number.parseFloat(style.borderTopLeftRadius),
      topRight: Number.parseFloat(style.borderTopRightRadius),
      bottomLeft: Number.parseFloat(style.borderBottomLeftRadius),
      bottomRight: Number.parseFloat(style.borderBottomRightRadius)
    };
  });
  expect(inputRadii.topLeft).toBe(0);
  expect(inputRadii.topRight).toBe(0);
  expect(inputRadii.bottomLeft).toBeGreaterThan(0);
  expect(inputRadii.bottomRight).toBeGreaterThan(0);

  const landscape = page.getByRole('option', { name: 'Landscape' });
  await expect(landscape).toBeVisible();
  await expect(landscape).toHaveAttribute('data-highlighted');
  const highlightedBackground = await landscape.evaluate(
    (element) => getComputedStyle(element).backgroundColor
  );
  const neutralHighlight = await page.evaluate(() => {
    const probe = document.createElement('span');
    probe.className = 'bg-white/6';
    document.body.append(probe);
    const color = getComputedStyle(probe).backgroundColor;
    probe.remove();
    return color;
  });
  expect(highlightedBackground).toBe(neutralHighlight);
  await expect(page.getByRole('option', { name: 'Landmark' })).toBeVisible();
  await expect(page.getByRole('option', { name: 'Portrait' })).toBeHidden();

  await page.getByRole('option', { name: 'Landscape' }).click();

  await expect(search).toHaveValue('');
  await expect(page.getByRole('option')).toHaveCount(0);
  await expect(search).toBeFocused();
  await search.click();
  await expect(listbox).toBeVisible();
  const remove = page.getByRole('button', { name: 'Remove Landscape' });
  await expect(remove).toBeVisible();
  await expect(remove).toHaveCSS('height', '24px');

  await search.pressSequentially('land');
  await expect(page.getByRole('option', { name: 'Landmark' })).toBeVisible();
  await expect(page.getByRole('option', { name: 'Landscape' })).toHaveCount(0);

  await page.keyboard.press('Escape');
  await remove.click();
  await expect(page.getByRole('button', { name: 'Remove Landscape' })).toHaveCount(0);
});
