import { describe, expect, it } from 'vitest';
import { boxFrame, containBox, onScreen } from './zoomGeometry.ts';

const FRAME = { left: 10, top: 20, width: 400, height: 300 };
const VIEWPORT = { width: 800, height: 600 };

describe('containBox', () => {
  it('letterboxes a wide image inside a taller frame', () => {
    expect(containBox(FRAME, 2)).toEqual({ left: 10, top: 70, width: 400, height: 200 });
  });

  it('pillarboxes a tall image inside a wider frame', () => {
    expect(containBox(FRAME, 0.5)).toEqual({ left: 135, top: 20, width: 150, height: 300 });
  });

  it('keeps the frame when the ratio is unknown', () => {
    expect(containBox(FRAME, 0)).toBe(FRAME);
    expect(containBox(FRAME, Number.NaN)).toBe(FRAME);
  });
});

describe('onScreen', () => {
  it('accepts a box mostly inside the viewport', () => {
    expect(onScreen({ left: -40, top: 0, width: 100, height: 100 }, VIEWPORT)).toBe(true);
  });

  it('rejects a box mostly scrolled away or empty', () => {
    expect(onScreen({ left: 0, top: 560, width: 100, height: 100 }, VIEWPORT)).toBe(false);
    expect(onScreen({ left: 0, top: 900, width: 100, height: 100 }, VIEWPORT)).toBe(false);
    expect(onScreen({ left: 0, top: 0, width: 0, height: 100 }, VIEWPORT)).toBe(false);
  });
});

describe('boxFrame', () => {
  it('positions a fixed element over the box', () => {
    expect(boxFrame(FRAME)).toEqual({ left: '10px', top: '20px', width: '400px', height: '300px' });
  });
});
