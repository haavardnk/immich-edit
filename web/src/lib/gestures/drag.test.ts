import { describe, expect, it, vi } from 'vitest';
import { pointerDrag } from './drag.ts';

type Rect = { left: number; top: number; width: number; height: number };

type FakeTarget = {
  getBoundingClientRect: () => Rect;
  setPointerCapture: (id: number) => void;
  releasePointerCapture: (id: number) => void;
  hasPointerCapture: (id: number) => boolean;
  captured: Set<number>;
};

const BOX: Rect = { left: 0, top: 0, width: 100, height: 100 };

function target(): FakeTarget {
  const captured = new Set<number>();
  return {
    captured,
    getBoundingClientRect: () => BOX,
    setPointerCapture: vi.fn((id: number) => void captured.add(id)),
    releasePointerCapture: vi.fn((id: number) => void captured.delete(id)),
    hasPointerCapture: (id: number) => captured.has(id)
  };
}

function event(
  currentTarget: object | null,
  pointerId = 1,
  extra: Partial<{ clientX: number; target: object }> = {}
): PointerEvent {
  return {
    currentTarget,
    pointerId,
    clientX: 0,
    preventDefault: vi.fn(),
    stopPropagation: vi.fn(),
    ...extra
  } as unknown as PointerEvent;
}

function spec() {
  return {
    start: vi.fn((e: PointerEvent, ctx: string) => ({ ctx, from: e.clientX })),
    move: vi.fn(),
    end: vi.fn(),
    hover: vi.fn()
  };
}

describe('pointerDrag', () => {
  it('captures on start, follows moves and releases on up', () => {
    const s = spec();
    const drag = pointerDrag(s);
    const el = target();
    const handlers = drag.on('a');
    handlers.onpointerdown(event(el, 1, { clientX: 5 }));
    expect(el.captured.has(1)).toBe(true);
    expect(drag.dragging()).toBe(true);
    handlers.onpointermove(event(el));
    expect(s.move).toHaveBeenCalledWith(expect.anything(), { ctx: 'a', from: 5 });
    handlers.onpointerup(event(el));
    expect(s.end).toHaveBeenCalledOnce();
    expect(el.captured.size).toBe(0);
    expect(drag.dragging()).toBe(false);
  });

  it('does not start or capture when start returns null', () => {
    const drag = pointerDrag<number, void>({ start: () => null, end: vi.fn() });
    const el = target();
    expect(drag.begin(event(el), undefined)).toBe(false);
    expect(el.setPointerCapture).not.toHaveBeenCalled();
    expect(drag.dragging()).toBe(false);
  });

  it('ignores other pointers while a drag is active and stops their press', () => {
    const s = spec();
    const drag = pointerDrag(s);
    const el = target();
    drag.begin(event(el, 1), 'a');
    const second = event(el, 2);
    expect(drag.begin(second, 'b')).toBe(false);
    expect(second.preventDefault).toHaveBeenCalledOnce();
    expect(second.stopPropagation).toHaveBeenCalledOnce();
    drag.follow.onpointermove(event(el, 2));
    drag.follow.onpointerup(event(el, 2));
    expect(s.start).toHaveBeenCalledOnce();
    expect(s.move).not.toHaveBeenCalled();
    expect(s.end).not.toHaveBeenCalled();
    expect(drag.dragging()).toBe(true);
  });

  it('leaves a press alone when start declines it', () => {
    const drag = pointerDrag<number, void>({ start: () => null });
    const press = event(target());
    drag.begin(press, undefined);
    expect(press.preventDefault).not.toHaveBeenCalled();
    expect(press.stopPropagation).not.toHaveBeenCalled();
  });

  it('ends a stale drag before starting the next one', () => {
    const s = spec();
    const drag = pointerDrag(s);
    const el = target();
    drag.begin(event(el, 1), 'a');
    el.captured.clear();
    expect(drag.begin(event(el, 2), 'b')).toBe(true);
    expect(s.end).toHaveBeenCalledOnce();
    expect(s.end).toHaveBeenLastCalledWith(expect.anything(), { ctx: 'a', from: 0 });
    drag.follow.onpointerup(event(el, 2));
    expect(s.end).toHaveBeenLastCalledWith(expect.anything(), { ctx: 'b', from: 0 });
  });

  it('cancels a stale drag through cancel when given', () => {
    const cancel = vi.fn();
    const s = { ...spec(), cancel };
    const drag = pointerDrag(s);
    const el = target();
    drag.begin(event(el, 1), 'a');
    el.captured.clear();
    drag.begin(event(el, 2), 'b');
    expect(cancel).toHaveBeenCalledWith(expect.anything(), { ctx: 'a', from: 0 });
    expect(s.end).not.toHaveBeenCalled();
  });

  it('ends a stale drag even when the next start declines', () => {
    const end = vi.fn();
    let accept = true;
    const drag = pointerDrag<string, void>({ start: () => (accept ? 'first' : null), end });
    const el = target();
    drag.begin(event(el, 1), undefined);
    el.captured.clear();
    accept = false;
    expect(drag.begin(event(el, 2), undefined)).toBe(false);
    expect(end).toHaveBeenCalledWith(expect.anything(), 'first');
    expect(drag.dragging()).toBe(false);
  });

  it('cancels through end by default', () => {
    const s = spec();
    const drag = pointerDrag(s);
    const el = target();
    drag.begin(event(el), 'a');
    drag.follow.onpointercancel(event(el));
    expect(s.end).toHaveBeenCalledOnce();
    expect(el.captured.size).toBe(0);
  });

  it('prefers an explicit cancel handler', () => {
    const cancel = vi.fn();
    const s = { ...spec(), cancel };
    const drag = pointerDrag(s);
    const el = target();
    drag.begin(event(el), 'a');
    drag.follow.onpointercancel(event(el));
    expect(cancel).toHaveBeenCalledOnce();
    expect(s.end).not.toHaveBeenCalled();
  });

  it('cancels on lost capture only from the capture target', () => {
    const cancel = vi.fn();
    const drag = pointerDrag({ ...spec(), cancel });
    const el = target();
    drag.begin(event(el), 'a');
    drag.follow.onlostpointercapture(event(el, 1, { target: {} }));
    expect(cancel).not.toHaveBeenCalled();
    drag.follow.onlostpointercapture(event(el, 1, { target: el }));
    expect(cancel).toHaveBeenCalledOnce();
    expect(drag.dragging()).toBe(false);
  });

  it('reports hover moves while idle', () => {
    const s = spec();
    const drag = pointerDrag(s);
    const el = target();
    drag.follow.onpointermove(event(el));
    expect(s.hover).toHaveBeenCalledOnce();
    drag.begin(event(el), 'a');
    drag.follow.onpointermove(event(el));
    expect(s.hover).toHaveBeenCalledOnce();
    expect(s.move).toHaveBeenCalledOnce();
  });

  it('works without pointer capture support', () => {
    const s = spec();
    const drag = pointerDrag(s);
    const el = { getBoundingClientRect: () => BOX };
    expect(drag.begin(event(el), 'a')).toBe(true);
    drag.follow.onpointerup(event(el));
    expect(s.end).toHaveBeenCalledOnce();
  });

  it('does not run start without an element target', () => {
    const s = spec();
    const drag = pointerDrag(s);
    expect(drag.begin(event(null), 'a')).toBe(false);
    expect(s.start).not.toHaveBeenCalled();
    expect(drag.dragging()).toBe(false);
  });
});
