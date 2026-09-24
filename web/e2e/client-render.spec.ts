import fs from 'node:fs';
import path from 'node:path';
import { expect, test, type Locator, type Page } from '@playwright/test';
import { gotoAsset, installMocks } from './helpers';
import { webgpu } from './webgpu';

const web = path.resolve(import.meta.dirname, '..');
const fixtures = path.join(web, 'e2e/fixtures/render');
const spec = JSON.parse(fs.readFileSync(path.join(fixtures, 'case.json'), 'utf8'));
const dcp = path.join(web, '../crates/backend/assets/dcp', spec.dcp);
const FRAME = { timeout: 60_000 };

test.use(webgpu);
test.describe.configure({ timeout: 180_000 });

function renderPosts(page: Page): string[] {
  const posts: string[] = [];
  page.on('request', (req) => {
    const name = new URL(req.url()).pathname.split('/').pop();
    if (req.method() !== 'POST' || (name !== 'preview' && name !== 'source')) return;
    posts.push(name === 'source' && req.postDataJSON()?.roi ? 'tile' : name);
  });
  return posts;
}

async function serveFixture(page: Page): Promise<void> {
  await installMocks(page, { renderer: 'auto' });
  await page.route(/\/api\/assets\/[^/]+\/source$/, (route) =>
    route.fulfill({
      path: path.join(fixtures, 'source.iesr'),
      contentType: 'application/vnd.immich-edit.source',
      headers: { 'x-source-dcp': 'fixture' }
    })
  );
  await page.route('**/api/dcp/fixture/raw', (route) =>
    route.fulfill({ path: dcp, contentType: 'application/octet-stream' })
  );
}

function pixels(canvas: Locator): () => Promise<string> {
  return () => canvas.evaluate((c: HTMLCanvasElement) => c.toDataURL());
}

async function nudgeExposure(page: Page): Promise<void> {
  await page.getByRole('slider', { name: 'Exposure', exact: true }).focus();
  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('ArrowRight');
}

test('a display slider renders in the browser without a server round trip', async ({ page }) => {
  const complaints: string[] = [];
  page.on('console', (msg) => {
    if (msg.type() === 'error') complaints.push(msg.text());
  });
  const posts = renderPosts(page);
  await serveFixture(page);
  await gotoAsset(page);

  const canvas = page.locator('canvas[data-testid="preview-image"]');
  await expect(canvas).toBeVisible(FRAME);
  await expect.poll(() => posts).toEqual(['source']);
  const before = await pixels(canvas)();

  posts.length = 0;
  await nudgeExposure(page);
  await expect.poll(pixels(canvas), FRAME).not.toBe(before);
  await page.waitForTimeout(500);

  expect(posts).toEqual([]);
  expect(complaints).toEqual([]);
});

test('at 1:1 a slider tick posts nothing and a pan posts one tile', async ({ page }) => {
  const posts = renderPosts(page);
  await serveFixture(page);
  await gotoAsset(page);
  const base = page.locator('canvas[data-testid="preview-image"]');
  await expect(base).toBeVisible(FRAME);

  await base.dblclick();
  const tile = page.locator('canvas[data-testid="view-render"]');
  await expect(tile).toBeVisible(FRAME);
  await expect.poll(() => posts.filter((p) => p === 'tile').length, FRAME).toBe(1);
  await page.waitForTimeout(500);
  const before = await pixels(tile)();

  posts.length = 0;
  await nudgeExposure(page);
  await expect.poll(pixels(tile), FRAME).not.toBe(before);
  await page.waitForTimeout(500);
  expect(posts).toEqual([]);

  await base.hover();
  await page.mouse.wheel(400, 0);
  await expect.poll(() => posts, FRAME).toEqual(['tile']);
  await page.waitForTimeout(500);
  expect(posts).toEqual(['tile']);
});

test('previews fall back to the server without WebGPU', async ({ page }) => {
  const posts = renderPosts(page);
  await page.addInitScript(() =>
    Object.defineProperty(Navigator.prototype, 'gpu', { get: () => undefined })
  );
  await installMocks(page, { renderer: 'auto' });
  await gotoAsset(page);

  await expect.poll(() => posts).toContain('preview');
  expect(posts).not.toContain('source');
  await expect(page.locator('canvas[data-testid="preview-image"]')).toHaveCount(0);
});
