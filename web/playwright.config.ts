import { defineConfig, devices } from '@playwright/test';

const port: number = Number(process.env.PORT ?? 4173);
const TOUCH = /@touch(?![-\w])/;

export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  workers: 4,
  retries: process.env.CI ? 1 : 0,
  reporter: 'list',
  use: {
    baseURL: `http://127.0.0.1:${port}`,
    trace: 'retain-on-failure'
  },
  webServer: {
    command: `npm run build && cp build/200.html build/index.html && npx serve -s build -l ${port} --no-clipboard --no-port-switching`,
    url: `http://127.0.0.1:${port}/login`,
    reuseExistingServer: !process.env.CI,
    timeout: 180_000
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
      grepInvert: /@touch/
    },
    {
      name: 'touch-phone',
      use: { ...devices['Pixel 7'] },
      grep: TOUCH,
      grepInvert: /@tablet/
    },
    {
      name: 'touch-tablet',
      use: {
        ...devices['Desktop Chrome'],
        viewport: { width: 1180, height: 820 },
        hasTouch: true,
        isMobile: true,
        deviceScaleFactor: 2
      },
      grep: TOUCH,
      grepInvert: /@phone/
    },
    {
      name: 'webkit-iphone',
      use: { ...devices['iPhone 15'] },
      grep: /@touch-smoke/
    }
  ]
});
