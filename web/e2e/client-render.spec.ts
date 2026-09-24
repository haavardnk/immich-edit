import fs from 'node:fs';
import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';
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
    if (req.method() === 'POST' && (name === 'preview' || name === 'source')) posts.push(name);
  });
  return posts;
}

test('a display slider renders in the browser without a server round trip', async ({ page }) => {
  const complaints: string[] = [];
  page.on('console', (msg) => {
    if (msg.type() === 'error') complaints.push(msg.text());
  });
  const posts = renderPosts(page);
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
  await gotoAsset(page);

  const canvas = page.getByTestId('preview-canvas');
  await expect(canvas).toBeVisible(FRAME);
  await expect.poll(() => posts).toEqual(['source']);
  const drawn = (): Promise<string> => canvas.evaluate((c: HTMLCanvasElement) => c.toDataURL());
  const before = await drawn();

  posts.length = 0;
  const exposure = page.getByRole('slider', { name: 'Exposure', exact: true });
  await exposure.focus();
  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('ArrowRight');
  await expect.poll(drawn, FRAME).not.toBe(before);
  await page.waitForTimeout(500);

  expect(posts).toEqual([]);
  expect(complaints).toEqual([]);
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
  await expect(page.getByTestId('preview-canvas')).toHaveCount(0);
});
