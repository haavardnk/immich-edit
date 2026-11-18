import { describe, expect, it, vi } from 'vitest';
import { gestureArbiter } from '#lib/gestures/arbiter.ts';
import { pinchRecognizer } from '#lib/gestures/pinch.ts';
import type { PointerSample } from '#lib/gestures/pointers.ts';

function at(pointerId: number, clientX: number, clientY: number, timeStamp = 0): PointerSample {
  return { pointerId, pointerType: 'touch', clientX, clientY, timeStamp };
}

function setup() {
  const spec = { start: vi.fn(), move: vi.fn(), end: vi.fn(), cancel: vi.fn() };
  const recognizer = pinchRecognizer(spec);
  const arbiter = gestureArbiter([recognizer]);
  arbiter.down(at(1, 100, 100));
  arbiter.down(at(2, 200, 100));
  return { spec, recognizer, arbiter };
}

describe('pinchRecognizer', () => {
  it('waits until the scale change passes the slop', () => {
    const { spec, recognizer, arbiter } = setup();
    arbiter.move(at(2, 204, 100));
    expect(spec.start).not.toHaveBeenCalled();
    arbiter.move(at(2, 206, 100));
    expect(spec.start).toHaveBeenCalledExactlyOnceWith({
      scale: 1.06,
      x: 153,
      y: 100,
      dx: 3,
      dy: 0
    });
    expect(arbiter.locked()).toBe(recognizer);
  });

  it('claims a two-finger pan once the midpoint passes the slop', () => {
    const { spec, arbiter } = setup();
    arbiter.move(at(1, 100, 110));
    arbiter.move(at(2, 200, 110));
    expect(spec.start).not.toHaveBeenCalled();
    arbiter.move(at(2, 200, 112));
    expect(spec.start).toHaveBeenCalledWith({
      scale: expect.closeTo(1, 3),
      x: 150,
      y: 111,
      dx: 0,
      dy: 11
    });
  });

  it('reports scale and focal point relative to the start', () => {
    const { spec, arbiter } = setup();
    arbiter.move(at(1, 50, 100));
    arbiter.move(at(2, 250, 100));
    expect(spec.move).toHaveBeenLastCalledWith({ scale: 2, x: 150, y: 100, dx: 0, dy: 0 });
    arbiter.move(at(1, 150, 140));
    arbiter.move(at(2, 250, 140));
    expect(spec.move).toHaveBeenLastCalledWith({ scale: 1, x: 200, y: 140, dx: 50, dy: 40 });
  });

  it('ends with the last update when a finger lifts', () => {
    const { spec, arbiter } = setup();
    arbiter.move(at(2, 300, 100));
    arbiter.up(at(1, 0, 0));
    expect(spec.end).toHaveBeenCalledExactlyOnceWith({
      scale: 2,
      x: 200,
      y: 100,
      dx: 50,
      dy: 0
    });
    expect(arbiter.locked()).toBeNull();
  });

  it('ignores a single pointer', () => {
    const spec = { start: vi.fn() };
    const arbiter = gestureArbiter([pinchRecognizer(spec)]);
    arbiter.down(at(1, 0, 0));
    arbiter.move(at(1, 100, 100));
    expect(spec.start).not.toHaveBeenCalled();
  });

  it('reports a cancel after it claimed', () => {
    const { spec, arbiter } = setup();
    arbiter.move(at(2, 300, 100));
    arbiter.cancel(2);
    expect(spec.cancel).toHaveBeenCalledOnce();
    expect(spec.end).not.toHaveBeenCalled();
  });
});
