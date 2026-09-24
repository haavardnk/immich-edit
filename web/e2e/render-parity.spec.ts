import fs from 'node:fs';
import path from 'node:path';
import { expect, test } from '@playwright/test';

const web = path.resolve(import.meta.dirname, '..');
const fixtures = path.join(web, 'e2e/fixtures/render');
const spec = JSON.parse(fs.readFileSync(path.join(fixtures, 'case.json'), 'utf8'));

const files: Record<string, [string, string]> = {
  'index.html': [path.join(fixtures, 'harness.html'), 'text/html'],
  'web_render.js': [path.join(web, 'src/lib/wasm/web_render.js'), 'text/javascript'],
  'web_render_bg.wasm': [path.join(web, 'src/lib/wasm/web_render_bg.wasm'), 'application/wasm'],
  'case.json': [path.join(fixtures, 'case.json'), 'application/json'],
  'source.iesr': [path.join(fixtures, 'source.iesr'), 'application/octet-stream'],
  'expected.rgb': [path.join(fixtures, 'expected.rgb'), 'application/octet-stream'],
  'lut.cube': [path.join(fixtures, 'lut.cube'), 'text/plain'],
  dcp: [path.join(web, '../crates/backend/assets/dcp', spec.dcp), 'application/octet-stream']
};

const swiftshader =
  process.platform === 'linux' || process.env.WEBGPU_SWIFTSHADER === '1'
    ? [
        '--enable-features=Vulkan',
        '--use-vulkan=swiftshader',
        '--use-webgpu-adapter=swiftshader',
        '--disable-vulkan-surface'
      ]
    : [];

test.use({
  channel: 'chromium',
  launchOptions: { args: ['--enable-unsafe-webgpu', ...swiftshader] }
});

declare global {
  interface Window {
    runParity(): Promise<{
      adapter: string;
      width: number;
      height: number;
      frame: [number, number];
      expected: [number, number];
      histogramCount: number;
      meanAbs: number;
      max: number;
      initMs: number;
      firstRenderMs: number;
      renderMs: number;
    }>;
  }
}

test('the wasm renderer draws what the native renderer draws', async ({ page }) => {
  const complaints: string[] = [];
  page.on('console', (msg) => {
    if (msg.type() === 'warning' || msg.type() === 'error') complaints.push(msg.text());
  });
  await page.route('**/__render/**', (route) => {
    const name = new URL(route.request().url()).pathname.split('/').pop() ?? '';
    const file = files[name];
    if (!file) return route.fulfill({ status: 404 });
    return route.fulfill({ path: file[0], contentType: file[1] });
  });
  await page.goto('/__render/index.html');
  await page.waitForFunction(() => typeof window.runParity === 'function');
  const result = await page.evaluate(() => window.runParity());
  console.log(`render parity: ${JSON.stringify(result)}`);

  expect(complaints).toEqual([]);
  expect(result.frame).toEqual(result.expected);
  expect([result.width, result.height]).toEqual(result.expected);
  expect(result.histogramCount).toBeGreaterThan(0);
  expect(result.meanAbs).toBeLessThan(0.5);
});
