import { describe, expect, it } from 'vitest';
import {
  CONTENT_CAP,
  RUBBER_EXTENT,
  dragFrame,
  nextDetent,
  resolveDetents,
  rubberBand,
  snapSheet,
  type SnapInput
} from '#lib/components/sheet/sheetSnap.ts';

function snap(input: Partial<SnapInput> & Pick<SnapInput, 'height'>): ReturnType<typeof snapSheet> {
  return snapSheet({ heights: [200, 400, 700], velocity: 0, dismissable: true, ...input });
}

describe('resolveDetents', () => {
  it('keeps pixel detents and caps them at the available height', () => {
    expect(resolveDetents([120, 900], { available: 800, content: 0 })).toEqual([120, 800]);
  });

  it('scales fractions by the available height', () => {
    expect(
      resolveDetents([{ fraction: 0.5 }, { fraction: 1 }], { available: 800, content: 0 })
    ).toEqual([400, 800]);
  });

  it('clamps fractions to the unit range', () => {
    expect(
      resolveDetents([{ fraction: -1 }, { fraction: 2 }], { available: 500, content: 0 })
    ).toEqual([0, 500]);
  });

  it('sizes content detents to the content, capped below the full height', () => {
    expect(resolveDetents(['content'], { available: 1000, content: 300 })).toEqual([300]);
    expect(resolveDetents(['content'], { available: 1000, content: 2000 })).toEqual([
      1000 * CONTENT_CAP
    ]);
  });

  it('never resolves below zero', () => {
    expect(resolveDetents([-50, 'content'], { available: -10, content: -5 })).toEqual([0, 0]);
  });
});

describe('rubberBand', () => {
  it('is zero without overshoot', () => {
    expect(rubberBand(0)).toBe(0);
    expect(rubberBand(-40)).toBe(0);
  });

  it('grows slower than the overshoot and never reaches the extent', () => {
    const small = rubberBand(20);
    const large = rubberBand(10_000);
    expect(small).toBeGreaterThan(0);
    expect(small).toBeLessThan(20);
    expect(large).toBeGreaterThan(small);
    expect(large).toBeLessThan(RUBBER_EXTENT);
  });
});

describe('dragFrame', () => {
  const heights = [200, 600];

  it('follows the finger between detents', () => {
    expect(dragFrame(450, heights, true)).toEqual({ height: 450, offset: 0 });
  });

  it('resists above the top detent', () => {
    const frame = dragFrame(700, heights, true);
    expect(frame.offset).toBe(0);
    expect(frame.height).toBeGreaterThan(600);
    expect(frame.height).toBeLessThan(700);
  });

  it('slides the lowest detent down when dismissable', () => {
    expect(dragFrame(150, heights, true)).toEqual({ height: 200, offset: 50 });
  });

  it('resists below the lowest detent when not dismissable', () => {
    const frame = dragFrame(100, heights, false);
    expect(frame.height).toBe(200);
    expect(frame.offset).toBeGreaterThan(0);
    expect(frame.offset).toBeLessThan(100);
  });

  it('passes the raw height through without detents', () => {
    expect(dragFrame(-20, [], true)).toEqual({ height: 0, offset: 0 });
  });
});

describe('snapSheet', () => {
  it('settles on the nearest detent without a fling', () => {
    expect(snap({ height: 280 })).toBe(0);
    expect(snap({ height: 320 })).toBe(1);
    expect(snap({ height: 760 })).toBe(2);
  });

  it('moves one detent down on a downward fling', () => {
    expect(snap({ height: 690, velocity: 0.8 })).toBe(1);
    expect(snap({ height: 390, velocity: 0.8 })).toBe(0);
  });

  it('moves one detent up on an upward fling', () => {
    expect(snap({ height: 210, velocity: -0.8 })).toBe(1);
    expect(snap({ height: 420, velocity: -0.8 })).toBe(2);
  });

  it('stays on the top detent when flung up past it', () => {
    expect(snap({ height: 720, velocity: -2 })).toBe(2);
  });

  it('dismisses on a downward fling below the lowest detent', () => {
    expect(snap({ height: 195, velocity: 0.8 })).toBe('dismiss');
  });

  it('falls back to the lowest detent when a fling cannot dismiss', () => {
    expect(snap({ height: 195, velocity: 0.8, dismissable: false })).toBe(0);
  });

  it('dismisses after dragging well below the lowest detent', () => {
    expect(snap({ height: 110 })).toBe('dismiss');
    expect(snap({ height: 150 })).toBe(0);
    expect(snap({ height: 40, dismissable: false })).toBe(0);
  });

  it('honours custom fling and dismiss thresholds', () => {
    expect(snap({ height: 690, velocity: 0.8, flingVelocity: 1 })).toBe(2);
    expect(snap({ height: 150, dismissFraction: 0.2 })).toBe('dismiss');
  });

  it('keeps the caller order when detents are unsorted', () => {
    expect(snapSheet({ heights: [700, 200], height: 650, velocity: 0, dismissable: false })).toBe(
      0
    );
    expect(snapSheet({ heights: [700, 200], height: 650, velocity: 0.8, dismissable: false })).toBe(
      1
    );
  });

  it('dismisses or stays put without detents', () => {
    expect(snapSheet({ heights: [], height: 0, velocity: 0, dismissable: true })).toBe('dismiss');
    expect(snapSheet({ heights: [], height: 0, velocity: 0, dismissable: false })).toBe(0);
  });
});

describe('nextDetent', () => {
  it('steps forward and wraps to the first detent', () => {
    expect(nextDetent(0, 3)).toBe(1);
    expect(nextDetent(2, 3)).toBe(0);
    expect(nextDetent(0, 0)).toBe(0);
  });
});
