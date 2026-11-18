export type DragSpec<S, C> = {
  start: (e: PointerEvent, ctx: C) => S | null;
  move?: (e: PointerEvent, state: S) => void;
  end?: (e: PointerEvent, state: S) => void;
  cancel?: (e: PointerEvent, state: S) => void;
  hover?: (e: PointerEvent) => void;
};

export type DragFollow = {
  onpointermove: (e: PointerEvent) => void;
  onpointerup: (e: PointerEvent) => void;
  onpointercancel: (e: PointerEvent) => void;
  onlostpointercapture: (e: PointerEvent) => void;
};

export type DragHandlers = DragFollow & {
  onpointerdown: (e: PointerEvent) => void;
};

export type PointerDrag<C> = {
  begin: (e: PointerEvent, ctx: C) => boolean;
  on: (ctx: C) => DragHandlers;
  follow: DragFollow;
  dragging: () => boolean;
};

type Active<S> = { pointerId: number; target: Element; state: S };

function isElement(target: EventTarget | null): target is Element {
  return target !== null && 'getBoundingClientRect' in target;
}

export function pointerDrag<S, C = void>(spec: DragSpec<S, C>): PointerDrag<C> {
  let active: Active<S> | null = null;

  function live(current: Active<S>): boolean {
    return current.target.hasPointerCapture?.(current.pointerId) === true;
  }

  function finish(e: PointerEvent): Active<S> | null {
    const current = active;
    if (!current || e.pointerId !== current.pointerId) return null;
    active = null;
    if (live(current)) current.target.releasePointerCapture(current.pointerId);
    return current;
  }

  function abort(e: PointerEvent, state: S): void {
    (spec.cancel ?? spec.end)?.(e, state);
  }

  function begin(e: PointerEvent, ctx: C): boolean {
    const target = e.currentTarget;
    if (!isElement(target)) return false;
    const stale = active;
    if (stale && live(stale)) {
      e.preventDefault();
      e.stopPropagation();
      return false;
    }
    if (stale) {
      active = null;
      abort(e, stale.state);
    }
    const state = spec.start(e, ctx);
    if (state === null) return false;
    active = { pointerId: e.pointerId, target, state };
    target.setPointerCapture?.(e.pointerId);
    return true;
  }

  function cancel(e: PointerEvent): void {
    const current = finish(e);
    if (current) abort(e, current.state);
  }

  const follow: DragFollow = {
    onpointermove(e) {
      if (!active) {
        spec.hover?.(e);
        return;
      }
      if (e.pointerId !== active.pointerId) return;
      spec.move?.(e, active.state);
    },
    onpointerup(e) {
      const current = finish(e);
      if (current) spec.end?.(e, current.state);
    },
    onpointercancel: cancel,
    onlostpointercapture(e) {
      if (active && e.target === active.target) cancel(e);
    }
  };

  return {
    begin,
    on: (ctx) => ({ ...follow, onpointerdown: (e) => void begin(e, ctx) }),
    follow,
    dragging: () => active !== null
  };
}
