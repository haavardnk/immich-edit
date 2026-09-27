export function nextRatingFromKey(
  key: string,
  currentRating: number | null | undefined
): number | null | undefined {
  if (key === '0') return null;
  if (key < '1' || key > '5') return undefined;
  const n = Number(key);
  if (n === currentRating) return null;
  return n;
}

export function ratingFromCode(code: string): number | null | undefined {
  const digit = /^Digit([0-5])$/.exec(code);
  if (!digit) return undefined;
  const n = Number(digit[1]);
  return n === 0 ? null : n;
}
