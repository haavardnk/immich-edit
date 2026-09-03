import { beforeEach, describe, expect, it, vi } from 'vitest';
import { clickMask } from './masks';
import { sendJson } from './client';

vi.mock('./client', async (original) => ({
  ...(await original<typeof import('./client')>()),
  sendJson: vi.fn(() => Promise.resolve({}))
}));

const mocked = vi.mocked(sendJson);

describe('clickMask', () => {
  beforeEach(() => mocked.mockClear());

  it.each([
    ['no box', undefined, null],
    ['a box', { x0: 0.1, y0: 0.2, x1: 0.3, y1: 0.4 }, { x0: 0.1, y0: 0.2, x1: 0.3, y1: 0.4 }]
  ])('sends %s', async (_name, bbox, expected) => {
    await clickMask('asset', [], 0, 0, undefined, false, bbox);
    expect(mocked.mock.calls[0][2]).toMatchObject({ bbox: expected, points: [] });
  });

  it('keeps points and box independent', async () => {
    const points = [{ x: 0.5, y: 0.5, positive: true }];
    await clickMask('asset', points, 2, 3, 'base', true, { x0: 0, y0: 0, x1: 1, y1: 1 });
    expect(mocked.mock.calls[0][2]).toEqual({
      points,
      grow: 2,
      feather: 3,
      base_raster_id: 'base',
      subtract: true,
      bbox: { x0: 0, y0: 0, x1: 1, y1: 1 }
    });
  });
});
