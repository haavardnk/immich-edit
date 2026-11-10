import { listEditHistory, type EditHistoryEntry } from '#lib/api/edits.ts';
import { SvelteMap } from 'svelte/reactivity';

export class VersionSummaries {
  private entries = new SvelteMap<string, readonly EditHistoryEntry[]>();
  private requested = new Set<string>();
  private generation = 0;

  get(id: string): readonly EditHistoryEntry[] | undefined {
    return this.entries.get(id);
  }

  load(ids: readonly string[]): void {
    const generation = this.generation;
    for (const id of ids) {
      if (this.requested.has(id)) continue;
      this.requested.add(id);
      void listEditHistory(id).then(
        (entries) => {
          if (generation === this.generation) this.entries.set(id, entries);
        },
        () => undefined
      );
    }
  }

  forget(id: string): void {
    this.requested.delete(id);
    this.entries.delete(id);
  }

  clear(): void {
    this.generation++;
    this.requested.clear();
    this.entries.clear();
  }
}
