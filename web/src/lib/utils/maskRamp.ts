export type RampStop = { offset: number; opacity: number };

const RAMP_STEPS = 8;

export function smoothRamp(from: number, to: number, start: number, end: number): RampStop[] {
  return Array.from({ length: RAMP_STEPS + 1 }, (_, i) => {
    const t = i / RAMP_STEPS;
    const s = t * t * (3 - 2 * t);
    return { offset: from + (to - from) * t, opacity: start + (end - start) * s };
  });
}
