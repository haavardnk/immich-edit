import { expect, type Locator, type Page } from '@playwright/test';
import { ASSET_ID, json, type InstallOpts } from './helpers';
import { mockLibrary } from './touchShell';

export const MASK_MODELS = [
  { id: 'ormbg', kind: 'subject', installed: true },
  { id: 'skyseg', kind: 'sky', installed: true },
  { id: 'depth', kind: 'depth', installed: false },
  { id: 'sam', kind: 'click', installed: true }
];

export async function openMasks(page: Page, opts: InstallOpts = {}): Promise<void> {
  await mockLibrary(page, opts);
  await page.route('**/api/masks/models', (route) =>
    route.fulfill(
      json({ runtime: 'cpu', enabled: true, models: MASK_MODELS, active: {}, semantic_classes: [] })
    )
  );
  await page.goto(`/assets/${ASSET_ID}`);
  await expect(page.locator('html')).toHaveAttribute('data-layout', 'touch');
  await modes(page).getByRole('tab', { name: 'Masks' }).tap();
  await expect(page.getByRole('button', { name: 'New mask' })).toBeVisible();
}

export function modes(page: Page): Locator {
  return page.getByRole('tablist', { name: 'Editor modes' });
}

export function sheet(page: Page): Locator {
  return page.getByRole('dialog');
}

export function cards(page: Page): Locator {
  return page.getByTestId('mask-layer-card');
}

export function shapes(page: Page): Locator {
  return page.getByTestId('mask-shape-row');
}

export async function addFromSheet(page: Page, trigger: Locator, tool: string): Promise<void> {
  await trigger.tap();
  await sheet(page).getByRole('button', { name: tool, exact: true }).tap();
  await expect(sheet(page)).toHaveCount(0);
}

export type SavedLayer = { name?: string; invert?: boolean; components?: { invert?: boolean }[] };

export function savedLayers(saves: Array<Record<string, unknown>>): SavedLayer[] {
  const body = saves.at(-1) as { manifest?: { ops?: { masks?: { layers?: SavedLayer[] } } } };
  return body?.manifest?.ops?.masks?.layers ?? [];
}
