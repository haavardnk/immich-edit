import { describe, expect, it, vi } from 'vitest';
import { gestureArbiter } from '#lib/gestures/arbiter.ts';
import type { PointerSample } from '#lib/gestures/pointers.ts';
import { swipeRecognizer, type SwipeSpec } from '#lib/gestures/swipe.ts';

function at(clientX: number, clientY: number, timeStamp: number): PointerSample {
  return { pointerId: 1, pointerType: 'touch', clientX, clientY, timeStamp };
}

function setup(options: SwipeSpec = {}) {
  const spec = { start: vi.fn(), move: vi.fn(), end: vi.fn(), cancel: vi.fn() };
  const recognizer = swipeRecognizer({ ...spec, ...options });
  const arbiter = gestureArbiter([recognizer]);
  return { spec, recognizer, arbiter };
}

function slowDrag(arbiter: ReturnType<typeof setup>['arbiter'], dx: number, dy: number): void {
  arbiter.down(at(200, 200, 0));
  arbiter.move(at(200 + dx / 2, 200 + dy / 2, 200));
  arbiter.move(at(200 + dx, 200 + dy, 400));
  arbiter.up(at(200 + dx, 200 + dy, 600));
}

describe('swipeRecognizer', () => {
  it('claims past the slop along a dominant axis', () => {
    const { spec, recognizer, arbiter } = setup();
    arbiter.down(at(0, 0, 0));
    arbiter.move(at(-10, 0, 16));
    expect(spec.start).not.toHaveBeenCalled();
    arbiter.move(at(-11, 3, 32));
    expect(spec.start).toHaveBeenCalledExactlyOnceWith({ dx: -11, dy: 3, axis: 'x' });
    expect(arbiter.locked()).toBe(recognizer);
  });

  it('waits while the cross axis is more than half the main axis', () => {
    const { spec, arbiter } = setup();
    arbiter.down(at(0, 0, 0));
    arbiter.move(at(20, 11, 16));
    expect(spec.start).not.toHaveBeenCalled();
    arbiter.move(at(22, 11, 32));
    expect(spec.start).toHaveBeenCalledOnce();
  });

  it('only claims an allowed direction', () => {
    const { spec, arbiter } = setup({ directions: ['down'] });
    arbiter.down(at(0, 0, 0));
    arbiter.move(at(-40, 0, 16));
    arbiter.move(at(0, -40, 32));
    expect(spec.start).not.toHaveBeenCalled();
    arbiter.move(at(0, 40, 48));
    expect(spec.start).toHaveBeenCalledWith({ dx: 0, dy: 40, axis: 'y' });
  });

  it('reports progress so the photo can follow the finger', () => {
    const { spec, arbiter } = setup();
    arbiter.down(at(0, 0, 0));
    arbiter.move(at(20, 2, 16));
    arbiter.move(at(45, 6, 32));
    expect(spec.move).toHaveBeenCalledWith({ dx: 45, dy: 6, axis: 'x' });
  });

  it('commits a slow swipe past the minimum distance', () => {
    const { spec, arbiter } = setup();
    slowDrag(arbiter, -60, 0);
    expect(spec.end).toHaveBeenCalledExactlyOnceWith({
      direction: 'left',
      dx: -60,
      dy: 0,
      velocity: 0
    });
    expect(arbiter.locked()).toBeNull();
  });

  it('snaps back a slow swipe short of the minimum distance', () => {
    const { spec, arbiter } = setup();
    slowDrag(arbiter, 0, 59);
    expect(spec.end).toHaveBeenCalledWith(expect.objectContaining({ direction: null, dy: 59 }));
  });

  it('commits a fast release past half the minimum distance', () => {
    const { spec, arbiter } = setup();
    arbiter.down(at(0, 0, 0));
    arbiter.move(at(0, -15, 20));
    arbiter.up(at(0, -30, 40));
    expect(spec.end).toHaveBeenCalledWith({ direction: 'up', dx: 0, dy: -30, velocity: -0.75 });
  });

  it('snaps back a fast release short of half the minimum distance', () => {
    const { spec, arbiter } = setup();
    arbiter.down(at(0, 0, 0));
    arbiter.move(at(15, 0, 20));
    arbiter.up(at(29, 0, 40));
    expect(spec.end).toHaveBeenCalledWith(expect.objectContaining({ direction: null }));
  });

  it('snaps back when flung against the swipe', () => {
    const { spec, arbiter } = setup();
    arbiter.down(at(0, 0, 0));
    arbiter.move(at(120, 0, 200));
    arbiter.move(at(100, 0, 360));
    arbiter.up(at(80, 0, 380));
    expect(spec.end).toHaveBeenCalledWith(expect.objectContaining({ direction: null, dx: 80 }));
  });

  it('snaps back when released in a direction that is not allowed', () => {
    const { spec, arbiter } = setup({ directions: ['left'] });
    arbiter.down(at(0, 0, 0));
    arbiter.move(at(-20, 0, 100));
    arbiter.move(at(80, 0, 300));
    arbiter.up(at(80, 0, 500));
    expect(spec.end).toHaveBeenCalledWith(expect.objectContaining({ direction: null, dx: 80 }));
  });

  it('reports a cancel after it claimed', () => {
    const { spec, arbiter } = setup();
    arbiter.down(at(0, 0, 0));
    arbiter.move(at(30, 0, 16));
    arbiter.cancel(1);
    expect(spec.cancel).toHaveBeenCalledOnce();
    expect(spec.end).not.toHaveBeenCalled();
  });
});
