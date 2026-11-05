import { describe, expect, it } from 'vitest';
import type { AssetGridLayout } from './assetGridLayout';
import {
  anchoredViewTop,
  assetIndexAt,
  gridAnchor,
  scaleStepper,
  WHEEL_IDLE_MS,
  WHEEL_NOTCH_MS,
  wheelStepper,
  wheelZoomAmount,
  zoomAnchor,
  ZOOM_STEP_RATIO
} from './gridZoom';

function grid(rowHeight: number, perRow: number, rowCount: number, width = 100): AssetGridLayout {
  const gap = 4;
  const boxes = Array.from({ length: perRow * rowCount }, (_, index) => {
    const row = Math.floor(index / perRow);
    const column = index % perRow;
    return {
      top: row * (rowHeight + gap),
      left: column * (width + gap),
      width,
      height: rowHeight,
      row,
      column
    };
  });
  const rows = Array.from({ length: rowCount }, (_, row) => ({
    top: row * (rowHeight + gap),
    height: rowHeight,
    startIndex: row * perRow,
    endIndex: (row + 1) * perRow
  }));
  return { boxes, rows, height: rowCount * (rowHeight + gap) - gap };
}

describe('wheel zoom steps', () => {
  it('turns pixels, lines and pages into a zoom amount, wheel up zooming in', () => {
    expect(wheelZoomAmount(-100, 0)).toBeCloseTo(1);
    expect(wheelZoomAmount(3, 1)).toBeCloseTo(-0.48);
    expect(wheelZoomAmount(1, 2)).toBeCloseTo(-8);
  });

  it('moves one step per mouse notch however large the delta', () => {
    const steps = wheelStepper();
    expect(steps.push(wheelZoomAmount(-100, 0), 0, false)).toBe(1);
    expect(steps.push(wheelZoomAmount(-400, 0), WHEEL_NOTCH_MS, false)).toBe(1);
    expect(steps.push(wheelZoomAmount(120, 0), 2 * WHEEL_NOTCH_MS, false)).toBe(-1);
  });

  it('steps a free-spinning wheel or Cmd momentum at most once per notch interval', () => {
    const steps = wheelStepper();
    const results = Array.from({ length: 40 }, (_, frame) =>
      steps.push(wheelZoomAmount(-100, 0), frame * 10, false)
    );
    expect(results.filter((step) => step !== 0)).toEqual([1, 1, 1]);
    expect(results.indexOf(1, 1)).toBe(WHEEL_NOTCH_MS / 10);
  });

  it('adds small trackpad deltas until they reach a step and repeats within one pinch', () => {
    const steps = wheelStepper();
    const results = Array.from({ length: 12 }, (_, frame) =>
      steps.push(wheelZoomAmount(-5, 0), frame * 16, true)
    );
    expect(results.filter((step) => step !== 0)).toEqual([1, 1]);
  });

  it('lets a fast pinch stream step on consecutive events', () => {
    const steps = wheelStepper();
    const results = Array.from({ length: 4 }, (_, frame) =>
      steps.push(wheelZoomAmount(-30, 0), frame * 16, true)
    );
    expect(results).toEqual([1, 1, 1, 1]);
  });

  it('drops a partial step after a pause or a direction change', () => {
    const steps = wheelStepper();
    expect(steps.push(wheelZoomAmount(-15, 0), 0, true)).toBe(0);
    expect(steps.push(wheelZoomAmount(-15, 0), WHEEL_IDLE_MS + 1, true)).toBe(0);
    expect(steps.push(wheelZoomAmount(15, 0), WHEEL_IDLE_MS + 17, true)).toBe(0);
    expect(steps.push(wheelZoomAmount(10, 0), WHEEL_IDLE_MS + 33, true)).toBe(-1);
    expect(steps.push(0, WHEEL_IDLE_MS + 49, true)).toBe(0);
  });
});

describe('pinch zoom steps', () => {
  it('steps each time the scale moves one ratio from the last step', () => {
    const steps = scaleStepper();
    steps.start();
    expect(steps.to(1.2)).toBe(0);
    expect(steps.to(ZOOM_STEP_RATIO + 0.01)).toBe(1);
    expect(steps.to(1.5)).toBe(0);
    expect(steps.to(ZOOM_STEP_RATIO ** 3 + 0.01)).toBe(2);
    expect(steps.to(0.99)).toBe(-3);
    expect(steps.to(0.7)).toBe(-1);
  });

  it('starts over for each gesture and ignores a degenerate scale', () => {
    const steps = scaleStepper();
    steps.start();
    expect(steps.to(2)).toBe(3);
    steps.start();
    expect(steps.to(1.3)).toBe(1);
    expect(steps.to(0)).toBe(0);
    expect(steps.to(Number.NaN)).toBe(0);
  });
});

describe('zoom anchor', () => {
  const before = grid(150, 4, 20);
  const after = grid(320, 2, 40);

  it('finds the photo under a point, or the nearest one in the gaps and margins', () => {
    expect(assetIndexAt(before, { x: 10, y: 10 })).toBe(0);
    expect(assetIndexAt(before, { x: 150, y: 200 })).toBe(5);
    expect(assetIndexAt(before, { x: 102, y: 152 })).toBe(5);
    expect(assetIndexAt(before, { x: 900, y: 10 })).toBe(3);
    expect(assetIndexAt(before, { x: 10, y: 99_999 })).toBe(76);
    expect(assetIndexAt({ boxes: [], rows: [], height: 0 }, { x: 0, y: 0 })).toBe(-1);
  });

  it('keeps the same point of the anchored photo at the same place in the viewport', () => {
    const viewTop = 600;
    const anchor = zoomAnchor(before, {
      point: { x: 150, y: 700 },
      activeIndex: -1,
      viewTop,
      viewHeight: 800
    });
    expect(anchor).toEqual({ index: 17, ratio: 84 / 150, offset: 100 });
    if (!anchor) return;

    const top = anchoredViewTop(after, anchor);
    const box = after.boxes[17];
    expect(box).toBeDefined();
    if (top === null || !box) return;
    expect(box.top + anchor.ratio * box.height - top).toBeCloseTo(100);
  });

  it('anchors on the active photo when it is on screen', () => {
    const anchor = zoomAnchor(before, { point: null, activeIndex: 9, viewTop: 0, viewHeight: 800 });
    expect(anchor).toEqual({ index: 9, ratio: 0.5, offset: 308 + 75 });
  });

  it('keeps the top row in place when the active photo is off screen or missing', () => {
    expect(
      zoomAnchor(before, { point: null, activeIndex: 70, viewTop: 0, viewHeight: 800 })
    ).toEqual({ index: 0, ratio: 0, offset: 0 });
    const anchor = zoomAnchor(before, {
      point: null,
      activeIndex: -1,
      viewTop: 154 + 75,
      viewHeight: 800
    });
    expect(anchor).toEqual({ index: 4, ratio: 0.5, offset: 0 });
    if (!anchor) return;
    expect(anchoredViewTop(after, anchor)).toBe(2 * 324 + 160);
  });

  it('gives no anchor or position for a photo that is not in the layout', () => {
    expect(gridAnchor(before, 999, 0, 0)).toBeNull();
    expect(anchoredViewTop(before, { index: 999, ratio: 0, offset: 0 })).toBeNull();
  });
});
