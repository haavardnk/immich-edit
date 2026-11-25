export type StorageScope = 'local' | 'session';

function area(scope: StorageScope): Storage | null {
  if (scope === 'session') return typeof sessionStorage === 'undefined' ? null : sessionStorage;
  return typeof localStorage === 'undefined' ? null : localStorage;
}

export function readStored<T>(key: string, scope: StorageScope = 'local'): Partial<T> | null {
  try {
    const raw = area(scope)?.getItem(key);
    if (!raw) return null;
    return JSON.parse(raw) as Partial<T>;
  } catch {
    return null;
  }
}

export function writeStored(key: string, value: unknown, scope: StorageScope = 'local'): void {
  try {
    area(scope)?.setItem(key, JSON.stringify(value));
  } catch {
    return;
  }
}
