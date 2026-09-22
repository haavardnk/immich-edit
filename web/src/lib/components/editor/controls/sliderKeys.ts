type ArrowKeyEvent = {
  key: string;
  shiftKey: boolean;
  altKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
};

type StepOptions = {
  value: number;
  min: number;
  max: number;
  step: number;
  coarseStep: number;
};

const DIRECTIONS: Record<string, number> = {
  ArrowRight: 1,
  ArrowUp: 1,
  ArrowLeft: -1,
  ArrowDown: -1
};

function decimalsOf(step: number): number {
  const text = String(step);
  const dot = text.indexOf('.');
  return dot < 0 ? 0 : text.length - dot - 1;
}

export function arrowStepValue(event: ArrowKeyEvent, options: StepOptions): number | null {
  const direction = DIRECTIONS[event.key];
  if (!direction || !event.shiftKey || event.altKey || event.ctrlKey || event.metaKey) return null;
  const raw = options.value + direction * options.coarseStep;
  const next = Math.min(options.max, Math.max(options.min, raw));
  return Number(next.toFixed(decimalsOf(options.step)));
}
