import fs from 'node:fs';
import path from 'node:path';
import { expect, test } from '@playwright/test';
import { webgpu } from './webgpu';

const web = path.resolve(import.meta.dirname, '..');
const fixtures = path.join(web, 'e2e/fixtures/render');
const cases = ['fit', 'tile'];
const dcp = JSON.parse(fs.readFileSync(path.join(fixtures, 'fit/case.json'), 'utf8')).dcp;
const MIN_PSNR = 50;

const shared: Record<string, [string, string]> = {
  'index.html': [path.join(fixtures, 'harness.html'), 'text/html'],
  'web_render.js': [path.join(web, 'src/lib/wasm/web_render.js'), 'text/javascript'],
  'web_render_bg.wasm': [path.join(web, 'src/lib/wasm/web_render_bg.wasm'), 'application/wasm'],
  'lut.cube': [path.join(fixtures, 'lut.cube'), 'text/plain'],
  dcp: [path.join(web, '../crates/backend/assets/dcp', dcp), 'application/octet-stream']
};

function fixture(name: string): [string, string] | undefined {
  const [dir, file] = name.split('/');
  if (!file) return shared[dir];
  if (!cases.includes(dir) || !['case.json', 'source.iesr', 'expected.rgb'].includes(file)) {
    return undefined;
  }
  return [path.join(fixtures, dir, file), 'application/octet-stream'];
}

test.use(webgpu);

declare global {
  interface Window {
    runParity(name: string): Promise<{
      adapter: string;
      width: number;
      height: number;
      frame: [number, number];
      expected: [number, number];
      histogramCount: number;
      meanAbs: number;
      mse: number;
      max: number;
      initMs: number;
      firstRenderMs: number;
      renderMs: number;
    }>;
  }
}

for (const name of cases) {
  test(`the wasm renderer draws the ${name} view like the server`, async ({ page }) => {
    const complaints: string[] = [];
    page.on('console', (msg) => {
      if (msg.type() === 'warning' || msg.type() === 'error') complaints.push(msg.text());
    });
    await page.route('**/__render/**', (route) => {
      const file = fixture(new URL(route.request().url()).pathname.split('/__render/')[1]);
      if (!file) return route.fulfill({ status: 404 });
      return route.fulfill({ path: file[0], contentType: file[1] });
    });
    await page.goto('/__render/index.html');
    await page.waitForFunction(() => typeof window.runParity === 'function');
    const result = await page
      .evaluate((name) => window.runParity(name), name)
      .catch((err: Error) => {
        throw new Error(`${err.message}\nconsole:\n${complaints.join('\n')}`);
      });
    const psnr = result.mse === 0 ? Infinity : 10 * Math.log10((255 * 255) / result.mse);
    console.log(`render parity ${name}: psnr ${psnr.toFixed(2)} dB ${JSON.stringify(result)}`);

    expect(complaints).toEqual([]);
    expect(result.frame).toEqual(result.expected);
    expect([result.width, result.height]).toEqual(result.expected);
    expect(result.histogramCount).toBeGreaterThan(0);
    expect(psnr).toBeGreaterThanOrEqual(MIN_PSNR);
  });
}
