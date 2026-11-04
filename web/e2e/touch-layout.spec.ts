import { expect, test, type Locator, type Page } from '@playwright/test';
import { openBrowse } from './browse';

function root(page: Page): Locator {
  return page.locator('html');
}

function layoutGroup(page: Page): Locator {
  return page.getByRole('radiogroup', { name: 'Layout' });
}

test('a desktop browser gets the desktop layout', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await openBrowse(page);

  await expect(root(page)).toHaveAttribute('data-layout', 'desktop');
  await expect(root(page)).not.toHaveAttribute('data-wide');
});

test.describe('on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('auto picks the touch layout and a forced layout applies live', async ({ page }) => {
    await openBrowse(page);
    await expect(root(page)).toHaveAttribute('data-layout', 'touch');
    await expect(root(page)).not.toHaveAttribute('data-wide');

    await page.goto('/settings#layout');
    const group = layoutGroup(page);
    await expect(group.getByRole('radio', { name: 'Auto' })).toBeChecked();
    await expect(page.getByText('Using the touch layout, picked for this screen.')).toBeVisible();

    await group.getByRole('radio', { name: 'Desktop' }).click();
    await expect(root(page)).toHaveAttribute('data-layout', 'desktop');
    await expect(page.getByText('Using the desktop layout.')).toBeVisible();

    await page.reload();
    const reloaded = layoutGroup(page);
    await expect(reloaded).toBeVisible();
    await expect(root(page)).toHaveAttribute('data-layout', 'desktop');
    await expect(reloaded.getByRole('radio', { name: 'Desktop' })).toBeChecked();

    await reloaded.getByRole('radio', { name: 'Auto' }).click();
    await expect(root(page)).toHaveAttribute('data-layout', 'touch');
  });
});

test.describe('on a tablet', { tag: ['@touch', '@tablet'] }, () => {
  test('landscape is wide and portrait is not', async ({ page }) => {
    await openBrowse(page);
    await expect(root(page)).toHaveAttribute('data-layout', 'touch');
    await expect(root(page)).toHaveAttribute('data-wide');

    await page.setViewportSize({ width: 820, height: 1180 });
    await expect(root(page)).not.toHaveAttribute('data-wide');
    await expect(root(page)).toHaveAttribute('data-layout', 'touch');
  });
});

test(
  'the app boots to the touch layout on an iPhone',
  { tag: '@touch-smoke' },
  async ({ page }) => {
    await openBrowse(page);

    await expect(root(page)).toHaveAttribute('data-layout', 'touch');
    await expect(page.locator('meta[name="viewport"]')).toHaveAttribute(
      'content',
      /viewport-fit=cover/
    );
    const [frameHeight, viewportHeight] = await page
      .locator('.app-frame')
      .evaluate((frame) => [frame.getBoundingClientRect().height, window.innerHeight]);
    expect(frameHeight).toBe(viewportHeight);
  }
);
