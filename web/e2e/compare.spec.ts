import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_ID, gotoAsset, installMocks } from './helpers';
import { mockSnapshots } from './snapshotMocks';
import { mockLibrary } from './touchShell';

const WARM = { name: 'Warm v1', exposure: 0.7, minutesAgo: 125 };

type Placement = { x: number; y: number; w: number; h: number };

function bar(page: Page): Locator {
  return page.getByRole('group', { name: 'Compare' });
}

function chip(page: Page, name: string): Locator {
  return bar(page).getByRole('radio', { name, exact: true });
}

function compareView(page: Page, kind: 'split' | 'side'): Locator {
  return page.locator(`[data-compare="${kind}"]`);
}

function labels(page: Page): Locator {
  return page.locator('[data-compare-label]');
}

function pane(page: Page, side: 'left' | 'right'): Locator {
  return page.locator(`[data-compare-pane="${side}"]`);
}

function compareButton(page: Page): Locator {
  return page.getByRole('button', { name: /^Compare( \(Y\))?$/ });
}

function leftRenders(page: Page): number[] {
  const exposures: number[] = [];
  page.on('request', (request) => {
    if (request.method() !== 'POST' || !/\/api\/assets\/[^/]+\/preview$/.test(request.url())) {
      return;
    }
    const body = request.postDataJSON() as {
      lane: string;
      edits: { basic: { exposure_ev: number } };
    };
    if (body.lane === 'original') exposures.push(body.edits.basic.exposure_ev);
  });
  return exposures;
}

async function placement(image: Locator, frame: Locator): Promise<Placement | null> {
  const [box, outer] = await Promise.all([image.boundingBox(), frame.boundingBox()]);
  if (!box || !outer) return null;
  return {
    x: Math.round(box.x - outer.x),
    y: Math.round(box.y - outer.y),
    w: Math.round(box.width),
    h: Math.round(box.height)
  };
}

async function openDesktop(page: Page): Promise<void> {
  await installMocks(page);
  await mockSnapshots(page, [WARM]);
  await gotoAsset(page);
  await expect(page.getByTestId('preview-image')).toBeVisible();
}

async function openTouch(page: Page): Promise<void> {
  await mockLibrary(page);
  await mockSnapshots(page, [WARM]);
  await page.goto(`/assets/${ASSET_ID}`);
  await expect(page.locator('html')).toHaveAttribute('data-layout', 'touch');
  await expect(page.getByTestId('preview-image')).toBeVisible();
}

test.describe('compare on desktop', () => {
  test('y cycles the split, side by side and off', async ({ page }) => {
    await openDesktop(page);

    await page.keyboard.press('y');
    await expect(compareView(page, 'split')).toBeVisible();
    await expect(chip(page, 'Split')).toBeChecked();
    await expect(labels(page)).toHaveText(['Original', 'Edited']);
    await expect(bar(page).getByRole('button', { name: 'Left: Original' })).toBeVisible();

    await page.keyboard.press('y');
    await expect(compareView(page, 'side')).toBeVisible();
    await expect(chip(page, 'Side by side')).toBeChecked();
    await expect(pane(page, 'left').getByRole('img', { name: 'Original' })).toBeVisible();
    await expect(pane(page, 'right').getByTestId('preview-image')).toBeVisible();
    await expect(labels(page)).toHaveText(['Original', 'Current']);
    await expect(bar(page)).toContainText('Synced zoom');
    const [left, right] = await Promise.all([
      pane(page, 'left').boundingBox(),
      pane(page, 'right').boundingBox()
    ]);
    expect(left?.width).toBeGreaterThan(100);
    expect(Math.round(left?.width ?? 0)).toBe(Math.round(right?.width ?? 0));

    await page.keyboard.press('y');
    await expect(bar(page)).toHaveCount(0);
    await expect(compareView(page, 'side')).toHaveCount(0);
    await expect(page.getByTestId('preview-image')).toBeVisible();
  });

  test('the left menu switches the left side to a snapshot', async ({ page }) => {
    const exposures = leftRenders(page);
    await openDesktop(page);
    await page.keyboard.press('y');
    await chip(page, 'Side by side').click();
    await expect(compareView(page, 'side')).toBeVisible();

    const trigger = bar(page).getByRole('button', { name: 'Left: Original' });
    await trigger.click();
    const menu = page.locator('[data-popover-content]');
    await expect(menu.getByRole('button', { name: 'Original' })).toHaveAttribute(
      'aria-pressed',
      'true'
    );
    await menu.getByRole('button', { name: 'Warm v1' }).click();

    await expect(labels(page)).toHaveText(['Snapshot · Warm v1', 'Current']);
    await expect(bar(page).getByRole('button', { name: 'Left: Warm v1' })).toBeVisible();
    await expect.poll(() => exposures.at(-1)).toBe(0.7);

    await bar(page).getByRole('button', { name: 'Close compare' }).click();
    await expect(bar(page)).toHaveCount(0);
  });

  test('a snapshot row opens side by side with the snapshot on the left', async ({ page }) => {
    await openDesktop(page);
    await page.getByRole('button', { name: 'Edit history' }).click();
    const list = page.getByRole('region', { name: 'Snapshots' });
    const row = list.getByRole('button', { name: 'Restore Warm v1', exact: true });
    await row.hover();
    const compare = list.getByRole('button', { name: 'Compare Warm v1' });
    await compare.hover();
    await expect(
      page.locator('.tooltip-content', { hasText: 'Compare with current' })
    ).toBeVisible();
    await compare.click();

    await expect(compareView(page, 'side')).toBeVisible();
    await expect(labels(page)).toHaveText(['Snapshot · Warm v1', 'Current']);
    await expect(row).toContainText('Compared');
  });

  test('zooming side by side keeps both panes on the same view', async ({ page }) => {
    await openDesktop(page);
    await page.keyboard.press('y');
    await page.keyboard.press('y');
    const left = pane(page, 'left').getByRole('img', { name: 'Original' });
    const right = pane(page, 'right').getByTestId('preview-image');
    await expect(left).toBeVisible();
    const fitted = await placement(right, pane(page, 'right'));
    await expect.poll(async () => placement(left, pane(page, 'left'))).toEqual(fitted);

    await page.keyboard.press('z');
    await expect(page.getByRole('button', { name: 'Zoom', exact: true })).not.toHaveText('Fit');
    await expect
      .poll(async () => (await placement(right, pane(page, 'right')))?.w ?? 0)
      .toBeGreaterThan(fitted?.w ?? 0);
    await expect
      .poll(async () => placement(left, pane(page, 'left')))
      .toEqual(await placement(right, pane(page, 'right')));
  });
});

