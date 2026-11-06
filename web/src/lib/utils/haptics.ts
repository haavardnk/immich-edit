const TICK_MS = 8;

export function hapticTick(): void {
  if (typeof navigator !== 'undefined' && 'vibrate' in navigator) navigator.vibrate(TICK_MS);
}
