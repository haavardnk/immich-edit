export type Coverage = 'none' | 'some' | 'all';

export function coverage<T>(known: T[], selected: number, has: (item: T) => boolean): Coverage {
  const hits = known.filter(has).length;
  if (hits === selected) return 'all';
  if (hits === 0 && known.length === selected) return 'none';
  return 'some';
}

export function commonValue<T, V>(known: T[], selected: number, value: (item: T) => V): V | null {
  const first = known[0];
  if (first === undefined || known.length < selected) return null;
  const want = value(first);
  return known.every((item) => value(item) === want) ? want : null;
}
