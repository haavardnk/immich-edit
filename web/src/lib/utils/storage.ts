export function readStored<T>(key: string): Partial<T> | null {
  if (typeof localStorage === 'undefined') return null;
  try {
    const raw = localStorage.getItem(key);
    if (!raw) return null;
    return JSON.parse(raw) as Partial<T>;
  } catch {
    return null;
  }
}

export function writeStored(key: string, value: unknown): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(key, JSON.stringify(value));
}
