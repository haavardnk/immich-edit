import { clear, latch } from '#lib/input/modifiers.svelte.ts';
import { FIT_VIEW, viewportMetrics, type View, type ViewChange } from '#lib/utils/imageViewport.ts';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { viewportGestures, type ViewportGestureSpec } from './viewportGestures';

type Rect = { left: number; top: number; width: number; height: number };

const BOX: Rect = { left: 100, top: 50, width: 800, height: 600 };
const METRICS = viewportMetrics({ w: 800, h: 600 }, { w: 800, h: 600 }, 1600, 1);
const ZOOMED: View = { zoom: 100, cx: 0.5, cy: 0.5 };

type Applied = { view: View; change: ViewChange; event: MouseEvent | undefined };

function element(): Element {
  return { getBoundingClientRect: () => BOX } as unknown as Element;
}

function captureTarget(): Element {
  const captured = new Set<number>();
  return {
    getBoundingClientRect: () => BOX,
    setPointerCapture: (id: number) => void captured.add(id),
    releasePointerCapture: (id: number) => void captured.delete(id),
    hasPointerCapture: (id: number) => captured.has(id)
  } as unknown as Element;
}

function pointer(
  currentTarget: Element,
  clientX: number,
  clientY: number,
  pointerId = 1,
  pointerType = 'mouse'
): PointerEvent {
  return {
    currentTarget,
    target: currentTarget,
    pointerId,
    pointerType,
    clientX,
    clientY,
    shiftKey: false,
    altKey: false,
    ctrlKey: false,
    metaKey: false,
    preventDefault: vi.fn(),
    stopPropagation: vi.fn()
  } as unknown as PointerEvent;
}

function wheel(deltaX: number, deltaY: number, ctrlKey = false): WheelEvent {
  return {
    deltaX,
    deltaY,
    clientX: 500,
    clientY: 350,
    shiftKey: false,
    altKey: false,
    ctrlKey,
    metaKey: false,
    preventDefault: vi.fn()
  } as unknown as WheelEvent;
}

function setup(initial: View, extra: Partial<ViewportGestureSpec> = {}) {
  let view = initial;
  const applied: Applied[] = [];
  const gestures = viewportGestures({
    element,
    metrics: () => METRICS,
    view: () => view,
    apply: (next, event, change) => {
      applied.push({ view: next, change, event });
      view = next;
    },
    ...extra
  });
  return { handlers: gestures.handlers, gestures, applied, current: () => view };
}

afterEach(() => clear());

describe('viewportGestures drag', () => {
  it('pans a zoomed view with the pointer', () => {
    const dragging = vi.fn();
    const tap = vi.fn();
    const { handlers, applied, current } = setup(ZOOMED, { dragging, tap });
    const target = captureTarget();
    handlers.onpointerdown(pointer(target, 300, 300));
    handlers.onpointermove(pointer(target, 460, 180));
    handlers.onpointerup(pointer(target, 460, 180));
    expect(current()).toEqual({ zoom: 100, cx: 0.4, cy: 0.6 });
    expect(applied.map((a) => a.change)).toEqual(['pan']);
    expect(dragging.mock.calls).toEqual([[true], [false]]);
    expect(tap).not.toHaveBeenCalled();
  });

  it('stops at the photo edge', () => {
    const { handlers, current } = setup(ZOOMED);
    const target = captureTarget();
    handlers.onpointerdown(pointer(target, 0, 0));
    handlers.onpointermove(pointer(target, 2000, -2000));
    expect(current()).toEqual({ zoom: 100, cx: 0.25, cy: 0.75 });
  });

  it('presses but does not drag at fit', () => {
    const press = vi.fn();
    const dragging = vi.fn();
    const { handlers, applied } = setup(FIT_VIEW, { press, dragging });
    const target = captureTarget();
    handlers.onpointerdown(pointer(target, 300, 300));
    handlers.onpointermove(pointer(target, 302, 301));
    expect(press).toHaveBeenCalledOnce();
    expect(dragging).not.toHaveBeenCalled();
    expect(applied).toEqual([]);
  });

  it('stops a cancelled drag', () => {
    const dragging = vi.fn();
    const tap = vi.fn();
    const { handlers } = setup(ZOOMED, { dragging, tap });
    const target = captureTarget();
    handlers.onpointerdown(pointer(target, 300, 300));
    handlers.onpointercancel(pointer(target, 300, 300));
    handlers.onpointerup(pointer(target, 300, 300));
    expect(dragging.mock.calls).toEqual([[true], [false]]);
    expect(tap).not.toHaveBeenCalled();
  });

  it('stops a drag that lost capture without tapping', () => {
    const dragging = vi.fn();
    const tap = vi.fn();
    const { handlers } = setup(ZOOMED, { dragging, tap });
    const target = captureTarget();
    handlers.onpointerdown(pointer(target, 300, 300));
    target.releasePointerCapture(1);
    handlers.onlostpointercapture(pointer(target, 300, 300));
    handlers.onpointerup(pointer(target, 300, 300));
    expect(dragging.mock.calls).toEqual([[true], [false]]);
    expect(tap).not.toHaveBeenCalled();
  });
});

