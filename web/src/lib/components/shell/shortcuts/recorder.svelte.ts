import { chordSpec, isModifierOnly } from '#lib/shortcuts/chord.ts';
import {
  bindKey,
  browserOwned,
  conflictOf,
  keysOf,
  sameChord,
  swapKey,
  type Conflict
} from '#lib/shortcuts/custom.ts';
import { preferences } from '#lib/stores/preferences.svelte.ts';

export type RecordNotice = 'unsupported' | 'browser';

export interface PendingSwap {
  spec: string;
  conflict: Conflict;
}

export class ShortcutRecorder {
  id = $state<string | null>(null);
  pending = $state.raw<PendingSwap | null>(null);
  notice = $state<RecordNotice | null>(null);

  start = (id: string): void => {
    this.id = id;
    this.pending = null;
    this.notice = null;
  };

  cancel = (): void => {
    this.id = null;
    this.pending = null;
    this.notice = null;
  };

  keydown = (e: KeyboardEvent): boolean => {
    const id = this.id;
    if (!id) return false;
    e.preventDefault();
    if (e.key === 'Escape') {
      this.cancel();
      return true;
    }
    if (e.repeat || isModifierOnly(e)) return true;
    const spec = chordSpec(e);
    if (!spec) {
      this.notice = 'unsupported';
      return true;
    }
    if (browserOwned(spec)) {
      this.notice = 'browser';
      return true;
    }
    this.record(id, spec);
    return true;
  };

  swap = (): void => {
    const id = this.id;
    const pending = this.pending;
    if (!id || !pending) return;
    const custom = preferences.shortcuts;
    void preferences.setShortcuts(swapKey(custom, id, pending.spec, pending.conflict.bind.id));
    this.cancel();
  };

  private record(id: string, spec: string): void {
    const custom = preferences.shortcuts;
    const current = keysOf(id, custom);
    if (current.length === 1 && current[0] !== undefined && sameChord(current[0], spec)) {
      this.cancel();
      return;
    }
    const conflict = conflictOf(id, spec, custom);
    this.notice = null;
    if (conflict) {
      this.pending = { spec, conflict };
      return;
    }
    void preferences.setShortcuts(bindKey(custom, id, spec));
    this.cancel();
  }
}
