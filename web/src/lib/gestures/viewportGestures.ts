import { pointerDrag, type DragHandlers } from '#lib/gestures/drag.ts';
import { centredPoint, type ClientPoint, type Point } from '#lib/gestures/point.ts';
import { DEFAULT_THRESHOLDS } from '#lib/gestures/thresholds.ts';
import { held } from '#lib/input/modifiers.svelte.ts';
import {
  isZoomed,
  panByPixels,
  wheelZoom,
  zoomAround,
  type View,
  type ViewChange,
  type ViewportMetrics
} from '#lib/utils/imageViewport.ts';

export type ViewportGestureSpec = {
  element: () => Element | null;
  metrics: () => ViewportMetrics;
  view: () => View;
  apply: (view: View, e: MouseEvent | undefined, change: ViewChange) => void;
  press?: (e: PointerEvent) => void;
  dragging?: (active: boolean) => void;
  tap?: (e: PointerEvent) => void;
  doubleTap?: (e: MouseEvent) => void;
};

export type ViewportHandlers = DragHandlers & {
  onwheel: (e: WheelEvent) => void;
  ondblclick: (e: MouseEvent) => void;
};

export type ViewportGestures = {
  handlers: ViewportHandlers;
  zoomAt: (point: ClientPoint, zoom: number, e?: MouseEvent) => void;
};

type Press = { pointerId: number; x: number; y: number; travel: number };

export function viewportGestures(spec: ViewportGestureSpec): ViewportGestures {
  let press: Press | null = null;

  const drag = pointerDrag<Point>({
    start(e) {
      press = { pointerId: e.pointerId, x: e.clientX, y: e.clientY, travel: 0 };
      spec.press?.(e);
      if (!isZoomed(spec.metrics(), spec.view())) return null;
      spec.dragging?.(true);
      return { x: e.clientX, y: e.clientY };
    },
    move(e, last) {
      const dx = e.clientX - last.x;
      const dy = e.clientY - last.y;
      last.x = e.clientX;
      last.y = e.clientY;
      spec.apply(panByPixels(spec.metrics(), spec.view(), dx, dy), e, 'pan');
    },
    end() {
      spec.dragging?.(false);
    },
    cancel() {
      spec.dragging?.(false);
      press = null;
    }
  });

  function pressed(e: PointerEvent): Press | null {
    return press && press.pointerId === e.pointerId ? press : null;
  }

  function onpointermove(e: PointerEvent): void {
    const current = pressed(e);
    if (current) {
      current.travel += Math.abs(e.clientX - current.x) + Math.abs(e.clientY - current.y);
      current.x = e.clientX;
      current.y = e.clientY;
    }
    drag.follow.onpointermove(e);
  }

  function onpointerup(e: PointerEvent): void {
    const current = pressed(e);
    if (!current) return;
    press = null;
    drag.follow.onpointerup(e);
    if (current.travel <= DEFAULT_THRESHOLDS.clickTravel) spec.tap?.(e);
  }

  function onpointercancel(e: PointerEvent): void {
    if (pressed(e)) press = null;
    drag.follow.onpointercancel(e);
  }

  function zoomAt(point: ClientPoint, zoom: number, e?: MouseEvent): void {
    const element = spec.element();
    const anchor = element ? centredPoint(point, element) : { x: 0, y: 0 };
    spec.apply(zoomAround(spec.metrics(), spec.view(), zoom, anchor), e, 'zoom');
  }

  function onwheel(e: WheelEvent): void {
    const metrics = spec.metrics();
    const view = spec.view();
    if (held('zoom', e)) {
      e.preventDefault();
      zoomAt(e, wheelZoom(view.zoom ?? metrics.fitZoom, e.deltaY), e);
      return;
    }
    if (!isZoomed(metrics, view)) return;
    e.preventDefault();
    spec.apply(panByPixels(metrics, view, -e.deltaX, -e.deltaY), e, 'pan');
  }

  return {
    handlers: {
      onpointerdown: (e) => void drag.begin(e),
      onpointermove,
      onpointerup,
      onpointercancel,
      onlostpointercapture: drag.follow.onlostpointercapture,
      onwheel,
      ondblclick: (e) => spec.doubleTap?.(e)
    },
    zoomAt
  };
}
