import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { gestureArbiter } from '#lib/gestures/arbiter.ts';
import { longPressRecognizer } from '#lib/gestures/longPress.ts';
import { panRecognizer } from '#lib/gestures/pan.ts';
import { pinchRecognizer } from '#lib/gestures/pinch.ts';
import type { PointerKind, PointerSample } from '#lib/gestures/pointers.ts';
import { swipeRecognizer } from '#lib/gestures/swipe.ts';
import { tapRecognizer } from '#lib/gestures/tap.ts';

function at(
  pointerId: number,
  clientX: number,
  clientY: number,
  timeStamp = 0,
  pointerType: PointerKind = 'touch'
): PointerSample {
  return { pointerId, pointerType, clientX, clientY, timeStamp };
}

function handlers() {
  return { start: vi.fn(), move: vi.fn(), end: vi.fn(), cancel: vi.fn() };
}

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

describe('gestureArbiter', () => {
  it('locks a dominant horizontal drag to the swipe listed first', () => {
    const swipe = handlers();
    const pan = handlers();
    const arbiter = gestureArbiter([swipeRecognizer(swipe), panRecognizer(pan)]);
    arbiter.down(at(1, 0, 0));
    arbiter.move(at(1, 15, 1, 16));
    arbiter.move(at(1, 40, 30, 32));
    arbiter.up(at(1, 40, 30, 48));
    expect(swipe.start).toHaveBeenCalledOnce();
    expect(swipe.move).toHaveBeenCalledOnce();
    expect(swipe.end).toHaveBeenCalledOnce();
    expect(pan.start).not.toHaveBeenCalled();
    expect(pan.end).not.toHaveBeenCalled();
  });

  it('lets the pan claim a diagonal drag the swipe declines', () => {
    const swipe = handlers();
    const pan = handlers();
    const arbiter = gestureArbiter([swipeRecognizer(swipe), panRecognizer(pan)]);
    arbiter.down(at(1, 0, 0));
    arbiter.move(at(1, 15, 12, 16));
    arbiter.move(at(1, 80, 0, 32));
    expect(pan.start).toHaveBeenCalledOnce();
    expect(pan.move).toHaveBeenCalledOnce();
    expect(swipe.start).not.toHaveBeenCalled();
  });

  it('gives the lock to the first recognizer to cross its threshold', () => {
    const swipe = handlers();
    const pan = handlers();
    const arbiter = gestureArbiter([
      panRecognizer({ ...pan, thresholds: { panSlop: 20 } }),
      swipeRecognizer(swipe)
    ]);
    arbiter.down(at(1, 0, 0));
    arbiter.move(at(1, 15, 0, 16));
    arbiter.move(at(1, 30, 0, 32));
    expect(swipe.start).toHaveBeenCalledOnce();
    expect(swipe.move).toHaveBeenCalledOnce();
    expect(pan.start).not.toHaveBeenCalled();
  });

  it('pans when the finger moves before the long-press fires', () => {
    const longPress = handlers();
    const pan = handlers();
    const tap = vi.fn();
    const arbiter = gestureArbiter([
      longPressRecognizer(longPress),
      panRecognizer(pan),
      tapRecognizer({ tap })
    ]);
    arbiter.down(at(1, 0, 0));
    vi.advanceTimersByTime(100);
    arbiter.move(at(1, 15, 0, 100));
    vi.advanceTimersByTime(1000);
    arbiter.up(at(1, 15, 0, 1100));
    expect(pan.start).toHaveBeenCalledOnce();
    expect(pan.end).toHaveBeenCalledOnce();
    expect(longPress.start).not.toHaveBeenCalled();
    expect(tap).not.toHaveBeenCalled();
  });

  it('long-presses when the finger holds still', () => {
    const longPress = handlers();
    const pan = handlers();
    const tap = vi.fn();
    const arbiter = gestureArbiter([
      longPressRecognizer(longPress),
      panRecognizer(pan),
      tapRecognizer({ tap })
    ]);
    arbiter.down(at(1, 0, 0));
    vi.advanceTimersByTime(500);
    arbiter.move(at(1, 50, 0, 600));
    arbiter.up(at(1, 50, 0, 700));
    expect(longPress.start).toHaveBeenCalledOnce();
    expect(longPress.move).toHaveBeenCalledWith({ x: 50, y: 0 });
    expect(longPress.end).toHaveBeenCalledOnce();
    expect(pan.start).not.toHaveBeenCalled();
    expect(tap).not.toHaveBeenCalled();
  });

  it('taps when nothing claims the gesture', () => {
    const tap = vi.fn();
    const arbiter = gestureArbiter([
      longPressRecognizer(handlers()),
      panRecognizer(handlers()),
      tapRecognizer({ tap })
    ]);
    arbiter.down(at(1, 5, 5));
    arbiter.move(at(1, 8, 5, 50));
    arbiter.up(at(1, 8, 5, 100));
    expect(tap).toHaveBeenCalledExactlyOnceWith({ x: 5, y: 5 });
  });

  it('hands a locked swipe over to the pinch when a second finger lands', () => {
    const swipe = handlers();
    const pinch = handlers();
    const pinching = pinchRecognizer(pinch);
    const arbiter = gestureArbiter([swipeRecognizer(swipe), pinching]);
    arbiter.down(at(1, 100, 100));
    arbiter.move(at(1, 140, 100, 16));
    expect(swipe.start).toHaveBeenCalledOnce();
    arbiter.down(at(2, 240, 100, 32));
    expect(swipe.cancel).toHaveBeenCalledOnce();
    expect(arbiter.locked()).toBeNull();
    arbiter.move(at(2, 340, 100, 48));
    expect(pinch.start).toHaveBeenCalledWith(expect.objectContaining({ scale: 2 }));
    expect(arbiter.locked()).toBe(pinching);
    arbiter.up(at(2, 340, 100, 64));
    arbiter.up(at(1, 140, 100, 80));
    expect(swipe.end).not.toHaveBeenCalled();
    expect(pinch.end).toHaveBeenCalledOnce();
  });

  it('cancels a running long-press when a second finger lands', () => {
    const longPress = handlers();
    const arbiter = gestureArbiter([longPressRecognizer(longPress), pinchRecognizer(handlers())]);
    arbiter.down(at(1, 0, 0));
    vi.advanceTimersByTime(500);
    arbiter.down(at(2, 100, 0, 600));
    expect(longPress.start).toHaveBeenCalledOnce();
    expect(longPress.cancel).toHaveBeenCalledOnce();
    arbiter.up(at(1, 0, 0, 700));
    expect(longPress.end).not.toHaveBeenCalled();
  });

  it('gives a pinch both fingers in one move', () => {
    const pinch = handlers();
    const arbiter = gestureArbiter([pinchRecognizer(pinch)]);
    arbiter.down(at(1, 100, 100));
    arbiter.down(at(2, 200, 100));
    arbiter.moveMany([at(1, 50, 100, 16), at(2, 250, 100, 16)]);
    expect(pinch.start).toHaveBeenCalledExactlyOnceWith({
      scale: 2,
      x: 150,
      y: 100,
      dx: 0,
      dy: 0
    });
    expect(pinch.move).not.toHaveBeenCalled();
  });

  it('continues the remaining finger as a pan after a pinch, without a tap', () => {
    const tap = vi.fn();
    const pan = handlers();
    const pinch = handlers();
    const arbiter = gestureArbiter([
      tapRecognizer({ tap }),
      panRecognizer(pan),
      pinchRecognizer(pinch)
    ]);
    arbiter.down(at(1, 100, 100));
    arbiter.down(at(2, 200, 100));
    arbiter.move(at(2, 300, 100, 16));
    arbiter.move(at(1, 120, 100, 24));
    arbiter.up(at(2, 300, 100, 32));
    expect(pinch.end).toHaveBeenCalledOnce();
    arbiter.move(at(1, 128, 100, 48));
    expect(pan.start).not.toHaveBeenCalled();
    arbiter.move(at(1, 140, 100, 64));
    expect(pan.start).toHaveBeenCalledWith(expect.objectContaining({ dx: 20, dy: 0 }));
    arbiter.up(at(1, 140, 100, 80));
    expect(pan.end).toHaveBeenCalledOnce();
    expect(tap).not.toHaveBeenCalled();
    expect(arbiter.locked()).toBeNull();
  });

  it('suppresses tap, double tap and long-press on the finger left after a pinch', () => {
    const tap = vi.fn();
    const doubleTap = vi.fn();
    const longPress = handlers();
    const arbiter = gestureArbiter([
      tapRecognizer({ tap, doubleTap }),
      longPressRecognizer(longPress),
      pinchRecognizer(handlers())
    ]);
    arbiter.down(at(1, 100, 100));
    arbiter.up(at(1, 100, 100, 50));
    arbiter.down(at(1, 100, 100, 100));
    arbiter.down(at(2, 200, 100, 110));
    arbiter.move(at(2, 300, 100, 120));
    arbiter.up(at(2, 300, 100, 130));
    vi.advanceTimersByTime(1000);
    arbiter.up(at(1, 100, 100, 1130));
    vi.advanceTimersByTime(1000);
    expect(longPress.start).not.toHaveBeenCalled();
    expect(doubleTap).not.toHaveBeenCalled();
    expect(tap).not.toHaveBeenCalled();
    expect(arbiter.contenders()).toEqual([]);
  });

  it('emits nothing for a two-finger tap', () => {
    const tap = vi.fn();
    const pinch = handlers();
    const arbiter = gestureArbiter([tapRecognizer({ tap }), pinchRecognizer(pinch)]);
    arbiter.down(at(1, 100, 100));
    arbiter.down(at(2, 200, 100, 10));
    arbiter.up(at(2, 200, 100, 50));
    arbiter.up(at(1, 100, 100, 60));
    vi.advanceTimersByTime(1000);
    expect(tap).not.toHaveBeenCalled();
    expect(pinch.start).not.toHaveBeenCalled();
  });

  it('resumes the pinch when a finger lands again', () => {
    const pinch = handlers();
    const arbiter = gestureArbiter([pinchRecognizer(pinch)]);
    arbiter.down(at(1, 100, 100));
    arbiter.down(at(2, 200, 100));
    arbiter.move(at(2, 300, 100, 16));
    arbiter.up(at(2, 300, 100, 32));
    arbiter.down(at(3, 300, 100, 48));
    arbiter.move(at(3, 400, 100, 64));
    expect(pinch.start).toHaveBeenCalledTimes(2);
    expect(pinch.start).toHaveBeenLastCalledWith(expect.objectContaining({ scale: 1.5 }));
  });

  it('keeps a pinch through a third finger and regroups when a member lifts', () => {
    const pinch = handlers();
    const pinching = pinchRecognizer(pinch);
    const arbiter = gestureArbiter([pinching]);
    arbiter.down(at(1, 100, 100));
    arbiter.down(at(2, 200, 100));
    arbiter.move(at(2, 300, 100, 16));
    arbiter.down(at(3, 400, 100, 20));
    arbiter.up(at(3, 400, 100, 24));
    expect(arbiter.locked()).toBe(pinching);
    expect(pinch.end).not.toHaveBeenCalled();
    arbiter.down(at(4, 500, 100, 30));
    arbiter.up(at(1, 100, 100, 40));
    expect(pinch.end).toHaveBeenCalledOnce();
    expect(arbiter.locked()).toBeNull();
    expect(pinching.pointerIds()).toEqual([2, 4]);
    arbiter.move(at(4, 700, 100, 50));
    expect(pinch.start).toHaveBeenCalledTimes(2);
    expect(arbiter.locked()).toBe(pinching);
  });

  it('replaces a stale pointer when its id goes down again', () => {
    const pan = handlers();
    const arbiter = gestureArbiter([panRecognizer(pan)]);
    arbiter.down(at(1, 0, 0, 0, 'mouse'));
    arbiter.move(at(1, 30, 0, 16, 'mouse'));
    arbiter.down(at(1, 500, 500, 1000, 'mouse'));
    expect(pan.cancel).toHaveBeenCalledOnce();
    expect(arbiter.pointers.count()).toBe(1);
    expect(arbiter.pointers.first()?.start).toEqual({ x: 500, y: 500 });
    arbiter.move(at(1, 520, 500, 1016, 'mouse'));
    expect(pan.start).toHaveBeenCalledTimes(2);
    expect(pan.start).toHaveBeenLastCalledWith(expect.objectContaining({ dx: 20 }));
  });

  it('reports every owner change', () => {
    const onOwnerChange = vi.fn();
    const panning = panRecognizer(handlers());
    const arbiter = gestureArbiter([panning], { onOwnerChange });
    arbiter.down(at(1, 0, 0));
    arbiter.move(at(1, 30, 0, 16));
    arbiter.move(at(1, 40, 0, 32));
    arbiter.up(at(1, 40, 0, 48));
    expect(onOwnerChange.mock.calls).toEqual([[panning], [null]]);
  });

  it('ignores pointer types outside the arbiter filter entirely', () => {
    const pan = handlers();
    const arbiter = gestureArbiter([panRecognizer(pan)], { pointerTypes: ['pen'] });
    arbiter.down(at(1, 0, 0, 0, 'pen'));
    arbiter.move(at(1, 30, 0, 16, 'pen'));
    arbiter.down(at(2, 50, 50, 20));
    arbiter.move(at(1, 40, 0, 32, 'pen'));
    expect(arbiter.pointers.has(2)).toBe(false);
    expect(pan.cancel).not.toHaveBeenCalled();
    expect(pan.move).toHaveBeenCalledOnce();
  });

  it('skips disabled recognizers and re-checks at the next gesture', () => {
    const pan = handlers();
    let enabled = false;
    const arbiter = gestureArbiter([panRecognizer({ ...pan, enabled: () => enabled })]);
    arbiter.down(at(1, 0, 0));
    arbiter.move(at(1, 30, 0, 16));
    arbiter.up(at(1, 30, 0, 32));
    expect(pan.start).not.toHaveBeenCalled();
    enabled = true;
    arbiter.down(at(1, 0, 0, 100));
    arbiter.move(at(1, 30, 0, 116));
    expect(pan.start).toHaveBeenCalledOnce();
  });

  it('cancels the owner and consumes the gesture until every pointer lifts', () => {
    const pan = handlers();
    const pinch = handlers();
    const tap = vi.fn();
    const arbiter = gestureArbiter([
      panRecognizer(pan),
      tapRecognizer({ tap }),
      pinchRecognizer(pinch)
    ]);
    arbiter.down(at(1, 0, 0));
    arbiter.move(at(1, 30, 0, 16));
    arbiter.cancel(1);
    expect(pan.cancel).toHaveBeenCalledOnce();
    arbiter.down(at(2, 0, 0, 100));
    arbiter.down(at(3, 100, 0, 110));
    arbiter.cancel(3);
    arbiter.down(at(4, 100, 0, 120));
    arbiter.move(at(4, 300, 0, 136));
    arbiter.up(at(4, 300, 0, 150));
    arbiter.move(at(2, 60, 0, 160));
    arbiter.up(at(2, 60, 0, 170));
    expect(pinch.start).not.toHaveBeenCalled();
    expect(pan.start).toHaveBeenCalledOnce();
    expect(pan.end).not.toHaveBeenCalled();
    expect(tap).not.toHaveBeenCalled();
    expect(arbiter.pointers.count()).toBe(0);
  });

  it('cancels the owner and forgets every pointer on reset', () => {
    const pan = handlers();
    const arbiter = gestureArbiter([panRecognizer(pan)]);
    arbiter.down(at(1, 0, 0));
    arbiter.move(at(1, 30, 0, 16));
    arbiter.reset();
    arbiter.move(at(1, 60, 0, 32));
    expect(pan.cancel).toHaveBeenCalledOnce();
    expect(pan.move).not.toHaveBeenCalled();
    expect(arbiter.pointers.count()).toBe(0);
    expect(arbiter.locked()).toBeNull();
  });
});
