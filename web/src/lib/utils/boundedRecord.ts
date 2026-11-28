export interface BoundedRecord<T> {
  record: Record<string, T>;
  order: string[];
}

export function putAllBounded<T>(
  record: Record<string, T>,
  order: string[],
  entries: readonly (readonly [string, T])[],
  max: number
): BoundedRecord<T> {
  const nextRecord: Record<string, T> = { ...record };
  const fresh = new Set(entries.map(([key]) => key));
  const nextOrder = order.filter((k) => !fresh.has(k));
  for (const [key, value] of entries) nextRecord[key] = value;
  nextOrder.push(...fresh);
  while (nextOrder.length > max) {
    const evicted = nextOrder.shift();
    if (evicted !== undefined) delete nextRecord[evicted];
  }
  return { record: nextRecord, order: nextOrder };
}

export function putBounded<T>(
  record: Record<string, T>,
  order: string[],
  key: string,
  value: T,
  max: number
): BoundedRecord<T> {
  return putAllBounded(record, order, [[key, value]], max);
}
