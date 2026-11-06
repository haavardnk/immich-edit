export function fitCount(
  widths: readonly number[],
  available: number,
  gap: number,
  overflow: number
): number {
  let used = 0;
  for (const [index, width] of widths.entries()) {
    used += (index > 0 ? gap : 0) + width;
    const reserve = index < widths.length - 1 ? gap + overflow : 0;
    if (used + reserve > available) return Math.max(1, index);
  }
  return widths.length;
}
