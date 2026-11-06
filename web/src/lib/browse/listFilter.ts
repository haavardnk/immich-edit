import { countOf } from '#lib/utils/plural.ts';
import { textMatches } from '#lib/utils/textMatch.ts';

export const LIST_FILTER_MIN = 12;

export interface ListNoun {
  one: string;
  many: string;
}

export function listFilterShown(total: number): boolean {
  return total > LIST_FILTER_MIN;
}

export function filterByName<T>(
  items: readonly T[],
  query: string,
  nameOf: (item: T) => string
): T[] {
  return items.filter((item) => textMatches(nameOf(item), query));
}

export function listFilterStatus(shown: number, total: number, noun: ListNoun): string {
  if (shown === 0) return `No matching ${noun.many}`;
  return `${shown} of ${countOf(total, noun.one, noun.many)}`;
}
