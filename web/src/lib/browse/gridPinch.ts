import { stepGrid } from '#lib/browse/gridResize.ts';
import { scaleStepper } from '#lib/browse/gridZoom.ts';
import { listenGridZoom } from '#lib/browse/gridZoomInput.ts';
import { gestures } from '#lib/gestures/gestures.ts';
import { pinchRecognizer, type PinchUpdate } from '#lib/gestures/pinch.ts';

export function gridPinch(enabled: () => boolean): ReturnType<typeof gestures> {
  const steps = scaleStepper();
  const step = (pinch: PinchUpdate): void => {
    stepGrid(steps.to(pinch.scale), { anchor: { x: pinch.x, y: pinch.y } });
  };
  return gestures({
    touchAction: 'pan-y',
    pointerTypes: ['touch'],
    recognizers: [
      pinchRecognizer({
        enabled,
        start: (pinch) => {
          steps.start();
          step(pinch);
        },
        move: step
      })
    ]
  });
}

export function watchGridZoom(scroller: HTMLElement): () => void {
  return listenGridZoom(scroller, (steps, at) => stepGrid(steps, { anchor: at, hud: at }));
}
