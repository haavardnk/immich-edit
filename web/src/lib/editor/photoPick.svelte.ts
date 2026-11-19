import { pointerDrag, type DragHandlers } from '#lib/gestures/drag.ts';
import { clampedPoint } from '#lib/gestures/point.ts';
import type { UvRect } from '#lib/utils/previewSurface.ts';

export type PickSpot = { u: number; v: number; x: number; y: number };

export type PhotoPickSpec = {
  busy?: () => boolean;
  track?: (box: UvRect) => void;
  pick: (box: UvRect, e: PointerEvent) => void;
};

const MIN_DRAG_PX = 6;

function spotAt(e: PointerEvent): PickSpot {
  const at = clampedPoint(e, e.currentTarget as Element);
  return { u: at.x, v: at.y, x: e.clientX, y: e.clientY };
}

export function boxBetween(a: PickSpot, b: PickSpot): UvRect {
  const u = Math.min(a.u, b.u);
  const v = Math.min(a.v, b.v);
  return { u, v, w: Math.max(a.u, b.u) - u, h: Math.max(a.v, b.v) - v };
}

export class PhotoPick {
  start = $state<PickSpot | null>(null);
  pointer = $state<PickSpot | null>(null);
  touching = $state(false);
  readonly handlers: DragHandlers;
  #spec: PhotoPickSpec;

  constructor(spec: PhotoPickSpec) {
    this.#spec = spec;
    const drag = pointerDrag<true>({
      start: (e) => {
        e.preventDefault();
        e.stopPropagation();
        if (spec.busy?.() || this.start || e.button !== 0) return null;
        this.touching = e.pointerType === 'touch';
        this.start = spotAt(e);
        this.pointer = this.start;
        return true;
      },
      hover: (e) => this.#track(e),
      move: (e) => this.#track(e),
      end: (e) => {
        const start = this.start;
        if (!start) return;
        e.stopPropagation();
        const at = this.touching && this.pointer ? this.pointer : start;
        const picked =
          this.dragging && this.pointer ? boxBetween(start, this.pointer) : boxBetween(at, at);
        this.start = null;
        if (this.touching) this.pointer = null;
        spec.pick(picked, e);
      },
      cancel: () => {
        this.start = null;
      }
    });
    this.handlers = drag.on();
  }

  get dragging(): boolean {
    const { start, pointer } = this;
    return (
      !this.touching &&
      !!start &&
      !!pointer &&
      Math.hypot(pointer.x - start.x, pointer.y - start.y) >= MIN_DRAG_PX
    );
  }

  get box(): UvRect | null {
    const { start, pointer } = this;
    if (start && pointer && this.dragging) return boxBetween(start, pointer);
    return pointer ? boxBetween(pointer, pointer) : null;
  }

  leave = (): boolean => {
    if (this.start) return false;
    this.pointer = null;
    return true;
  };

  reset = (): void => {
    this.start = null;
    this.pointer = null;
  };

  #track(e: PointerEvent): void {
    this.pointer = spotAt(e);
    const box = this.box;
    if (box) this.#spec.track?.(box);
  }
}