describe('viewportGestures tap', () => {
  it.each<[string, View]>([
    ['at fit', FIT_VIEW],
    ['when zoomed', ZOOMED]
  ])('taps a press released in place %s', (_label, view) => {
    const tap = vi.fn();
    const { handlers } = setup(view, { tap });
    const target = captureTarget();
    const up = pointer(target, 303, 302);
    handlers.onpointerdown(pointer(target, 300, 300));
    handlers.onpointermove(pointer(target, 303, 302));
    handlers.onpointerup(up);
    expect(tap).toHaveBeenCalledExactlyOnceWith(up);
  });

  it.each<[string, View]>([
    ['at fit', FIT_VIEW],
    ['when zoomed', ZOOMED]
  ])('does not tap after travelling past the threshold %s', (_label, view) => {
    const tap = vi.fn();
    const { handlers } = setup(view, { tap });
    const target = captureTarget();
    handlers.onpointerdown(pointer(target, 300, 300));
    handlers.onpointermove(pointer(target, 304, 302));
    handlers.onpointermove(pointer(target, 300, 300));
    handlers.onpointerup(pointer(target, 300, 300));
    expect(tap).not.toHaveBeenCalled();
  });

  it('ignores ups, moves and cancels from another pointer', () => {
    const tap = vi.fn();
    const dragging = vi.fn();
    const { handlers, applied } = setup(ZOOMED, { tap, dragging });
    const target = captureTarget();
    handlers.onpointerdown(pointer(target, 300, 300, 1));
    handlers.onpointerdown(pointer(target, 500, 500, 2));
    handlers.onpointermove(pointer(target, 900, 900, 2));
    handlers.onpointerup(pointer(target, 500, 500, 2));
    handlers.onpointercancel(pointer(target, 500, 500, 2));
    expect(tap).not.toHaveBeenCalled();
    expect(applied).toEqual([]);
    expect(dragging.mock.calls).toEqual([[true]]);
    handlers.onpointerup(pointer(target, 300, 300, 1));
    expect(tap).toHaveBeenCalledOnce();
    expect(dragging.mock.calls).toEqual([[true], [false]]);
  });

  it('ignores presses from pointer types it is not given', () => {
    const tap = vi.fn();
    const press = vi.fn();
    const { handlers, applied } = setup(ZOOMED, { tap, press, pointerTypes: ['mouse', 'pen'] });
    const target = captureTarget();
    handlers.onpointerdown(pointer(target, 300, 300, 1, 'touch'));
    handlers.onpointermove(pointer(target, 460, 180, 1, 'touch'));
    handlers.onpointerup(pointer(target, 460, 180, 1, 'touch'));
    expect(press).not.toHaveBeenCalled();
    expect(tap).not.toHaveBeenCalled();
    expect(applied).toEqual([]);

    handlers.onpointerdown(pointer(target, 300, 300, 2, 'pen'));
    handlers.onpointerup(pointer(target, 300, 300, 2, 'pen'));
    expect(tap).toHaveBeenCalledOnce();
  });

  it('does not tap a cancelled press', () => {
    const tap = vi.fn();
    const { handlers } = setup(FIT_VIEW, { tap });
    const target = captureTarget();
    handlers.onpointerdown(pointer(target, 300, 300));
    handlers.onpointercancel(pointer(target, 300, 300));
    handlers.onpointerup(pointer(target, 300, 300));
    expect(tap).not.toHaveBeenCalled();
  });

  it('passes the native double click on', () => {
    const doubleTap = vi.fn();
    const { handlers } = setup(FIT_VIEW, { doubleTap });
    const e = { clientX: 1, clientY: 2 } as MouseEvent;
    handlers.ondblclick(e);
    expect(doubleTap).toHaveBeenCalledExactlyOnceWith(e);
  });

  it('ignores a double click without a handler', () => {
    const { handlers, applied } = setup(FIT_VIEW);
    handlers.ondblclick({ clientX: 1, clientY: 2 } as MouseEvent);
    expect(applied).toEqual([]);
  });
});

