import { readStored, writeStored } from '#lib/utils/storage.ts';

const STORAGE_KEY = 'immich-edit:tagTree';

type Persisted = { open: string[] };

function storedOpen(): string[] {
  const open = readStored<Persisted>(STORAGE_KEY)?.open;
  return Array.isArray(open) ? open.filter((key) => typeof key === 'string') : [];
}

class TagTreeOpen {
  #open = $state<ReadonlySet<string>>(new Set(storedOpen()));

  isOpen = (key: string): boolean => this.#open.has(key);

  toggle(key: string): void {
    const next = new Set(this.#open);
    if (!next.delete(key)) next.add(key);
    this.write(next);
  }

  openAll(keys: readonly string[]): void {
    this.write(new Set([...this.#open, ...keys]));
  }

  closeAll(keys: readonly string[]): void {
    const next = new Set(this.#open);
    for (const key of keys) next.delete(key);
    this.write(next);
  }

  allOpen(keys: readonly string[]): boolean {
    return keys.length > 0 && keys.every((key) => this.#open.has(key));
  }

  private write(open: Set<string>): void {
    this.#open = open;
    writeStored(STORAGE_KEY, { open: [...open] } satisfies Persisted);
  }
}

export const tagTreeOpen = new TagTreeOpen();
