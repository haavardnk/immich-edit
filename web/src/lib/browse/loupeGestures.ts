import type { LoupeStage } from '#lib/browse/loupeStage.ts';
import { runCommand } from '#lib/commands/command.ts';
import { loupeFullscreen } from '#lib/commands/loupe/view.ts';
import { gestures } from '#lib/gestures/gestures.ts';
import { panRecognizer } from '#lib/gestures/pan.ts';
import { pinchRecognizer, type PinchUpdate } from '#lib/gestures/pinch.ts';
import { centredPoint, type Point } from '#lib/gestures/point.ts';
import { swipeRecognizer } from '#lib/gestures/swipe.ts';
import { tapRecognizer } from '#lib/gestures/tap.ts';
import { twoFingerTapRecognizer } from '#lib/gestures/twoFingerTap.ts';
import {
  FIT_VIEW,
  isZoomed,
  panByPixels,
  pinchView,
  zoomAround,
  type View,
  type ViewportMetrics
} from '#lib/utils/imageViewport.ts';
import type { Attachment } from 'svelte/attachments';

export type LoupeGestureSpec = {
  element: () => Element | null;
  metrics: () => ViewportMetrics;
  view: () => View;
  apply: (view: View) => void;
  focus: () => void;
  zoomLevel: () => number;
  stage: () => LoupeStage | null;
};

export function loupeGestures(spec: LoupeGestureSpec): Attachment<HTMLElement> {
  let start: View = FIT_VIEW;
  let anchor: Point = { x: 0, y: 0 };

  function zoomed(): boolean {
    return isZoomed(spec.metrics(), spec.view());
  }

  function centred(point: Point): Point {
    const element = spec.element();
    return element ? centredPoint({ clientX: point.x, clientY: point.y }, element) : point;
  }

  function toggleZoom(point: Point): void {
    spec.focus();
    if (zoomed()) {
      spec.apply(FIT_VIEW);
      return;
    }
    spec.apply(zoomAround(spec.metrics(), spec.view(), spec.zoomLevel(), centred(point)));
  }

  function pinched(pinch: PinchUpdate): void {
    spec.apply(pinchView(spec.metrics(), start, pinch.scale, anchor, { x: pinch.dx, y: pinch.dy }));
  }

  function swiping(): LoupeStage | null {
    return zoomed() ? null : spec.stage();
  }

  return gestures({
    pointerTypes: ['touch'],
    recognizers: [
      tapRecognizer({ tap: spec.focus, doubleTap: toggleZoom }),
      swipeRecognizer({
        enabled: () => swiping() !== null,
        start: (progress) => swiping()?.begin(progress),
        move: (progress) => spec.stage()?.follow(progress),
        end: (result) => spec.stage()?.release(result),
        cancel: () => spec.stage()?.cancel()
      }),
      panRecognizer({
        enabled: zoomed,
        start: (pan) => {
          spec.focus();
          start = spec.view();
          spec.apply(panByPixels(spec.metrics(), start, pan.dx, pan.dy));
        },
        move: (pan) => spec.apply(panByPixels(spec.metrics(), start, pan.dx, pan.dy))
      }),
      pinchRecognizer({
        start: (pinch) => {
          spec.focus();
          start = spec.view();
          anchor = centred({ x: pinch.x - pinch.dx, y: pinch.y - pinch.dy });
          pinched(pinch);
        },
        move: pinched,
        end: () => {
          if (!zoomed()) spec.apply(FIT_VIEW);
        }
      }),
      twoFingerTapRecognizer({ tap: () => runCommand(loupeFullscreen) })
    ]
  });
}
