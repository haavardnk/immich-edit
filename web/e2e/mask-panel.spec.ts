import { expect, test, type Page } from '@playwright/test';
import { gotoAsset, installMocks } from './helpers';

type SavedRadial = { center?: { x: number; y: number }; feather?: number };

function lastRadial(saves: Array<Record<string, unknown>>): SavedRadial | undefined {
  const body = saves.at(-1) as {
    manifest?: {
      ops?: { masks?: { layers?: Array<{ components?: Array<{ kind?: SavedRadial }> }> } };
    };
  };
  return body?.manifest?.ops?.masks?.layers?.[0]?.components?.[0]?.kind;
}

async function addRadial(page: Page): Promise<void> {
  await page.getByRole('tab', { name: 'Masks' }).click();
  await page.getByRole('button', { name: 'New mask' }).click();
  await page.getByRole('button', { name: 'Radial gradient', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Radial center' })).toBeVisible();
}

async function dragBy(page: Page, name: string, to: { x: number; y: number }): Promise<void> {
  const box = await page.getByRole('button', { name }).boundingBox();
  if (!box) throw new Error(`${name} has no box`);
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(to.x, to.y, { steps: 5 });
  await page.mouse.up();
}

test('the radial feather handle stays grabbable at full feather', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, { onSave: (body) => saves.push(body) });
  await gotoAsset(page);
  await addRadial(page);

  const centre = await page.getByRole('button', { name: 'Radial center' }).boundingBox();
  if (!centre) throw new Error('radial centre has no box');
  const middle = { x: centre.x + centre.width / 2, y: centre.y + centre.height / 2 };
  await dragBy(page, 'Radial feather', middle);
  await expect.poll(() => lastRadial(saves)?.feather).toBe(1);

  const feather = page.getByRole('button', { name: 'Radial feather' });
  await expect(feather).toBeVisible();
  const edge = await page.getByRole('button', { name: 'Radial radius x' }).first().boundingBox();
  if (!edge) throw new Error('radial edge has no box');
  await dragBy(page, 'Radial feather', {
    x: (middle.x + edge.x + edge.width / 2) / 2,
    y: middle.y
  });
  await expect.poll(() => lastRadial(saves)?.feather ?? 1).toBeLessThan(0.9);
});

test('arrow keys nudge the selected radial', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, { onSave: (body) => saves.push(body) });
  await gotoAsset(page);
  await addRadial(page);
  await expect.poll(() => lastRadial(saves)?.center?.x).toBe(0.5);

  await page.keyboard.press('Shift+ArrowRight');
  await page.keyboard.press('ArrowUp');
  await expect.poll(() => lastRadial(saves)?.center?.x ?? 0).toBeGreaterThan(0.5);
  await expect.poll(() => lastRadial(saves)?.center?.y ?? 1).toBeLessThan(0.5);
  await expect(page).toHaveURL(/\/assets\/[^/?]+/);
});

test('the refine section keeps its state across a tab switch', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);
  await addRadial(page);

  const refine = page.getByRole('button', { name: /^Refine/ });
  const reselect = async (): Promise<void> => {
    await page.keyboard.press('d');
    await page.keyboard.press('m');
    await page.getByRole('button', { name: 'Mask 1', exact: true }).click();
  };
  await reselect();
  await expect(refine).toHaveAttribute('aria-expanded', 'false');
  await refine.click();
  await expect(refine).toHaveAttribute('aria-expanded', 'true');

  await reselect();
  await expect(refine).toHaveAttribute('aria-expanded', 'true');
});

test('mask shortcuts are shown where the actions live', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Masks' }).click();

  await page.getByRole('button', { name: 'Toggle mask overlays' }).hover();
  await expect(page.getByText('Hide mask overlays (O)')).toBeVisible();
  await page.getByRole('button', { name: 'New mask' }).click();
  await expect(page.getByRole('button', { name: 'Linear gradient', exact: true })).toContainText(
    /Shift\+L|⇧L/
  );
  await expect(page.getByRole('button', { name: 'Radial gradient', exact: true })).toContainText(
    /Shift\+M|⇧M/
  );
  await expect(page.getByRole('button', { name: 'Brush', exact: true })).toContainText('K');
  await page.keyboard.press('Escape');

  await page.keyboard.press('Shift+L');
  await expect(page.getByRole('button', { name: 'Linear start' })).toBeVisible();

  await page.keyboard.press('Shift+/');
  await expect(page.getByText('Add a polygon corner', { exact: true })).toBeVisible();
  await expect(page.getByText('Remove a polygon corner', { exact: true })).toBeVisible();
});
