export const HUE_GRADIENT = 'var(--gradient-hue-linear)';

export const BALANCE_GRADIENT = 'var(--gradient-grading-balance)';

export function saturationGradient(hue: number): string {
  return `linear-gradient(to right, hsl(${hue}, 0%, 50%), hsl(${hue}, 100%, 50%))`;
}

export function luminanceGradient(hue: number, sat: number): string {
  return `linear-gradient(to right, hsl(${hue}, ${sat}%, 0%), hsl(${hue}, ${sat}%, 50%), hsl(${hue}, ${sat}%, 100%))`;
}
