const COPY_KEY = /^([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})_([1-9][0-9]*)$/i;

export function copyIndex(id: string): number | null {
  const match = COPY_KEY.exec(id);
  return match ? Number(match[2]) : null;
}

export function isCopy(id: string): boolean {
  return COPY_KEY.test(id);
}

export function sourceId(id: string): string {
  const match = COPY_KEY.exec(id);
  return match ? match[1] : id;
}
