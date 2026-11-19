import { heldAlone, unmodified, type ModifierKeys } from '#lib/input/modifiers.svelte.ts';
import { clampTo, decimalsOf } from './sliderStep';

type ArrowKeyEvent = ModifierKeys & { key: string };

type StepOptions = {
  value: number;
  min: number;
  max: number;
  step: number;
  fineStep: number;
  coarseStep: number;
};

const DIRECTIONS: Record<string, number> = {
  ArrowRight: 1,
  ArrowUp: 1,
  ArrowLeft: -1,
  ArrowDown: -1
};

function stride(event: ArrowKeyEvent, options: StepOptions): number | null {
  if (heldAlone('coarse', event)) return options.coarseStep;
  if (heldAlone('fine', event)) return options.fineStep;
  return unmodified(event) ? options.step : null;
}

export function arrowStepValue(event: ArrowKeyEvent, options: StepOptions): number | null {
  const direction = DIRECTIONS[event.key];
  if (!direction) return null;
  const size = stride(event, options);
  if (size === null) return null;
  const next = clampTo(options.value + direction * size, options.min, options.max);
  return Number(next.toFixed(decimalsOf(options.fineStep)));
}