test.describe('compare on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('the compare button cycles only the split', async ({ page }) => {
    await openTouch(page);

    await compareButton(page).tap();
    await expect(compareView(page, 'split')).toBeVisible();
    await expect(chip(page, 'Split')).toBeChecked();
    await expect(chip(page, 'Side by side')).toHaveCount(0);

    await compareButton(page).tap();
    await expect(bar(page)).toHaveCount(0);
    await expect(compareView(page, 'split')).toHaveCount(0);
  });

  test('turning to portrait shows side by side as the split', async ({ page }) => {
    const portrait = page.viewportSize() ?? { width: 412, height: 839 };
    await page.setViewportSize({ width: portrait.height, height: portrait.width });
    await openTouch(page);
    await compareButton(page).tap();
    await compareButton(page).tap();
    await expect(compareView(page, 'side')).toBeVisible();

    await page.setViewportSize(portrait);
    await expect(compareView(page, 'split')).toBeVisible();
    await expect(compareView(page, 'side')).toHaveCount(0);
    await expect(chip(page, 'Split')).toBeChecked();
    await expect(chip(page, 'Side by side')).toHaveCount(0);

    await page.setViewportSize({ width: portrait.height, height: portrait.width });
    await expect(compareView(page, 'side')).toBeVisible();
  });

  test('the snapshot sheet compares a snapshot with the current edits', async ({ page }) => {
    await openTouch(page);
    await page.getByRole('button', { name: 'More editor actions' }).tap();
    await page.getByRole('button', { name: 'Snapshots', exact: true }).tap();
    const sheet = page.getByRole('dialog', { name: 'Snapshots' });
    await sheet.getByRole('button', { name: 'Actions for Warm v1' }).tap();
    await page.getByRole('button', { name: 'Compare with current' }).tap();

    await expect(sheet).toBeHidden();
    await expect(compareView(page, 'split')).toBeVisible();
    await expect(labels(page)).toHaveText(['Snapshot · Warm v1', 'Current']);
    await expect(bar(page).getByRole('button', { name: 'Left: Warm v1' })).toBeVisible();
  });
});

test.describe('compare on a tablet', { tag: ['@touch', '@tablet'] }, () => {
  test('the compare button reaches side by side', async ({ page }) => {
    await openTouch(page);

    await compareButton(page).tap();
    await expect(compareView(page, 'split')).toBeVisible();
    await compareButton(page).tap();
    await expect(compareView(page, 'side')).toBeVisible();
    await expect(chip(page, 'Side by side')).toBeChecked();
    await expect(labels(page)).toHaveText(['Original', 'Current']);
    await expect(bar(page)).not.toContainText('Synced zoom');

    await compareButton(page).tap();
    await expect(bar(page)).toHaveCount(0);
  });
});
