export interface BoundedRecord<T> {
  record: Record<string, T>;
  order: string[];
}

export function putBounded<T>(
  record: Record<string, T>,
  order: string[],
  key: string,
  value: T,
  max: number
): BoundedRecord<T> {
  const nextRecord: Record<string, T> = { ...record, [key]: value };
  const nextOrder = order.filter((k) => k !== key);
  nextOrder.push(key);
  while (nextOrder.length > max) {
    const evicted = nextOrder.shift();
    if (evicted !== undefined && evicted !== key) delete nextRecord[evicted];
  }
  return { record: nextRecord, order: nextOrder };
}
