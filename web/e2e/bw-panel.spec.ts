import { expect, test } from '@playwright/test';
import { gotoAsset, installMocks, type PreviewRequest } from './helpers';

interface BwSent {
  enabled: boolean;
  mix: { green: number };
}

function lastBw(requests: PreviewRequest[]): BwSent | undefined {
  return (requests.at(-1)?.edits as { color?: { bw?: BwSent } } | undefined)?.color?.bw;
}

test('black and white converts, mixes, bypasses and resets', async ({ page }) => {
  const requests: PreviewRequest[] = [];
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, {
    onPreview: (req) => requests.push(req),
    onSave: (body) => saves.push(body)
  });
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Black & White', exact: true }).click();
  const green = page.getByRole('slider', { name: 'Green', exact: true });
  await expect(green).toHaveCount(0);
  await page.getByLabel('Convert to Black & White').check();
  await expect.poll(() => lastBw(requests)?.enabled).toBe(true);

  await green.fill('40');
  await expect.poll(() => lastBw(requests)?.mix.green).toBe(40);
  await expect.poll(() => saves.some((s) => JSON.stringify(s).includes('"green":40'))).toBe(true);

  await page.getByRole('button', { name: 'Bypass B&W' }).hover();
  await page.mouse.down();
  await expect.poll(() => lastBw(requests)?.enabled).toBe(false);
  await page.mouse.up();
  await expect.poll(() => lastBw(requests)?.enabled).toBe(true);

  await page.getByRole('button', { name: 'Reset B&W', exact: true }).click();
  await expect.poll(() => lastBw(requests)?.enabled).toBe(false);
  await expect(green).toHaveCount(0);
});
