import type { PillText } from '#lib/editor/pill.ts';
import { keyHeld, type ModifierKeys } from '#lib/input/modifiers.svelte.ts';
import { keysFor } from '#lib/shortcuts/labels.ts';
import type { BrushTool } from '#lib/stores/ui.svelte.ts';

type BrushMode = BrushTool['mode'];

class BrushErase {
  held = $state(false);

  note = (e: ModifierKeys): void => {
    this.held = keyHeld('erase', e);
  };

  release = (): void => {
    this.held = false;
  };

  mode(chosen: BrushMode, e?: ModifierKeys): BrushMode {
    return this.held || (e !== undefined && keyHeld('erase', e)) ? 'erase' : chosen;
  }

  get pill(): PillText | null {
    if (!this.held) return null;
    return { lead: `${keysFor('brushEraseHold')} ·`, label: 'Erase', tail: '', mono: false };
  }
}

export const brushErase = new BrushErase();
