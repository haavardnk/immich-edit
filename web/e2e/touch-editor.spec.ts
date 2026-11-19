import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_ID } from './helpers';
import {
  drag,
  longPressUntil,
  tapAt,
  tapCentre,
  touchDown,
  touchUp,
  type TouchPoint
} from './touch';
import { open } from './touchShell';

async function openEditor(page: Page): Promise<void> {
  await open(page, `/assets/${ASSET_ID}`);
  await expect(toolbar(page)).toBeVisible();
  await expect(page.getByRole('slider', { name: 'Exposure' })).toBeVisible();
}

function toolbar(page: Page): Locator {
  return page.getByRole('navigation', { name: 'Editor toolbar' });
}

function modes(page: Page): Locator {
  return page.getByRole('tablist', { name: 'Editor modes' });
}

function groups(page: Page): Locator {
  return page.getByRole('tablist', { name: 'Develop groups' });
}

function tray(page: Page): Locator {
  return page.getByRole('region', { name: 'Adjustments' });
}

function exposureValue(page: Page): Locator {
  return page.getByRole('button', { name: 'Edit Exposure value' });
}

async function centreOf(target: Locator): Promise<TouchPoint> {
  const box = await target.boundingBox();
  if (!box) throw new Error('target has no box');
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}

async function doubleTap(page: Page, point: TouchPoint): Promise<void> {
  await tapAt(page, point.x, point.y);
  await tapAt(page, point.x, point.y);
}

