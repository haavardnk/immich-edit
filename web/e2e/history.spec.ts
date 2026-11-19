import { expect, test, type Locator, type Page } from '@playwright/test';
import { neutralEdits, type Edits } from '../src/lib/types/edits';
import { ASSET_ID, gotoAsset, installMocks } from './helpers';
import { mockHistory, type HistoryMock, type HistorySeed } from './historyMocks';
import { mockLibrary } from './touchShell';

function seeds(): HistorySeed[] {
  const steps: Array<(edits: Edits) => void> = [
    (e) => (e.basic.exposure_ev = 0.35),
    (e) => (e.basic.contrast = 12),
    (e) => (e.tone.highlights = -38),
    (e) => (e.tone.shadows = 32),
    (e) => (e.basic.clarity = 20),
    (e) => (e.effects.vignette_amount = -18)
  ];
  const edits = neutralEdits();
  return [
    { edits: null, minutesAgo: 30 },
    ...steps.map((change, index) => {
      change(edits);
      return {
        edits: structuredClone(edits),
        minutesAgo: 20 - index,
        undone: index >= 4
      };
    })
  ];
}

function step(scope: Locator | Page, id: number): Locator {
  return scope.locator(`li[data-history-step="${id}"]`);
}

function jump(scope: Locator | Page, id: number): Locator {
  return step(scope, id).getByRole('button', { name: /^Jump/ });
}

function pill(page: Page): Locator {
  return page.locator('[data-photo-pill]');
}

function toast(page: Page, message: string | RegExp): Locator {
  return page.getByRole('status').filter({ hasText: message });
}

async function expectCurrent(scope: Locator | Page, id: number): Promise<void> {
  await expect(jump(scope, id)).toHaveAttribute('aria-current', 'step');
}

async function expectUndone(scope: Locator | Page, ids: number[], kept: number[]): Promise<void> {
  for (const id of ids) await expect(step(scope, id)).toHaveAttribute('data-undone', 'true');
  for (const id of kept) await expect(step(scope, id)).not.toHaveAttribute('data-undone');
}

test.describe('edit history on desktop', () => {
  let mock: HistoryMock;
  let puts: number;

  test.beforeEach(async ({ page }) => {
    puts = 0;
    await installMocks(page);
    mock = await mockHistory(page, seeds());
    page.on('request', (request) => {
      if (request.method() === 'PUT' && /\/edits$/.test(new URL(request.url()).pathname)) puts++;
    });
    await gotoAsset(page);
  });

  async function openHistory(page: Page): Promise<Locator> {
    await page.getByRole('button', { name: 'Edit history' }).click();
    const popover = page.locator('[data-popover-content]');
    await expect(jump(popover, 7)).toBeVisible();
    return popover;
  }

  test('hovering an older step previews it on the photo', async ({ page }) => {
    const popover = await openHistory(page);
    await expectCurrent(popover, 5);
    await expectUndone(popover, [7, 6], [5, 4]);
    await expect(step(popover, 7).locator('.line-through')).toHaveCSS(
      'text-decoration-line',
      'line-through'
    );

    await jump(popover, 4).hover();
    await expect(pill(page)).toHaveText(
      /^Previewing Highlights\s.*38 · move off the list to return$/
    );
    await expect(step(popover, 4)).toContainText('previewing');
    for (const id of [7, 6, 5]) {
      await expect(step(popover, id)).toHaveAttribute('data-superseded', 'true');
    }
    await expect(step(popover, 3)).not.toHaveAttribute('data-superseded');
    await expect(popover).toContainText('Newer steps are kept until your next edit.');

    await page.getByTestId('preview-image').hover();
    await expect(pill(page)).toHaveCount(0);
    await expect(step(popover, 5)).not.toHaveAttribute('data-superseded');
    await expect(popover).not.toContainText('Newer steps are kept');
  });

  test('focusing an older step from the keyboard previews it', async ({ page }) => {
    const popover = await openHistory(page);
    await page.keyboard.press('Shift');
    await jump(popover, 4).focus();
    await expect(pill(page)).toHaveText(/^Previewing Highlights\s.*38/);
    await expect(step(popover, 4)).toContainText('previewing');

    await jump(popover, 4).blur();
    await expect(pill(page)).toHaveCount(0);
    await expect(step(popover, 4)).not.toContainText('previewing');
  });

  test('a jump keeps the list open and undo restores the previous step', async ({ page }) => {
    const popover = await openHistory(page);

    await jump(popover, 4).click();
    await expect.poll(() => mock.restores).toEqual([4]);
    await expect(popover).toBeVisible();
    await expectCurrent(popover, 4);
    await expectUndone(popover, [7, 6, 5], [4]);
    await expect(pill(page)).toHaveCount(0);

    const restored = toast(page, /Restored Highlights/);
    await expect(restored).toBeVisible();
    await restored.getByRole('button', { name: 'Undo' }).click();
    await expect.poll(() => mock.restores).toEqual([4, 5]);
    expect(puts).toBe(0);

    await expect(popover).toBeHidden();
    const reopened = await openHistory(page);
    await expectCurrent(reopened, 5);
    await expectUndone(reopened, [7, 6], [5]);
  });

  test('an undone step jumps forward and the shortcut goes back', async ({ page }) => {
    const popover = await openHistory(page);

    await jump(popover, 7).click();
    await expect.poll(() => mock.restores).toEqual([7]);
    await expectCurrent(popover, 7);
    await expectUndone(popover, [], [7, 6, 5]);

    await page.getByTestId('preview-image').click();
    await expect(popover).toBeHidden();
    await page.keyboard.press('ControlOrMeta+z');
    await expect.poll(() => mock.restores).toEqual([7, 5]);
    await page.keyboard.press('ControlOrMeta+Shift+z');
    await expect.poll(() => mock.restores).toEqual([7, 5, 7]);
    expect(puts).toBe(0);
  });
});

test.describe('edit history on touch', { tag: ['@touch'] }, () => {
  test('More opens the history sheet and a tap jumps without closing it', async ({ page }) => {
    await mockLibrary(page);
    const mock = await mockHistory(page, seeds());
    await page.goto(`/assets/${ASSET_ID}`);
    await expect(page.locator('html')).toHaveAttribute('data-layout', 'touch');

    await page.getByRole('button', { name: 'More editor actions' }).tap();
    const entry = page.getByRole('button', { name: 'History', exact: true });
    await expect(entry).toContainText('7 steps');
    await entry.tap();

    const sheet = page.getByRole('dialog', { name: 'History' });
    await expect(sheet).toContainText('Newest first · tap a step to jump');
    await expectCurrent(sheet, 5);
    await expect(step(sheet, 5)).toContainText('current');
    await expectUndone(sheet, [7, 6], [5]);
    await expect(pill(page)).toHaveText(/^Step 5 of 7 · Shadows \+32$/);

    await jump(sheet, 3).tap();
    await expect.poll(() => mock.restores).toEqual([3]);
    await expect(sheet).toBeVisible();
    await expectCurrent(sheet, 3);
    await expectUndone(sheet, [7, 6, 5, 4], [3]);
    await expect(pill(page)).toHaveText(/^Step 3 of 7 · Contrast \+12$/);

    await sheet.getByRole('button', { name: 'Close' }).tap();
    await expect(sheet).toBeHidden();
    await expect(pill(page)).toHaveCount(0);
  });
});
