import type { CDPSession, Page } from '@playwright/test';

export type TouchPoint = { x: number; y: number };

type TouchType = 'touchStart' | 'touchMove' | 'touchEnd' | 'touchCancel';

const sessions = new WeakMap<Page, Promise<CDPSession>>();

function session(page: Page): Promise<CDPSession> {
  const browser = page.context().browser()?.browserType().name();
  if (browser !== undefined && browser !== 'chromium') {
    throw new Error(`CDP touch input needs Chromium, not ${browser}`);
  }
  const existing = sessions.get(page);
  if (existing) return existing;
  const created = page.context().newCDPSession(page);
  sessions.set(page, created);
  return created;
}

async function dispatch(page: Page, type: TouchType, points: readonly TouchPoint[]): Promise<void> {
  const client = await session(page);
  await client.send('Input.dispatchTouchEvent', {
    type,
    touchPoints: points.map((point, id) => ({ x: point.x, y: point.y, id }))
  });
}

function between(from: TouchPoint, to: TouchPoint, t: number): TouchPoint {
  return { x: from.x + (to.x - from.x) * t, y: from.y + (to.y - from.y) * t };
}

export async function tapAt(page: Page, x: number, y: number): Promise<void> {
  await dispatch(page, 'touchStart', [{ x, y }]);
  await dispatch(page, 'touchEnd', []);
}

export async function longPress(page: Page, x: number, y: number, ms = 700): Promise<void> {
  await dispatch(page, 'touchStart', [{ x, y }]);
  await page.waitForTimeout(ms);
  await dispatch(page, 'touchEnd', []);
}

export async function drag(
  page: Page,
  from: TouchPoint,
  to: TouchPoint,
  { steps = 12, stepMs = 16 }: { steps?: number; stepMs?: number } = {}
): Promise<void> {
  await dispatch(page, 'touchStart', [from]);
  for (let step = 1; step <= steps; step += 1) {
    await page.waitForTimeout(stepMs);
    await dispatch(page, 'touchMove', [between(from, to, step / steps)]);
  }
  await dispatch(page, 'touchEnd', []);
}

export async function pinch(
  page: Page,
  center: TouchPoint,
  fromDistance: number,
  toDistance: number,
  { steps = 12, stepMs = 16 }: { steps?: number; stepMs?: number } = {}
): Promise<void> {
  const fingers = (distance: number): TouchPoint[] => [
    { x: center.x - distance / 2, y: center.y },
    { x: center.x + distance / 2, y: center.y }
  ];
  await dispatch(page, 'touchStart', fingers(fromDistance));
  for (let step = 1; step <= steps; step += 1) {
    await page.waitForTimeout(stepMs);
    const distance = fromDistance + (toDistance - fromDistance) * (step / steps);
    await dispatch(page, 'touchMove', fingers(distance));
  }
  await dispatch(page, 'touchEnd', []);
}

export async function touchDown(page: Page, point: TouchPoint): Promise<void> {
  await dispatch(page, 'touchStart', [point]);
}

export async function touchMoveTo(page: Page, point: TouchPoint): Promise<void> {
  await dispatch(page, 'touchMove', [point]);
}

export async function touchUp(page: Page): Promise<void> {
  await dispatch(page, 'touchEnd', []);
}
