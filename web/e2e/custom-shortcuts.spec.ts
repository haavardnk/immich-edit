import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_ID, installMocks, type InstallOpts } from './helpers';

const CLIP = 'Change the shortcut for Toggle the clipping indicators';

function shortcutsSaved(saved: unknown[]): InstallOpts['onPreferences'] {
  return (patch) => {
    if ('shortcuts' in patch) saved.push(patch.shortcuts);
  };
}

async function openEditor(page: Page, opts: InstallOpts = {}): Promise<void> {
  await installMocks(page, opts);
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto(`/assets/${ASSET_ID}`);
  await expect(page.getByRole('slider', { name: 'Exposure' })).toBeVisible();
}

async function openHelp(page: Page): Promise<Locator> {
  await page.keyboard.press('?');
  const dialog = page.getByRole('dialog', { name: 'Keyboard shortcuts' });
  await expect(dialog).toBeVisible();
  return dialog;
}

async function clippingOn(page: Page): Promise<Locator> {
  await page.getByRole('button', { name: 'More editor actions' }).click();
  return page.getByRole('button', { name: 'Clipping overlay' });
}

test.describe('custom shortcuts', () => {
  test('records a key, swaps a conflict and keeps it after a reload', async ({ page }) => {
    const saved: unknown[] = [];
    await openEditor(page, { onPreferences: shortcutsSaved(saved) });
    const dialog = await openHelp(page);
    const frame = dialog.getByRole('button', { name: CLIP });
    await expect(frame).toHaveText('J');

    await frame.click();
    await expect(frame).toHaveText('Press keys…');
    await page.keyboard.press('w');
    const conflict = dialog.getByRole('alert');
    await expect(conflict).toContainText(
      'W already runs Pick white balance from a neutral area (Editor).'
    );
    await expect(conflict).toContainText('Swap gives that action J');
    await conflict.getByRole('button', { name: 'Swap' }).click();

    await expect(conflict).toHaveCount(0);
    await expect(frame).toHaveText('W');
    await expect(dialog).toContainText('Toggle the clipping indicators · default J');
    await expect(
      dialog.getByRole('button', {
        name: 'Change the shortcut for Pick white balance from a neutral area'
      })
    ).toHaveText('J');
    await expect(dialog).toContainText('2 custom shortcuts');
    await expect.poll(() => saved).toEqual([{ clipWarn: ['w'], pickWhiteBalance: ['j'] }]);

    await page.reload();
    await expect(page.getByRole('slider', { name: 'Exposure' })).toBeVisible();
    await page.keyboard.press('w');
    await expect(await clippingOn(page)).toHaveAttribute('aria-pressed', 'true');
  });

  test('Escape cancels, browser chords are refused and fixed keys only cancel', async ({
    page
  }) => {
    const saved: unknown[] = [];
    await openEditor(page, { onPreferences: shortcutsSaved(saved) });
    const dialog = await openHelp(page);
    const frame = dialog.getByRole('button', { name: CLIP });

    await frame.click();
    await page.keyboard.press('Escape');
    await expect(frame).toHaveText('J');
    await expect(dialog).toBeVisible();

    await frame.click();
    await page.keyboard.press('Control+1');
    await expect(dialog.getByRole('status')).toHaveText(
      'The browser keeps that shortcut; try another'
    );
    await expect(frame).toHaveText('Press keys…');

    await page.keyboard.press('ArrowLeft');
    const conflict = dialog.getByRole('alert');
    await expect(conflict).toContainText('already runs Previous / next photo (Loupe).');
    await expect(conflict.getByRole('button', { name: 'Swap' })).toHaveCount(0);
    await conflict.getByRole('button', { name: 'Cancel' }).click();
    await expect(frame).toHaveText('J');
    expect(saved).toEqual([]);
  });

  test('resets one shortcut, then all of them', async ({ page }) => {
    const saved: unknown[] = [];
    await openEditor(page, {
      shortcuts: { autoAdjust: ['Mod+Shift+a'], undo: ['Mod+y'] },
      onPreferences: shortcutsSaved(saved)
    });
    const dialog = await openHelp(page);
    await expect(
      dialog.getByRole('button', { name: 'Change the shortcut for Auto adjust' })
    ).toHaveText('Ctrl+Shift+A');
    await expect(dialog).toContainText('Auto adjust · default Ctrl+U');
    await expect(dialog).toContainText('2 custom shortcuts');

    await dialog.getByRole('button', { name: 'Reset the shortcut for Auto adjust' }).click();
    await expect(
      dialog.getByRole('button', { name: 'Change the shortcut for Auto adjust' })
    ).toHaveText('Ctrl+U');
    await expect.poll(() => saved).toEqual([{ undo: ['Mod+y'] }]);

    await dialog.getByRole('button', { name: 'Reset all' }).click();
    await expect(dialog).toContainText('0 custom shortcuts');
    await expect(dialog.getByRole('button', { name: 'Reset all' })).toBeDisabled();
    await expect.poll(() => saved).toEqual([{ undo: ['Mod+y'] }, {}]);
  });
});
