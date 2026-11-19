import { expect, test, type Locator, type Page } from '@playwright/test';
import { gotoAsset, installMocks } from './helpers';

type SaveBody = {
  action?: string | null;
  manifest?: { ops?: { transform?: { perspective?: { vertical?: number } } } };
};

async function draw(
  page: Page,
  surface: Locator,
  from: [number, number],
  to: [number, number]
): Promise<void> {
  const box = await surface.boundingBox();
  if (!box) throw new Error('upright surface has no box');
  await page.mouse.move(box.x + box.width * from[0], box.y + box.height * from[1]);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width * to[0], box.y + box.height * to[1], { steps: 6 });
  await page.mouse.up();
}

async function armUpright(page: Page): Promise<Locator> {
  await page.keyboard.press('Shift+R');
  await expect(page.getByRole('button', { name: 'resize nw' })).toBeVisible();
  const button = page.getByRole('button', { name: 'Guided upright' });
  await button.click();
  await expect(button).toHaveAttribute('aria-pressed', 'true');
  return page.getByTestId('upright-surface');
}

test('two converging verticals save a vertical keystone as one guided upright step', async ({
  page
}) => {
  const saves: SaveBody[] = [];
  await installMocks(page, { onSave: (body) => saves.push(body as SaveBody) });
  await gotoAsset(page);
  const surface = await armUpright(page);

  const counts = page.getByTestId('upright-counts');
  const card = page.getByTestId('upright-card');
  await expect(counts).toHaveText(/Vertical 0 of 2\s*·\s*Horizontal 0 of 2/);
  await draw(page, surface, [0.3, 0.85], [0.36, 0.15]);
  await expect(page.getByRole('button', { name: 'Remove guide 1' })).toBeVisible();
  await expect(counts).toHaveText(/Vertical 1 of 2\s*·\s*Horizontal 0 of 2/);
  await expect(card).toContainText('Draw one more line');
  await draw(page, surface, [0.2, 0.2], [0.6, 0.6]);
  await expect(page.getByRole('button', { name: 'Remove guide 2' })).toHaveCount(0);
  await expect(card).toContainText(/Last line dropped: \d+° is neither level nor upright/);
  expect(saves).toHaveLength(0);

  await draw(page, surface, [0.7, 0.85], [0.64, 0.15]);
  await expect.poll(() => saves.length).toBe(1);
  const body = saves[0];
  expect(body?.action).toBe('Guided upright');
  expect(body?.manifest?.ops?.transform?.perspective?.vertical ?? 0).toBeGreaterThan(5);
  await expect(page.getByRole('button', { name: /^upright guide \d (start|end)$/ })).toHaveCount(4);
  await expect(page.getByTestId('upright-label')).toHaveText(['V1', 'V2']);
  await expect(card).not.toContainText('Last line dropped');
  await expect(card).toContainText('A third vertical replaces V1, the oldest');

  const handle = page.getByRole('button', { name: 'upright guide 2 end' });
  const at = await handle.boundingBox();
  if (!at) throw new Error('guide handle has no box');
  await page.mouse.move(at.x + at.width / 2, at.y + at.height / 2);
  await page.mouse.down();
  await page.mouse.move(at.x + at.width / 2 - 20, at.y + at.height / 2, { steps: 4 });
  expect(saves).toHaveLength(1);
  await page.mouse.up();
  await expect.poll(() => saves.length).toBe(2);
  expect(saves[1]?.action).toBe('Guided upright');

  await card.getByRole('button', { name: 'Clear guides' }).click();
  await expect(page.getByTestId('upright-label')).toHaveCount(0);
  await expect(counts).toHaveText(/Vertical 0 of 2\s*·\s*Horizontal 0 of 2/);
  expect(saves).toHaveLength(2);
});

test('guides are removable, tools stay exclusive and Escape disarms upright', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);
  const surface = await armUpright(page);

  await draw(page, surface, [0.3, 0.85], [0.32, 0.15]);
  await draw(page, surface, [0.15, 0.4], [0.85, 0.45]);
  await expect(page.getByRole('button', { name: /^Remove guide \d$/ })).toHaveCount(2);
  await page.getByRole('button', { name: 'Remove guide 1' }).click();
  await expect(page.getByRole('button', { name: /^Remove guide \d$/ })).toHaveCount(1);

  const upright = page.getByRole('button', { name: 'Guided upright' });
  await page.getByRole('button', { name: 'Corner handles' }).click();
  await expect(upright).toHaveAttribute('aria-pressed', 'false');
  await expect(surface).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'perspective corner 1' })).toBeVisible();

  await upright.click();
  await expect(page.getByRole('button', { name: 'perspective corner 1' })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Remove guide 1' })).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(upright).toHaveAttribute('aria-pressed', 'false');
  await expect(surface).toHaveCount(0);
  await expect(page.getByRole('tab', { name: 'Crop' })).toHaveAttribute('aria-selected', 'true');
});