describe('viewportGestures wheel', () => {
  it('zooms around the pointer with the zoom modifier', () => {
    const { handlers, applied } = setup(FIT_VIEW);
    const e = wheel(0, -100, true);
    handlers.onwheel(e);
    expect(e.preventDefault).toHaveBeenCalled();
    const [last] = applied;
    expect(last?.change).toBe('zoom');
    expect(last?.event).toBe(e);
    expect(last?.view.zoom).toBeCloseTo(55, 9);
    expect(last?.view.cx).toBeCloseTo(0.5, 9);
    expect(last?.view.cy).toBeCloseTo(0.5, 9);
  });

  it('honours a latched zoom modifier', () => {
    latch('zoom', true);
    const { handlers, current } = setup(ZOOMED);
    handlers.onwheel(wheel(0, 100));
    expect(current().zoom).toBeCloseTo(100 / 1.1, 9);
  });

  it('pans a zoomed view against the wheel delta', () => {
    const { handlers, applied } = setup(ZOOMED);
    const e = wheel(160, -120);
    handlers.onwheel(e);
    expect(e.preventDefault).toHaveBeenCalled();
    expect(applied).toEqual([{ view: { zoom: 100, cx: 0.6, cy: 0.4 }, change: 'pan', event: e }]);
  });

  it('lets the page scroll at fit', () => {
    const { handlers, applied } = setup(FIT_VIEW);
    const e = wheel(0, 100);
    handlers.onwheel(e);
    expect(e.preventDefault).not.toHaveBeenCalled();
    expect(applied).toEqual([]);
  });
});

describe('viewportGestures zoomAt', () => {
  it('keeps the focal point under the pointer', () => {
    const { gestures, applied } = setup(FIT_VIEW);
    gestures.zoomAt({ clientX: 700, clientY: 200 }, 100);
    expect(applied).toEqual([
      { view: { zoom: 100, cx: 0.625, cy: 0.375 }, change: 'zoom', event: undefined }
    ]);
  });

  it('passes the event on for its modifiers', () => {
    const { gestures, applied } = setup(FIT_VIEW);
    const e = { clientX: 0, clientY: 0, altKey: true } as MouseEvent;
    gestures.zoomAt({ clientX: 500, clientY: 350 }, 100, e);
    expect(applied[0]?.event).toBe(e);
    expect(applied[0]?.view).toEqual(ZOOMED);
  });

  it('zooms about the centre without an element', () => {
    const { gestures, current } = setup(FIT_VIEW, { element: () => null });
    gestures.zoomAt({ clientX: 700, clientY: 200 }, 100);
    expect(current()).toEqual(ZOOMED);
  });
});
