import type { RatingMatch } from '$lib/types/search';

export type Star = 1 | 2 | 3 | 4 | 5;
export type RatingOp = 'eq' | 'gte' | 'lte';
export type RangeRating = `${Star}+` | `${Star}-`;
export type RatingFilter = 'any' | 'unrated' | Star | RangeRating;

export const STARS: Star[] = [1, 2, 3, 4, 5];
export const EXACT_RATINGS: RatingFilter[] = ['any', 'unrated', ...STARS];
export const RANGE_RATINGS: RangeRating[] = STARS.flatMap((n): RangeRating[] => [`${n}+`, `${n}-`]);

export function isRange(rating: RatingFilter): rating is RangeRating {
  return RANGE_RATINGS.some((range) => range === rating);
}

export function ratingStars(rating: RatingFilter): Star | 0 {
  if (typeof rating === 'number') return rating;
  return STARS.find((n) => rating.startsWith(String(n))) ?? 0;
}

export function ratingOp(rating: RatingFilter): RatingOp | null {
  if (typeof rating === 'number') return 'eq';
  if (rating.endsWith('+')) return 'gte';
  if (rating.endsWith('-')) return 'lte';
  return null;
}

export function starRating(stars: Star, op: RatingOp): RatingFilter {
  if (op === 'eq') return stars;
  return `${stars}${op === 'gte' ? '+' : '-'}`;
}

export function ratingMatch(rating: RatingFilter): RatingMatch | undefined {
  if (rating === 'any') return undefined;
  if (rating === 'unrated') return 'unrated';
  if (typeof rating === 'number') return { eq: rating };
  const stars = ratingStars(rating);
  return rating.endsWith('+') ? { gte: stars } : { lte: stars };
}

export function ratingHides(rating: RatingFilter, own: number | null | undefined): boolean {
  const stars = ratingStars(rating);
  const op = ratingOp(rating);
  if (op === 'gte') return (own ?? 0) < stars;
  if (op === 'lte') return (own ?? 0) > stars;
  return false;
}
