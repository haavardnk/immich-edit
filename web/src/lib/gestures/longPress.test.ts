import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { gestureArbiter } from '#lib/gestures/arbiter.ts';
import { longPressRecognizer } from '#lib/gestures/longPress.ts';
import type { Point } from '#lib/gestures/point.ts';
import type { PointerSample } from '#lib/gestures/pointers.ts';

function at(clientX: number, clientY: number, timeStamp: number): PointerSample {
  return { pointerId: 1, pointerType: 'touch', clientX, clientY, timeStamp };
}

function setup(accepts?: (point: Point) => boolean) {
  const spec = { accepts, start: vi.fn(), move: vi.fn(), end: vi.fn(), cancel: vi.fn() };
  const recognizer = longPressRecognizer(spec);
  const arbiter = gestureArbiter([recognizer]);
  return { spec, recognizer, arbiter };
}

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

describe('longPressRecognizer', () => {
  it('claims and starts after the hold time', () => {
    const { spec, recognizer, arbiter } = setup();
    arbiter.down(at(20, 30, 0));
    vi.advanceTimersByTime(499);
    expect(spec.start).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(spec.start).toHaveBeenCalledExactlyOnceWith({ x: 20, y: 30 });
    expect(arbiter.locked()).toBe(recognizer);
  });

  it('leaves the gesture unclaimed when the held point is rejected', () => {
    const accepts = vi.fn(() => false);
    const { spec, arbiter } = setup(accepts);
    arbiter.down(at(20, 30, 0));
    vi.advanceTimersByTime(500);
    expect(accepts).toHaveBeenCalledExactlyOnceWith({ x: 20, y: 30 });
    expect(spec.start).not.toHaveBeenCalled();
    expect(arbiter.locked()).toBeNull();
    arbiter.move(at(20, 120, 600));
    arbiter.up(at(20, 120, 700));
    expect(spec.move).not.toHaveBeenCalled();
    expect(spec.end).not.toHaveBeenCalled();
  });

  it('reports moves while held and ends on release', () => {
    const { spec, arbiter } = setup();
    arbiter.down(at(0, 0, 0));
    vi.advanceTimersByTime(500);
    arbiter.move(at(80, 5, 600));
    arbiter.move(at(160, 10, 700));
    arbiter.up(at(170, 10, 800));
    expect(spec.move.mock.calls).toEqual([[{ x: 80, y: 5 }], [{ x: 160, y: 10 }]]);
    expect(spec.end).toHaveBeenCalledExactlyOnceWith({ x: 170, y: 10 });
    expect(spec.cancel).not.toHaveBeenCalled();
    expect(arbiter.locked()).toBeNull();
  });

  it('starts at the latest point within the slop', () => {
    const { spec, arbiter } = setup();
    arbiter.down(at(0, 0, 0));
    arbiter.move(at(6, 8, 100));
    vi.advanceTimersByTime(500);
    expect(spec.start).toHaveBeenCalledWith({ x: 6, y: 8 });
  });

  it('gives up when travel passes the slop before the hold time', () => {
    const { spec, arbiter } = setup();
    arbiter.down(at(0, 0, 0));
    arbiter.move(at(11, 0, 100));
    arbiter.move(at(0, 0, 200));
    vi.advanceTimersByTime(1000);
    expect(spec.start).not.toHaveBeenCalled();
  });

  it('gives up on an early release', () => {
    const { spec, arbiter } = setup();
    arbiter.down(at(0, 0, 0));
    vi.advanceTimersByTime(400);
    arbiter.up(at(0, 0, 400));
    vi.advanceTimersByTime(1000);
    expect(spec.start).not.toHaveBeenCalled();
    expect(spec.end).not.toHaveBeenCalled();
  });

  it('reports a cancel after it started', () => {
    const { spec, arbiter } = setup();
    arbiter.down(at(0, 0, 0));
    vi.advanceTimersByTime(500);
    arbiter.cancel(1);
    expect(spec.cancel).toHaveBeenCalledOnce();
    expect(spec.end).not.toHaveBeenCalled();
  });

  it('stays silent when cancelled before it started', () => {
    const { spec, arbiter } = setup();
    arbiter.down(at(0, 0, 0));
    arbiter.cancel(1);
    vi.advanceTimersByTime(1000);
    expect(spec.start).not.toHaveBeenCalled();
    expect(spec.cancel).not.toHaveBeenCalled();
  });
});
