import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { gestureArbiter } from '#lib/gestures/arbiter.ts';
import type { PointerSample } from '#lib/gestures/pointers.ts';
import { tapRecognizer, type TapSpec } from '#lib/gestures/tap.ts';

function at(clientX: number, clientY: number, timeStamp: number, pointerId = 1): PointerSample {
  return { pointerId, pointerType: 'touch', clientX, clientY, timeStamp };
}

function setup(spec: TapSpec = {}) {
  const tap = vi.fn();
  const arbiter = gestureArbiter([tapRecognizer({ tap, ...spec })]);
  function press(x: number, y: number, time: number, travel = 0, pointerId = 1): void {
    arbiter.down(at(x, y, time, pointerId));
    if (travel) arbiter.move(at(x + travel, y, time + 10, pointerId));
    arbiter.up(at(x + travel, y, time + 50, pointerId));
  }
  return { arbiter, tap, press };
}

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

describe('tapRecognizer', () => {
  it('fires a tap at once at the press point without a double-tap handler', () => {
    const { tap, press } = setup();
    press(40, 50, 0, 10);
    expect(tap).toHaveBeenCalledExactlyOnceWith({ x: 40, y: 50 });
  });

  it('cancels when travel passes the slop', () => {
    const { tap, press } = setup();
    press(0, 0, 0, 11);
    expect(tap).not.toHaveBeenCalled();
  });

  it('cancels when the pointer strays and returns', () => {
    const { arbiter, tap } = setup();
    arbiter.down(at(0, 0, 0));
    arbiter.move(at(20, 0, 10));
    arbiter.up(at(0, 0, 20));
    expect(tap).not.toHaveBeenCalled();
  });

  it('honours a slop override', () => {
    const { tap, press } = setup({ thresholds: { tapSlop: 3 } });
    press(0, 0, 0, 4);
    expect(tap).not.toHaveBeenCalled();
  });

  it('delays a single tap by the double-tap window', () => {
    const doubleTap = vi.fn();
    const { tap, press } = setup({ doubleTap });
    press(10, 10, 0);
    vi.advanceTimersByTime(299);
    expect(tap).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(tap).toHaveBeenCalledExactlyOnceWith({ x: 10, y: 10 });
    expect(doubleTap).not.toHaveBeenCalled();
  });

  it('fires a double tap and drops the single tap', () => {
    const doubleTap = vi.fn();
    const { tap, press } = setup({ doubleTap });
    press(10, 10, 0);
    press(35, 25, 300);
    vi.advanceTimersByTime(1000);
    expect(doubleTap).toHaveBeenCalledExactlyOnceWith({ x: 35, y: 25 });
    expect(tap).not.toHaveBeenCalled();
  });

  it('treats a second tap beyond the slop as a new tap', () => {
    const doubleTap = vi.fn();
    const { tap, press } = setup({ doubleTap });
    press(0, 0, 0);
    press(31, 0, 100);
    expect(tap).toHaveBeenCalledExactlyOnceWith({ x: 0, y: 0 });
    vi.advanceTimersByTime(300);
    expect(tap).toHaveBeenLastCalledWith({ x: 31, y: 0 });
    expect(doubleTap).not.toHaveBeenCalled();
  });

  it('treats a late second tap as a new tap', () => {
    const doubleTap = vi.fn();
    const { tap, press } = setup({ doubleTap, thresholds: { doubleTapMs: 300 } });
    press(0, 0, 0);
    vi.advanceTimersByTime(200);
    press(0, 0, 351);
    expect(tap).toHaveBeenCalledTimes(1);
    vi.advanceTimersByTime(300);
    expect(tap).toHaveBeenCalledTimes(2);
    expect(doubleTap).not.toHaveBeenCalled();
  });

  it('drops both taps when the second press travels', () => {
    const doubleTap = vi.fn();
    const { tap, press } = setup({ doubleTap });
    press(0, 0, 0);
    press(0, 0, 100, 20);
    vi.advanceTimersByTime(1000);
    expect(tap).not.toHaveBeenCalled();
    expect(doubleTap).not.toHaveBeenCalled();
  });

  it('drops a pending tap on reset', () => {
    const doubleTap = vi.fn();
    const { arbiter, tap, press } = setup({ doubleTap });
    press(0, 0, 0);
    arbiter.reset();
    vi.advanceTimersByTime(1000);
    expect(tap).not.toHaveBeenCalled();
  });
});
