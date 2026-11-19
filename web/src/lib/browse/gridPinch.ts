import { stepGrid } from '#lib/browse/gridResize.ts';
import { scaleStepper } from '#lib/browse/gridZoom.ts';
import { listenGridZoom } from '#lib/browse/gridZoomInput.ts';
import { pinchRecognizer, type PinchUpdate } from '#lib/gestures/pinch.ts';
import type { Recognizer } from '#lib/gestures/recognizer.ts';

export function gridPinch(enabled: () => boolean): Recognizer {
  const steps = scaleStepper();
  const step = (pinch: PinchUpdate): void => {
    stepGrid(steps.to(pinch.scale), { anchor: { x: pinch.x, y: pinch.y } });
  };
  return pinchRecognizer({
    enabled,
    start: (pinch) => {
      steps.start();
      step(pinch);
    },
    move: step
  });
}

export function watchGridZoom(scroller: HTMLElement): () => void {
  return listenGridZoom(scroller, (steps, at) => stepGrid(steps, { anchor: at, hud: at }));
}
