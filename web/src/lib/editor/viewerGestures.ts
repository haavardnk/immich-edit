import { gestures } from '#lib/gestures/gestures.ts';
import { longPressRecognizer } from '#lib/gestures/longPress.ts';
import { panRecognizer } from '#lib/gestures/pan.ts';
import { pinchRecognizer, type PinchUpdate } from '#lib/gestures/pinch.ts';
import { centredPoint, type Box, type ClientPoint, type Point } from '#lib/gestures/point.ts';
import { tapRecognizer } from '#lib/gestures/tap.ts';
import {
  FIT_VIEW,
  isZoomed,
  panByPixels,
  pinchView,
  zoomAround,
  type View,
  type ViewChange,
  type ViewportMetrics
} from '#lib/utils/imageViewport.ts';
import type { Attachment } from 'svelte/attachments';

export const HOLD_ORIGINAL_MS = 350;
const PHOTO_LAYER = 'data-photo-layer';

export type ViewerAdjust = {
  start: () => void;
  move: (stepX: number, width: number) => void;
  end: () => void;
};

export type ViewerGestureSpec = {
  element: () => Element | null;
  anchor?: (point: ClientPoint) => Element | Box | null;
  metrics: () => ViewportMetrics;
  view: () => View;
  apply: (view: View, change: ViewChange) => void;
  zoomLevel: () => number;
  enabled: () => boolean;
  tap: () => void;
  hold: { start: () => void; end: () => void };
  adjust: () => ViewerAdjust | null;
};

export function viewerGestures(spec: ViewerGestureSpec): Attachment<HTMLElement> {
  let start: View = FIT_VIEW;
  let anchor: Point = { x: 0, y: 0 };
  let adjusting: ViewerAdjust | null = null;

  function zoomed(): boolean {
    return isZoomed(spec.metrics(), spec.view());
  }

  function centred(point: Point): Point {
    const client = { clientX: point.x, clientY: point.y };
    const element = spec.anchor?.(client) ?? spec.element();
    return element ? centredPoint(client, element) : point;
  }

  function toggleZoom(point: Point): void {
    if (zoomed()) spec.apply(FIT_VIEW, 'zoom');
    else
      spec.apply(zoomAround(spec.metrics(), spec.view(), spec.zoomLevel(), centred(point)), 'zoom');
  }

  function pinched(pinch: PinchUpdate): void {
    spec.apply(
      pinchView(spec.metrics(), start, pinch.scale, anchor, { x: pinch.dx, y: pinch.dy }),
      'pan'
    );
  }

  function endAdjust(): void {
    adjusting?.end();
    adjusting = null;
  }

  return gestures({
    pointerTypes: ['touch'],
    accept: (target) =>
      target === spec.element() || (target instanceof Element && target.hasAttribute(PHOTO_LAYER)),
    recognizers: [
      tapRecognizer({ enabled: spec.enabled, tap: spec.tap, doubleTap: toggleZoom }),
      longPressRecognizer({
        enabled: spec.enabled,
        thresholds: { longPressMs: HOLD_ORIGINAL_MS },
        start: spec.hold.start,
        end: spec.hold.end,
        cancel: spec.hold.end
      }),
      panRecognizer({
        enabled: () => spec.enabled() && zoomed(),
        start: (pan) => {
          start = spec.view();
          spec.apply(panByPixels(spec.metrics(), start, pan.dx, pan.dy), 'pan');
        },
        move: (pan) => spec.apply(panByPixels(spec.metrics(), start, pan.dx, pan.dy), 'pan')
      }),
      panRecognizer({
        enabled: () => spec.enabled() && !zoomed() && spec.adjust() !== null,
        lockAxis: true,
        thresholds: { panSlop: 8 },
        start: (pan) => {
          if (pan.axis !== 'x') return;
          adjusting = spec.adjust();
          adjusting?.start();
        },
        move: (pan) => adjusting?.move(pan.stepX, spec.element()?.clientWidth ?? 0),
        end: endAdjust,
        cancel: endAdjust
      }),
      pinchRecognizer({
        enabled: spec.enabled,
        start: (pinch) => {
          start = spec.view();
          anchor = centred({ x: pinch.x - pinch.dx, y: pinch.y - pinch.dy });
          pinched(pinch);
        },
        move: pinched,
        end: () => spec.apply(zoomed() ? spec.view() : FIT_VIEW, 'zoom')
      })
    ]
  });
}
