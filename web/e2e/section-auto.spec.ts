import { expect, test, type Page } from '@playwright/test';
import { ASSET_ID, installMocks } from './helpers';
import { open } from './touchShell';

const TONE = { highlights: -40, shadows: 25, whites: 9, blacks: -11 };

type Gate = { release: () => void; calls: () => number };

async function routeAuto(page: Page): Promise<Gate> {
  let calls = 0;
  let release = (): void => {};
  const waiting = new Promise<void>((resolve) => (release = resolve));
  await page.route('**/api/assets/*/edits/auto', async (route) => {
    calls += 1;
    const body = JSON.parse(route.request().postData() ?? '{}');
    await waiting;
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        ...body,
        basic: { ...body.basic, exposure_ev: 0.5, contrast: 14, clarity: 8 },
        tone: TONE
      })
    });
  });
  return { release: () => release(), calls: () => calls };
}

async function openDesktop(page: Page): Promise<void> {
  await installMocks(page);
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto(`/assets/${ASSET_ID}`);
  await expect(page.getByRole('slider', { name: 'Exposure' })).toBeVisible();
}

test.describe('auto per section on desktop', () => {
  test('Auto Tone analyses in its header and applies only the tone sliders', async ({ page }) => {
    await openDesktop(page);
    const gate = await routeAuto(page);
    const tone = page.locator('[data-setting="section:tone"]');
    await tone.getByRole('button', { name: 'Auto Tone' }).click();
    await expect(tone.getByRole('status')).toHaveText('Analyzing…');
    await expect(page.getByRole('button', { name: 'Auto', exact: true })).toBeDisabled();
    gate.release();

    await expect(tone.getByRole('status')).toHaveCount(0);
    await expect(page.getByRole('slider', { name: 'Highlights' })).toHaveValue('-40');
    await expect(page.getByRole('slider', { name: 'Exposure' })).toHaveValue('0.5');
    await expect(page.getByRole('slider', { name: 'Clarity' })).toHaveValue('0');
    expect(gate.calls()).toBe(1);
  });
});

test.describe('auto per section on touch', { tag: ['@touch'] }, () => {
  test('Light › Tone carries its wand', async ({ page }) => {
    await open(page, `/assets/${ASSET_ID}`);
    const gate = await routeAuto(page);
    gate.release();
    await page.getByRole('button', { name: 'Auto Tone' }).tap();
    await expect(page.getByRole('slider', { name: 'Highlights' })).toHaveValue('-40');
  });
});
