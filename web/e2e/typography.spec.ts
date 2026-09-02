import { createHash } from 'node:crypto';
import { expect, test } from '@playwright/test';
import { installMocks } from './helpers';

const SANS_PATH = '/fonts/GoogleSans%5BGRAD,opsz,wght%5D.ttf';
const MONO_PATH = '/fonts/GoogleSansCode%5BMONO,wght%5D.ttf';
const ASSETS = new Map([
  [SANS_PATH, 'd0a87d835a944b8b40d0e82a5651bb59ab97b936a2aeed5946eb57e7b2a3a90a'],
  [MONO_PATH, '74642762fae66d5883077794a763e1086394b8cb72f2623e456e06359dc5c64b'],
  [
    '/licenses/GoogleSans/COPYRIGHT.txt',
    '44b50e396c70727aa4dc16e9bb967ff74e5086edfebfe4c3935a948aafb130a0'
  ],
  [
    '/licenses/GoogleSans/OFL.txt',
    '831b6e041705873c3e6efb18c4c96202bc41c8be42a456d3d26c47abf14816d0'
  ],
  [
    '/licenses/GoogleSans/TRADEMARKS.txt',
    '9b75a5efcc2de1e1aa214a896c4d58a5544c1d88cb1e3fb7a42281404ff10708'
  ],
  [
    '/licenses/GoogleSansCode/COPYRIGHT.txt',
    'a7bd5652d8c674886fcdf850357dd8f91a6a63ce0c9ecd81a1a15f122bc475f0'
  ],
  [
    '/licenses/GoogleSansCode/OFL.txt',
    'ff8f4fbc4f1a71c7d4c6481c48dc2b21c43f8dc15872536667fee45c15d20928'
  ],
  [
    '/licenses/GoogleSansCode/TRADEMARKS.md',
    'd9c627c968837b4a148195153271ca69e98d4bc750b041e0d98ba7b376b1cf50'
  ]
]);

test('uses the bundled Google Sans families', async ({ page }) => {
  for (const [path, hash] of ASSETS) {
    const response = await page.request.get(path);
    expect(response.ok(), path).toBe(true);
    expect(
      createHash('sha256')
        .update(await response.body())
        .digest('hex'),
      path
    ).toBe(hash);
  }

  await installMocks(page);
  await page.goto('/photos');
  await expect(page.getByRole('link').first()).toBeVisible();

  const typography = await page.evaluate(async () => {
    const mono = document.createElement('span');
    mono.className = 'font-mono';
    mono.textContent = '123';
    document.body.append(mono);

    await Promise.all([
      document.fonts.load('400 16px "Google Sans"'),
      document.fonts.load('400 16px "Google Sans Code"')
    ]);

    const result = {
      sans: getComputedStyle(document.body).fontFamily,
      mono: getComputedStyle(mono).fontFamily,
      sansLoaded: document.fonts.check('400 16px "Google Sans"'),
      monoLoaded: document.fonts.check('400 16px "Google Sans Code"')
    };
    mono.remove();
    return result;
  });

  expect(typography).toEqual({
    sans: '"Google Sans", sans-serif',
    mono: '"Google Sans Code", monospace',
    sansLoaded: true,
    monoLoaded: true
  });

  const preloads = await page
    .locator('link[rel="preload"][as="font"]')
    .evaluateAll((links) => links.map((link) => link.getAttribute('href')));
  expect(preloads).toEqual([SANS_PATH]);
});

test('keeps generic fallbacks when font requests fail', async ({ page }) => {
  const failedPaths = new Set<string>();
  await page.route('**/fonts/**', async (route) => {
    failedPaths.add(new URL(route.request().url()).pathname);
    await route.abort('failed');
  });
  await installMocks(page);
  await page.goto('/photos');
  await expect(page.getByRole('link').first()).toBeVisible();

  const typography = await page.evaluate(async () => {
    const mono = document.createElement('span');
    mono.className = 'font-mono';
    mono.textContent = '123';
    document.body.append(mono);
    mono.getBoundingClientRect();
    await document.fonts.ready;

    const result = {
      sans: getComputedStyle(document.body).fontFamily,
      mono: getComputedStyle(mono).fontFamily,
      sansLoaded: document.fonts.check('400 16px "Google Sans"'),
      monoLoaded: document.fonts.check('400 16px "Google Sans Code"')
    };
    mono.remove();
    return result;
  });

  expect(typography).toEqual({
    sans: '"Google Sans", sans-serif',
    mono: '"Google Sans Code", monospace',
    sansLoaded: false,
    monoLoaded: false
  });
  expect(failedPaths).toEqual(new Set([SANS_PATH, MONO_PATH]));
});
