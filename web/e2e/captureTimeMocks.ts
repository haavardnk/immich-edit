import type { Request, Route } from '@playwright/test';

export interface MockShift {
  id: string;
  minutes: number;
  timeZone?: string;
}

export class CaptureTimeMock {
  readonly calls: MockShift[][] = [];

  async handle(route: Route, req: Request, path: string, method: string): Promise<boolean> {
    if (path !== '/api/assets/capture-time' || method !== 'PUT') return false;
    const { items } = req.postDataJSON() as { items: MockShift[] };
    this.calls.push(items);
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ failed: [] })
    });
    return true;
  }
}
