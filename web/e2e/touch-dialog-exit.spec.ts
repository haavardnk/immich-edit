import { expect, test, type Page } from '@playwright/test';
import { bulkBar } from './browse';
import { datedAssets, dayShots, openTouchTimeline } from './timeline';
import { selectTwo } from './touchShell';

async function openTrashDialog(page: Page) {
  await openTouchTimeline(page, {
    assets: datedAssets(dayShots('2026-10-31', 4)),
    metadataPushConsented: true
  });
  await selectTwo(page);
  await bulkBar(page).getByRole('button', { name: 'More', exact: true }).tap();
  await page
    .getByRole('dialog', { name: 'More actions' })
    .getByRole('button', { name: 'Move to trash', exact: true })
    .tap();
  const dialog = page.getByRole('dialog', { name: 'Move to Immich trash' });
  await expect(dialog).toBeVisible();
  await page.evaluate(() => {
    Reflect.set(window, '__ghosts', 0);
    new MutationObserver((records) => {
      for (const record of records) {
        for (const node of record.addedNodes) {
          if (node instanceof HTMLElement && node.matches('[data-sheet-ghost]')) {
            Reflect.set(window, '__ghosts', (Reflect.get(window, '__ghosts') as number) + 1);
          }
        }
      }
    }).observe(document.body, { childList: true });
  });
  return dialog;
}

const ghosts = (page: Page) => page.evaluate(() => Reflect.get(window, '__ghosts') as number);

test.describe('touch dialogs leave with their sheet', { tag: '@touch' }, () => {
  test('a confirmed dialog slides away instead of vanishing', async ({ page }) => {
    const dialog = await openTrashDialog(page);

    await dialog.getByRole('button', { name: 'Move to trash' }).tap();

    await expect(dialog).toBeHidden();
    expect(await ghosts(page)).toBeGreaterThan(0);
    await expect(page.locator('[data-sheet-ghost]')).toHaveCount(0);
  });

  test('closing from the sheet itself leaves no copy behind', async ({ page }) => {
    const dialog = await openTrashDialog(page);

    await dialog.getByRole('button', { name: 'close', exact: true }).tap();

    await expect(dialog).toBeHidden();
    expect(await ghosts(page)).toBe(0);
  });
});
