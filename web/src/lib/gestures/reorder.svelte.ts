import { pointerDrag } from '#lib/gestures/drag.ts';

export type ReorderMove = { group: string; id: string; to: number };

type Dragging = { group: string; id: string; from: number; to: number; dy: number };
type Context = { group: string; order: readonly string[]; id: string };
type Start = { y: number; mids: number[]; shift: number };

export type ReorderHandle = {
  onpointerdown: (e: PointerEvent) => void;
  onpointermove: (e: PointerEvent) => void;
  onpointerup: (e: PointerEvent) => void;
  onpointercancel: (e: PointerEvent) => void;
  onlostpointercapture: (e: PointerEvent) => void;
  onkeydown: (e: KeyboardEvent) => void;
};

function itemsOf(handle: Element): Element[] | null {
  const item = handle.closest('[data-reorder-item]');
  const list = item?.parentElement;
  if (!item || !list) return null;
  return [...list.children].filter((child) => child.hasAttribute('data-reorder-item'));
}

export class Reorder {
  dragging = $state<Dragging | null>(null);
  #onmove: (move: ReorderMove) => void;
  #start: Start | null = null;

  constructor(onmove: (move: ReorderMove) => void) {
    this.#onmove = onmove;
  }

  #drag = pointerDrag<Start, Context>({
    start: (e, { group, order, id }) => {
      if (order.length < 2 || !(e.currentTarget instanceof Element)) return null;
      const items = itemsOf(e.currentTarget);
      const from = order.indexOf(id);
      if (!items || items.length !== order.length || from < 0) return null;
      const rects = items.map((item) => item.getBoundingClientRect());
      const first = rects[0];
      const second = rects[1];
      const gap = first && second ? Math.max(0, second.top - first.bottom) : 0;
      e.preventDefault();
      this.#start = {
        y: e.clientY,
        mids: rects.map((rect) => rect.top + rect.height / 2),
        shift: (rects[from]?.height ?? 0) + gap
      };
      this.dragging = { group, id, from, to: from, dy: 0 };
      return this.#start;
    },
    move: (e, start) => {
      const current = this.dragging;
      if (!current) return;
      const dy = e.clientY - start.y;
      const centre = (start.mids[current.from] ?? 0) + dy;
      const to = start.mids.filter((mid, i) => i !== current.from && mid < centre).length;
      this.dragging = { ...current, dy, to };
    },
    end: () => {
      const done = this.dragging;
      this.dragging = null;
      this.#start = null;
      if (done && done.to !== done.from)
        this.#onmove({ group: done.group, id: done.id, to: done.to });
    },
    cancel: () => {
      this.dragging = null;
      this.#start = null;
    }
  });

  handle(group: string, order: readonly string[], id: string): ReorderHandle {
    return {
      onpointerdown: (e) => this.#drag.begin(e, { group, order, id }),
      ...this.#drag.follow,
      onkeydown: (e) => this.#step(e, group, order, id)
    };
  }

  offset(group: string, order: readonly string[], id: string): number {
    const current = this.dragging;
    const shift = this.#start?.shift ?? 0;
    if (!current || current.group !== group) return 0;
    if (current.id === id) return current.dy;
    const index = order.indexOf(id);
    if (current.from < current.to && index > current.from && index <= current.to) return -shift;
    if (current.to < current.from && index >= current.to && index < current.from) return shift;
    return 0;
  }

  active(group: string): boolean {
    return this.dragging?.group === group;
  }

  #step(e: KeyboardEvent, group: string, order: readonly string[], id: string): void {
    const delta = e.key === 'ArrowUp' ? -1 : e.key === 'ArrowDown' ? 1 : 0;
    if (delta === 0) return;
    e.preventDefault();
    const to = order.indexOf(id) + delta;
    if (to < 0 || to >= order.length) return;
    this.#onmove({ group, id, to });
  }
}
