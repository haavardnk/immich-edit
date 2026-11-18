import { describe, expect, it, vi } from 'vitest';
import { gestureArbiter } from '#lib/gestures/arbiter.ts';
import { panRecognizer, type PanSpec } from '#lib/gestures/pan.ts';
import type { PointerSample } from '#lib/gestures/pointers.ts';

function at(clientX: number, clientY: number, timeStamp: number): PointerSample {
  return { pointerId: 1, pointerType: 'touch', clientX, clientY, timeStamp };
}

function setup(options: PanSpec = {}) {
  const spec = { start: vi.fn(), move: vi.fn(), end: vi.fn(), cancel: vi.fn() };
  const recognizer = panRecognizer({ ...spec, ...options });
  const arbiter = gestureArbiter([recognizer]);
  return { spec, recognizer, arbiter };
}

describe('panRecognizer', () => {
  it('claims only once travel passes the slop', () => {
    const { spec, recognizer, arbiter } = setup();
    arbiter.down(at(100, 100, 0));
    arbiter.move(at(106, 108, 16));
    expect(spec.start).not.toHaveBeenCalled();
    arbiter.move(at(107, 108, 32));
    expect(spec.start).toHaveBeenCalledExactlyOnceWith({
      x: 107,
      y: 108,
      dx: 7,
      dy: 8,
      stepX: 7,
      stepY: 8,
      axis: null
    });
    expect(arbiter.locked()).toBe(recognizer);
  });

  it('reports the delta from start and from the last move', () => {
    const { spec, arbiter } = setup();
    arbiter.down(at(0, 0, 0));
    arbiter.move(at(20, 0, 16));
    arbiter.move(at(30, -5, 32));
    expect(spec.move).toHaveBeenLastCalledWith({
      x: 30,
      y: -5,
      dx: 30,
      dy: -5,
      stepX: 10,
      stepY: -5,
      axis: null
    });
  });

  it('ends with the release velocity', () => {
    const { spec, arbiter } = setup();
    arbiter.down(at(0, 0, 0));
    arbiter.move(at(20, 0, 20));
    arbiter.move(at(60, 10, 40));
    arbiter.up(at(100, 20, 60));
    expect(spec.end).toHaveBeenCalledExactlyOnceWith(
      expect.objectContaining({ dx: 100, dy: 20, stepX: 40, stepY: 10, vx: 100 / 60, vy: 20 / 60 })
    );
    expect(arbiter.locked()).toBeNull();
  });

  it('locks to the dominant horizontal axis at claim time', () => {
    const { spec, arbiter } = setup({ lockAxis: true });
    arbiter.down(at(0, 0, 0));
    arbiter.move(at(12, 5, 16));
    arbiter.move(at(30, 40, 32));
    arbiter.up(at(40, 80, 48));
    expect(spec.start).toHaveBeenCalledWith(expect.objectContaining({ dx: 12, dy: 0, axis: 'x' }));
    expect(spec.move).toHaveBeenCalledWith(
      expect.objectContaining({ dx: 30, dy: 0, stepY: 0, axis: 'x' })
    );
    expect(spec.end).toHaveBeenCalledWith(expect.objectContaining({ dx: 40, dy: 0, vy: 0 }));
  });

  it('locks to the dominant vertical axis at claim time', () => {
    const { spec, arbiter } = setup({ lockAxis: true });
    arbiter.down(at(0, 0, 0));
    arbiter.move(at(4, -11, 16));
    arbiter.move(at(50, -20, 32));
    expect(spec.move).toHaveBeenCalledWith(
      expect.objectContaining({ dx: 0, dy: -20, stepX: 0, axis: 'y' })
    );
  });

  it('reports a cancel only after it started', () => {
    const { spec, arbiter } = setup();
    arbiter.down(at(0, 0, 0));
    arbiter.cancel(1);
    expect(spec.cancel).not.toHaveBeenCalled();
    arbiter.down(at(0, 0, 100));
    arbiter.move(at(20, 0, 116));
    arbiter.cancel(1);
    expect(spec.cancel).toHaveBeenCalledOnce();
    expect(spec.end).not.toHaveBeenCalled();
  });

  it('never ends a pan that did not start', () => {
    const { spec, arbiter } = setup();
    arbiter.down(at(0, 0, 0));
    arbiter.up(at(5, 0, 16));
    expect(spec.end).not.toHaveBeenCalled();
  });
});
