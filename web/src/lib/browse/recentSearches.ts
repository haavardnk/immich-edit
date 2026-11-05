export const MAX_RECENT_SEARCHES = 8;

function sameQuery(a: string, b: string): boolean {
  return a.toLocaleLowerCase() === b.toLocaleLowerCase();
}

function isPrefixEdit(previous: string, query: string): boolean {
  const a = previous.toLocaleLowerCase();
  const b = query.toLocaleLowerCase();
  return a.startsWith(b) || b.startsWith(a);
}

export function withRecent(
  list: readonly string[],
  query: string,
  continues = false,
  max = MAX_RECENT_SEARCHES
): string[] {
  const trimmed = query.trim();
  if (!trimmed) return [...list];
  const [newest, ...older] = list;
  const kept = continues && newest !== undefined && isPrefixEdit(newest, trimmed) ? older : list;
  return [trimmed, ...kept.filter((entry) => !sameQuery(entry, trimmed))].slice(0, max);
}

export function withoutRecent(list: readonly string[], query: string): string[] {
  return list.filter((entry) => entry !== query);
}

export function storedRecents(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  return value
    .filter((entry): entry is string => typeof entry === 'string' && entry.trim() !== '')
    .slice(0, MAX_RECENT_SEARCHES);
}

export function sameRecents(a: readonly string[], b: readonly string[]): boolean {
  return a.length === b.length && a.every((entry, index) => entry === b[index]);
}