test.describe('touch editor on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('Info opens the photo info sheet of the loupe and closes the tray', async ({ page }) => {
    await openEditor(page);
    await toolbar(page).getByRole('button', { name: 'More editor actions' }).tap();
    await page.getByRole('button', { name: 'Info', exact: true }).tap();
    const sheet = page.getByRole('dialog', { name: 'Info' });
    await expect(sheet).toBeVisible();
    await expect(sheet.getByText('Tags', { exact: true })).toBeVisible();
    await expect(tray(page)).toHaveCount(0);
    await sheet.getByRole('button', { name: 'Close info' }).tap();
    await expect(sheet).toHaveCount(0);
  });

  test('modes, groups and views switch and each group keeps its view', async ({ page }) => {
    await openEditor(page);
    await groups(page).getByRole('tab', { name: 'Color' }).tap();
    await expect(page.getByRole('slider', { name: 'Vibrance' })).toBeVisible();
    await page
      .getByRole('tablist', { name: 'Color views' })
      .getByRole('tab', { name: 'Mix' })
      .tap();
    await expect(page.getByRole('radiogroup', { name: 'Colour band' })).toBeVisible();
    await groups(page).getByRole('tab', { name: 'Light' }).tap();
    await expect(page.getByRole('slider', { name: 'Exposure' })).toBeVisible();
    await groups(page).getByRole('tab', { name: 'Color' }).tap();
    await expect(page.getByRole('radiogroup', { name: 'Colour band' })).toBeVisible();

    await modes(page).getByRole('tab', { name: 'Crop' }).tap();
    await expect(page.getByRole('tablist', { name: 'Dial' })).toBeVisible();
    await modes(page).getByRole('tab', { name: 'Masks' }).tap();
    await expect(page.getByRole('region', { name: 'New mask' })).toBeVisible();
    await modes(page).getByRole('tab', { name: 'Develop' }).tap();
    await expect(groups(page)).toBeVisible();

    await modes(page).getByRole('tab', { name: 'Develop' }).tap();
    await expect(tray(page)).toHaveCount(0);
    await expect(modes(page).getByRole('tab', { selected: true })).toHaveCount(0);
    await modes(page).getByRole('tab', { name: 'Develop' }).tap();
    await expect(groups(page)).toBeVisible();
  });

  test('a slider drags relative to its value, ignores vertical drags and resets on double-tap', async ({
    page
  }) => {
    await openEditor(page);
    const track = page.getByRole('slider', { name: 'Exposure' });
    const box = await track.boundingBox();
    if (!box) throw new Error('no track');
    const y = box.y + box.height / 2;
    const nearEnd = box.x + box.width * 0.9;

    await drag(page, { x: nearEnd, y }, { x: nearEnd, y: y + 60 });
    await expect(exposureValue(page)).toHaveText('0.00');

    await drag(page, { x: nearEnd, y }, { x: nearEnd + 30, y });
    await expect(exposureValue(page)).not.toHaveText('0.00');
    const dragged = Number(await exposureValue(page).textContent());
    expect(dragged).toBeGreaterThan(0);
    expect(dragged).toBeLessThan(2.5);
    await expect(toolbar(page).getByRole('button', { name: /^Undo/ })).toBeEnabled();

    await doubleTap(page, await centreOf(track));
    await expect(exposureValue(page)).toHaveText('0.00');
  });

  test('focus mode adjusts from a horizontal drag on the photo and ignores vertical ones', async ({
    page
  }) => {
    await openEditor(page);
    await page.getByRole('button', { name: 'Focus Exposure' }).tap();
    const focus = page.getByRole('region', { name: 'Focus Exposure' });
    await expect(focus).toBeVisible();
    await expect(toolbar(page)).toHaveCount(0);

    const photo = await centreOf(page.locator('.editor-stage'));
    await drag(page, photo, { x: photo.x, y: photo.y + 120 });
    await expect(focus.getByTestId('focus-value')).toHaveText('0.00');
    await drag(page, photo, { x: photo.x + 100, y: photo.y });
    await expect(focus.getByTestId('focus-value')).not.toHaveText('0.00');

    await focus.getByRole('button', { name: 'Brightness' }).tap();
    await expect(page.getByRole('region', { name: 'Focus Brightness' })).toBeVisible();
    await page.getByRole('button', { name: 'Done' }).tap();
    await expect(toolbar(page)).toBeVisible();
    await expect(exposureValue(page)).not.toHaveText('0.00');
  });

  test('holding the photo shows the original and a tap hides the chrome', async ({ page }) => {
    await openEditor(page);
    const stage = page.locator('.editor-stage');
    await longPressUntil(page, stage, page.getByRole('status').filter({ hasText: 'Original' }));
    await expect(page.getByRole('status').filter({ hasText: 'Original' })).toHaveCount(0);

    const photo = await centreOf(stage);
    const swap = page.locator('html');
    await tapAt(page, photo.x, photo.y);
    await expect(toolbar(page)).toHaveCount(0);
    await expect(modes(page)).toHaveCount(0);
    await expect(swap).not.toHaveAttribute('data-view-swap');
    await tapAt(page, photo.x, photo.y);
    await expect(toolbar(page)).toBeVisible();
  });

  test('picking white balance samples under the finger and keeps the chrome', async ({ page }) => {
    await page.clock.install();
    await openEditor(page);
    await groups(page).getByRole('tab', { name: 'Color' }).tap();
    await page.getByRole('button', { name: 'Pick white balance' }).tap();
    const surface = page.getByTestId('wb-picker-surface');
    await touchDown(page, await centreOf(surface));
    await expect(page.getByTestId('touch-magnifier')).toBeVisible();
    await touchUp(page);
    await expect(surface).toHaveCount(0);
    await page.clock.runFor(400);
    await expect(toolbar(page)).toBeVisible();
    await expect(page.getByRole('button', { name: 'Edit Temperature value' })).toHaveText('4300 K');
  });

  test('entering from the loupe starts on the photo, and Develop opens on Light Tone', async ({
    page
  }) => {
    await page.addInitScript(() => {
      localStorage.setItem(
        'immich-edit:touchEditor',
        JSON.stringify({ group: 'color', views: { light: 'curve' }, detent: 'compact' })
      );
      localStorage.setItem('immich-edit:editorUi', JSON.stringify({ tab: 'retouch' }));
    });
    await open(page, '/photos');
    await tapCentre(page, page.locator('[role="group"][data-name]').first());
    await page.getByRole('button', { name: 'Edit', exact: true }).tap();
    await expect(page.getByRole('navigation', { name: 'Editor toolbar' })).toBeVisible();
    await expect(tray(page)).toHaveCount(0);
    await page
      .getByRole('tablist', { name: 'Editor modes' })
      .getByRole('tab', { name: 'Develop' })
      .tap();
    await expect(groups(page).getByRole('tab', { name: 'Light' })).toHaveAttribute(
      'aria-selected',
      'true'
    );
    await expect(
      page.getByRole('tablist', { name: 'Light views' }).getByRole('tab', { name: 'Tone' })
    ).toHaveAttribute('aria-selected', 'true');
  });

  test('the tray steps from its handle or active chip, and closes by drag or its mode', async ({
    page
  }) => {
    await openEditor(page);
    const handle = page.getByRole('button', { name: 'Resize adjustments' });
    const develop = page.getByRole('tablist', { name: 'Editor modes' }).getByRole('tab', {
      name: 'Develop'
    });
    await expect(tray(page)).toHaveAttribute('data-detent', 'compact');
    await handle.tap();
    await expect(tray(page)).toHaveAttribute('data-detent', 'expanded');
    await handle.tap();
    await expect(tray(page)).toHaveAttribute('data-detent', 'compact');
    await expect(tray(page)).not.toHaveAttribute('data-snapping');

    const grip = await centreOf(handle);
    await drag(page, grip, { x: grip.x, y: grip.y + 240 });
    await expect(tray(page)).toHaveCount(0);
    await develop.tap();
    await expect(tray(page)).toHaveAttribute('data-detent', 'compact');

    await groups(page).getByRole('tab', { name: 'Color' }).tap();
    await expect(tray(page)).toHaveAttribute('data-detent', 'compact');
    await groups(page).getByRole('tab', { name: 'Light' }).tap();
    await expect(tray(page)).toHaveAttribute('data-detent', 'compact');
    await handle.tap();
    await expect(tray(page)).toHaveAttribute('data-detent', 'expanded');
    await groups(page).getByRole('tab', { name: 'Color' }).tap();
    await expect(tray(page)).toHaveAttribute('data-detent', 'expanded');
    await groups(page).getByRole('tab', { name: 'Light' }).tap();
    await expect(tray(page)).toHaveAttribute('data-detent', 'expanded');

    await groups(page).getByRole('tab', { name: 'Light' }).tap();
    await expect(tray(page)).toHaveAttribute('data-detent', 'compact');
    await groups(page).getByRole('tab', { name: 'Light' }).tap();
    await expect(tray(page)).toHaveAttribute('data-detent', 'expanded');
    await develop.tap();
    await expect(tray(page)).toHaveCount(0);
    await develop.tap();
    await expect(tray(page)).toHaveAttribute('data-detent', 'expanded');
    await groups(page).getByRole('tab', { name: 'Color' }).tap();
    await expect(tray(page)).toHaveAttribute('data-detent', 'expanded');
    await expect(page.getByRole('slider', { name: 'Vibrance' })).toBeVisible();
  });

  test('an edit marks its group and view, and Reset clears every develop edit', async ({
    page
  }) => {
    await openEditor(page);
    const light = groups(page).getByRole('tab', { name: 'Light' });
    const views = page.getByRole('tablist', { name: 'Light views' });
    const tone = views.getByRole('tab', { name: 'Tone' });

    const track = page.getByRole('slider', { name: 'Exposure' });
    const start = await centreOf(track);
    await drag(page, start, { x: start.x + 40, y: start.y });
    await expect(exposureValue(page)).not.toHaveText('0.00');
    await expect(light.locator('[data-modified-indicator]')).toHaveCount(1);
    await expect(tone.locator('[data-modified-indicator]')).toHaveCount(1);
    await expect(
      views.getByRole('tab', { name: 'Curve' }).locator('[data-modified-indicator]')
    ).toHaveCount(0);

    await page.getByRole('button', { name: 'Reset develop settings' }).tap();
    await expect(exposureValue(page)).toHaveText('0.00');
    await expect(light.locator('[data-modified-indicator]')).toHaveCount(0);
    await expect(tone.locator('[data-modified-indicator]')).toHaveCount(0);
  });

  test('rotating keeps the mode, group and view', async ({ page }) => {
    await openEditor(page);
    await groups(page).getByRole('tab', { name: 'Color' }).tap();
    await page
      .getByRole('tablist', { name: 'Color views' })
      .getByRole('tab', { name: 'Mix' })
      .tap();
    const size = page.viewportSize();
    if (!size) throw new Error('no viewport');
    await page.setViewportSize({ width: size.height, height: size.width });
    await expect(page.locator('[data-arrangement="side"]')).toBeVisible();
    await expect(groups(page).getByRole('tab', { name: 'Color' })).toHaveAttribute(
      'aria-selected',
      'true'
    );
    await expect(page.getByRole('radiogroup', { name: 'Colour band' })).toBeVisible();
    await page.setViewportSize(size);
    await expect(page.locator('[data-arrangement="stack"]')).toBeVisible();
    await expect(page.getByRole('radiogroup', { name: 'Colour band' })).toBeVisible();
  });
});

