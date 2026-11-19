import { gestureArbiter } from '#lib/gestures/arbiter.ts';
import { panRecognizer } from '#lib/gestures/pan.ts';
import { pinchRecognizer } from '#lib/gestures/pinch.ts';
import type { PointerSample } from '#lib/gestures/pointers.ts';
import { twoFingerTapRecognizer } from '#lib/gestures/twoFingerTap.ts';
import { describe, expect, it, vi } from 'vitest';

function at(pointerId: number, clientX: number, clientY: number, timeStamp = 0): PointerSample {
  return { pointerId, pointerType: 'touch', clientX, clientY, timeStamp };
}

function setup() {
  const tap = vi.fn();
  const pinch = { start: vi.fn() };
  const pan = { start: vi.fn() };
  const arbiter = gestureArbiter([
    twoFingerTapRecognizer({ tap }),
    pinchRecognizer(pinch),
    panRecognizer(pan)
  ]);
  arbiter.down(at(1, 100, 100));
  arbiter.down(at(2, 200, 120, 20));
  return { tap, pinch, pan, arbiter };
}

describe('twoFingerTapRecognizer', () => {
  it('taps at the midpoint once both fingers lift without moving', () => {
    const { tap, arbiter } = setup();
    arbiter.move(at(1, 104, 102, 60));
    arbiter.up(at(1, 104, 102, 120));
    expect(tap).not.toHaveBeenCalled();
    arbiter.up(at(2, 200, 120, 150));
    expect(tap).toHaveBeenCalledExactlyOnceWith({ x: 150, y: 110 });
  });

  it('does not tap when a finger travels past the slop', () => {
    const { tap, arbiter } = setup();
    arbiter.move(at(2, 230, 120, 60));
    arbiter.up(at(1, 100, 100, 120));
    arbiter.up(at(2, 230, 120, 150));
    expect(tap).not.toHaveBeenCalled();
  });

  it('does not tap when the fingers rest longer than a long press', () => {
    const { tap, arbiter } = setup();
    arbiter.up(at(1, 100, 100, 480));
    arbiter.up(at(2, 200, 120, 520));
    expect(tap).not.toHaveBeenCalled();
  });

  it('yields to a pinch that spreads the fingers', () => {
    const { tap, pinch, arbiter } = setup();
    arbiter.move(at(2, 300, 120, 60));
    expect(pinch.start).toHaveBeenCalledOnce();
    arbiter.up(at(1, 100, 100, 120));
    arbiter.up(at(2, 300, 120, 150));
    expect(tap).not.toHaveBeenCalled();
  });

  it('lets the last finger pan once a stray finger has lifted', () => {
    const { tap, pan, arbiter } = setup();
    arbiter.move(at(1, 100, 115, 60));
    arbiter.up(at(1, 100, 115, 120));
    arbiter.move(at(2, 200, 160, 200));
    expect(pan.start).toHaveBeenCalledOnce();
    arbiter.up(at(2, 200, 160, 240));
    expect(tap).not.toHaveBeenCalled();
  });
});
