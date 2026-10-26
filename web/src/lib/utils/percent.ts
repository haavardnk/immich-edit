export function percent(fraction: number): number {
  return Math.round(fraction * 100);
}

export function percentRange(min: number, max: number): string {
  return `${percent(min)}–${percent(max)}`;
}