test.describe('touch editor on a tablet', { tag: ['@touch', '@tablet'] }, () => {
  test('the inspector shows one view per tab', async ({ page }) => {
    await openEditor(page);
    await expect(page.locator('[data-arrangement="inspector"]')).toBeVisible();
    await expect(toolbar(page).getByText('IMG_0001.ARW')).toBeVisible();
    await expect(page.getByRole('region', { name: 'Tone' })).toBeVisible();
    const curve = page.getByRole('region', { name: 'Curve' });
    await expect(curve).toHaveCount(0);
    const track = page.getByRole('slider', { name: 'Exposure' });
    const start = await centreOf(track);
    await drag(page, start, { x: start.x + 40, y: start.y });
    await expect(
      groups(page).getByRole('tab', { name: 'Light' }).locator('[data-modified-indicator]')
    ).toHaveCount(1);
    const offsets = await groups(page)
      .getByRole('tab')
      .evaluateAll((chips) =>
        chips.map((chip) => {
          const range = document.createRange();
          range.selectNodeContents(chip);
          const text = range.getClientRects()[0];
          const box = chip.getBoundingClientRect();
          return Math.abs(text.left + text.width / 2 - (box.left + box.width / 2));
        })
      );
    for (const offset of offsets) expect(offset).toBeLessThan(1.5);

    await page
      .getByRole('tablist', { name: 'Light views' })
      .getByRole('tab', { name: 'Curve' })
      .tap();
    await expect(curve).toBeVisible();
    await expect(page.getByRole('region', { name: 'Tone' })).toHaveCount(0);
    await modes(page).getByRole('tab', { name: 'Retouch' }).tap();
    await expect(groups(page)).toHaveCount(0);
    const panel = page.getByRole('complementary', { name: 'Adjustments' });
    await modes(page).getByRole('tab', { name: 'Retouch' }).tap();
    await expect(panel).toHaveCount(0);
    await modes(page).getByRole('tab', { name: 'Retouch' }).tap();
    await expect(panel).toBeVisible();
  });
});
