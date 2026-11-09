import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { gestures, type GesturesSpec } from '#lib/gestures/gestures.ts';
import { longPressRecognizer } from '#lib/gestures/longPress.ts';
import { panRecognizer } from '#lib/gestures/pan.ts';
import { pinchRecognizer } from '#lib/gestures/pinch.ts';
import type { Recognizer } from '#lib/gestures/recognizer.ts';

let frames: Map<number, FrameRequestCallback>;
let lastFrame: number;

function runFrame(): void {
  const callbacks = [...frames.values()];
  frames.clear();
  for (const callback of callbacks) callback(0);
}

beforeEach(() => {
  frames = new Map();
  lastFrame = 0;
  vi.stubGlobal(
    'requestAnimationFrame',
    vi.fn((callback: FrameRequestCallback) => {
      lastFrame += 1;
      frames.set(lastFrame, callback);
      return lastFrame;
    })
  );
  vi.stubGlobal(
    'cancelAnimationFrame',
    vi.fn((id: number) => frames.delete(id))
  );
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

function recorder() {
  const calls: string[] = [];
  const recognizer: Recognizer = {
    pointerCount: 1,
    exclusive: false,
    resumable: false,
    enabled: () => true,
    pointerIds: () => [],
    down: (pointer) => calls.push(`down ${pointer.id} ${pointer.current.x}`),
    move: (moved) => {
      for (const pointer of moved) calls.push(`move ${pointer.id} ${pointer.current.x}`);
      return false;
    },
    up: (pointer) => calls.push(`up ${pointer.id} ${pointer.current.x}`),
    cancel: () => calls.push('cancel'),
    reset: () => calls.push('reset')
  };
  return { calls, recognizer };
}

function mount(spec: GesturesSpec) {
  const element = document.createElement('div');
  const child = document.createElement('button');
  element.append(child);
  const captured = new Set<number>();
  element.setPointerCapture = vi.fn((id: number) => void captured.add(id));
  element.releasePointerCapture = vi.fn((id: number) => void captured.delete(id));
  element.hasPointerCapture = (id: number) => captured.has(id);
  const cleanup = gestures(spec)(element);
  if (typeof cleanup !== 'function') throw new Error('gestures attachment returned no cleanup');
  return { element, child, captured, cleanup };
}

function pointer(
  target: Element,
  type: string,
  clientX: number,
  init: PointerEventInit = {}
): void {
  target.dispatchEvent(
    new PointerEvent(type, {
      pointerId: 1,
      pointerType: 'touch',
      clientX,
      button: 0,
      buttons: 1,
      ...init
    })
  );
}

function touchMoveBlocked(element: Element): boolean {
  const event = new TouchEvent('touchmove', { cancelable: true });
  element.dispatchEvent(event);
  return event.defaultPrevented;
}

function contextMenuBlocked(element: Element): boolean {
  const event = new MouseEvent('contextmenu', { cancelable: true });
  element.dispatchEvent(event);
  return event.defaultPrevented;
}

describe('gestures attachment', () => {
  it('sets touch-action and restores it on teardown', () => {
    const { element, cleanup } = mount({ recognizers: [] });
    expect(element.style.touchAction).toBe('none');
    cleanup();
    expect(element.style.touchAction).toBe('');
  });

  it('takes a touch-action override or leaves it alone', () => {
    expect(mount({ recognizers: [], touchAction: 'pan-y' }).element.style.touchAction).toBe(
      'pan-y'
    );
    expect(mount({ recognizers: [], touchAction: null }).element.style.touchAction).toBe('');
  });

  it('captures pointers only once a recognizer locks', () => {
    const { element, captured } = mount({ recognizers: [panRecognizer({})] });
    pointer(element, 'pointerdown', 0, { pointerId: 4 });
    expect(element.setPointerCapture).not.toHaveBeenCalled();
    pointer(element, 'pointermove', 30, { pointerId: 4 });
    runFrame();
    expect(captured).toEqual(new Set([4]));
  });

  it('releases the remaining finger when a pinch ends and recaptures it for the pan', () => {
    const { element, captured } = mount({
      recognizers: [panRecognizer({}), pinchRecognizer({})]
    });
    pointer(element, 'pointerdown', 100, { pointerId: 1 });
    pointer(element, 'pointerdown', 200, { pointerId: 2 });
    pointer(element, 'pointermove', 300, { pointerId: 2 });
    runFrame();
    expect(captured).toEqual(new Set([1, 2]));
    pointer(element, 'pointerup', 300, { pointerId: 2 });
    expect(element.releasePointerCapture).toHaveBeenCalledWith(1);
    expect(captured).toEqual(new Set([2]));
    pointer(element, 'pointermove', 150, { pointerId: 1 });
    runFrame();
    expect(captured.has(1)).toBe(true);
  });

  it('releases captured pointers on teardown', () => {
    const { element, captured, cleanup } = mount({ recognizers: [panRecognizer({})] });
    pointer(element, 'pointerdown', 0);
    pointer(element, 'pointermove', 30);
    runFrame();
    cleanup();
    expect(captured.size).toBe(0);
  });

  it('coalesces moves into one dispatch per frame with the latest position', () => {
    const { calls, recognizer } = recorder();
    const { element } = mount({ recognizers: [recognizer] });
    pointer(element, 'pointerdown', 0);
    pointer(element, 'pointermove', 10);
    pointer(element, 'pointermove', 20);
    pointer(element, 'pointermove', 30);
    expect(requestAnimationFrame).toHaveBeenCalledOnce();
    expect(calls).toEqual(['down 1 0']);
    runFrame();
    expect(calls).toEqual(['down 1 0', 'move 1 30']);
    pointer(element, 'pointermove', 40);
    runFrame();
    expect(calls).toEqual(['down 1 0', 'move 1 30', 'move 1 40']);
  });

  it('hands a pinch both fingers in a single frame dispatch', () => {
    const start = vi.fn();
    const move = vi.fn();
    const { element } = mount({ recognizers: [pinchRecognizer({ start, move })] });
    pointer(element, 'pointerdown', 100, { pointerId: 1 });
    pointer(element, 'pointerdown', 200, { pointerId: 2 });
    pointer(element, 'pointermove', 50, { pointerId: 1 });
    pointer(element, 'pointermove', 250, { pointerId: 2 });
    runFrame();
    expect(start).toHaveBeenCalledExactlyOnceWith(expect.objectContaining({ scale: 2 }));
    expect(move).not.toHaveBeenCalled();
  });

  it('flushes pending moves before an up', () => {
    const { calls, recognizer } = recorder();
    const { element } = mount({ recognizers: [recognizer] });
    pointer(element, 'pointerdown', 0);
    pointer(element, 'pointermove', 15);
    pointer(element, 'pointerup', 20);
    expect(calls).toEqual(['down 1 0', 'move 1 15', 'up 1 20']);
    expect(cancelAnimationFrame).toHaveBeenCalledWith(1);
    runFrame();
    expect(calls).toHaveLength(3);
  });

  it('flushes pending moves before another pointer goes down', () => {
    const { calls, recognizer } = recorder();
    const { element } = mount({ recognizers: [recognizer] });
    pointer(element, 'pointerdown', 0);
    pointer(element, 'pointermove', 15);
    pointer(element, 'pointerdown', 90, { pointerId: 2 });
    expect(calls).toEqual(['down 1 0', 'move 1 15', 'reset']);
  });

  it('ignores hover moves and every button but the primary one', () => {
    const { calls, recognizer } = recorder();
    const { element } = mount({ recognizers: [recognizer] });
    pointer(element, 'pointermove', 10, { pointerType: 'mouse', buttons: 0 });
    pointer(element, 'pointerdown', 10, { pointerType: 'mouse', button: 2 });
    pointer(element, 'pointerdown', 10, { pointerId: 2, pointerType: 'pen', button: 5 });
    pointer(element, 'pointermove', 20, { pointerType: 'mouse' });
    expect(requestAnimationFrame).not.toHaveBeenCalled();
    expect(calls).toEqual([]);
  });

  it('cancels a pointer that moves with no button held', () => {
    const { calls, recognizer } = recorder();
    const { element } = mount({ recognizers: [recognizer] });
    pointer(element, 'pointerdown', 0, { pointerType: 'mouse' });
    pointer(element, 'pointermove', 40, { pointerType: 'mouse', buttons: 0 });
    expect(calls).toEqual(['down 1 0', 'reset']);
    expect(requestAnimationFrame).not.toHaveBeenCalled();
  });

  it('routes only the attachment pointer types', () => {
    const { calls, recognizer } = recorder();
    const { element } = mount({ recognizers: [recognizer], pointerTypes: ['pen'] });
    pointer(element, 'pointerdown', 0);
    pointer(element, 'pointerdown', 5, { pointerId: 2, pointerType: 'pen' });
    expect(calls).toEqual(['down 2 5']);
  });

  it('starts only on pointers whose target it accepts', () => {
    const { calls, recognizer } = recorder();
    const { element, child } = mount({
      recognizers: [recognizer],
      accept: (target) => target === element
    });
    pointer(child, 'pointerdown', 0);
    pointer(child, 'pointerup', 0);
    pointer(element, 'pointerdown', 5, { pointerId: 2 });
    expect(calls).toEqual(['down 2 5']);
  });

  it('cancels on a lost capture of its own element only', () => {
    const cancel = vi.fn();
    const { element, child } = mount({ recognizers: [panRecognizer({ cancel })] });
    pointer(element, 'pointerdown', 0);
    pointer(element, 'pointermove', 30);
    runFrame();
    pointer(child, 'lostpointercapture', 30, { bubbles: true });
    expect(cancel).not.toHaveBeenCalled();
    pointer(element, 'lostpointercapture', 30);
    expect(cancel).toHaveBeenCalledOnce();
  });

  it('ignores the lost capture that follows a release', () => {
    const { calls, recognizer } = recorder();
    const { element } = mount({ recognizers: [recognizer] });
    pointer(element, 'pointerdown', 0);
    pointer(element, 'lostpointercapture', 0);
    pointer(element, 'pointerup', 0);
    pointer(element, 'lostpointercapture', 0);
    expect(calls).toEqual(['down 1 0', 'up 1 0']);
  });

  it('blocks native touch scrolling only while an exclusive gesture holds it', () => {
    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] });
    const { element } = mount({
      recognizers: [longPressRecognizer({}), panRecognizer({}), pinchRecognizer({})],
      touchAction: 'pan-y'
    });
    pointer(element, 'pointerdown', 0);
    expect(touchMoveBlocked(element)).toBe(false);
    vi.advanceTimersByTime(500);
    expect(touchMoveBlocked(element)).toBe(true);
    pointer(element, 'pointerup', 0);
    pointer(element, 'pointerdown', 0);
    pointer(element, 'pointermove', 30);
    runFrame();
    expect(touchMoveBlocked(element)).toBe(false);
    pointer(element, 'pointerdown', 100, { pointerId: 2 });
    expect(touchMoveBlocked(element)).toBe(true);
  });

  it('blocks the context menu while a touch long-press is pending or running', () => {
    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] });
    const { element } = mount({ recognizers: [longPressRecognizer({})] });
    expect(contextMenuBlocked(element)).toBe(false);
    pointer(element, 'pointerdown', 0);
    expect(contextMenuBlocked(element)).toBe(true);
    vi.advanceTimersByTime(500);
    expect(contextMenuBlocked(element)).toBe(true);
    pointer(element, 'pointerup', 0);
    pointer(element, 'pointerdown', 0, { pointerType: 'mouse' });
    expect(contextMenuBlocked(element)).toBe(false);
  });

  it('removes listeners, cancels the frame and resets on teardown', () => {
    const { calls, recognizer } = recorder();
    const { element, cleanup } = mount({ recognizers: [recognizer] });
    pointer(element, 'pointerdown', 0);
    pointer(element, 'pointermove', 15);
    cleanup();
    expect(cancelAnimationFrame).toHaveBeenCalledWith(1);
    expect(frames.size).toBe(0);
    pointer(element, 'pointerdown', 0);
    pointer(element, 'pointermove', 30);
    pointer(element, 'pointerup', 30);
    expect(calls).toEqual(['down 1 0', 'reset']);
  });

  it('hands recognizers plain numbers once per frame', () => {
    const move = vi.fn();
    const start = vi.fn();
    const { element } = mount({ recognizers: [panRecognizer({ start, move })] });
    pointer(element, 'pointerdown', 0);
    pointer(element, 'pointermove', 20);
    runFrame();
    pointer(element, 'pointermove', 30);
    pointer(element, 'pointermove', 45);
    runFrame();
    expect(start).toHaveBeenCalledOnce();
    expect(move).toHaveBeenCalledExactlyOnceWith(
      expect.objectContaining({ dx: 45, stepX: 25, x: 45 })
    );
  });
});
