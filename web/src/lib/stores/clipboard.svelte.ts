import type { Edits } from '#lib/types/edits.ts';
import { ALL_COPY_SECTIONS, type CopySections } from '#lib/editor/copyPaste.ts';

export const NOTHING_COPIED = 'Nothing copied';

export const NO_EDITS_TO_COPY = 'This photo has no edits to copy';

export interface ClipboardPayload {
  edits: Edits;
  sections: CopySections;
}

class ClipboardStore {
  private edits: Edits | null = null;
  private sections: CopySections = { ...ALL_COPY_SECTIONS };
  has = $state(false);
  source = $state<string | null>(null);

  copy = (edits: Edits, sections: CopySections, source: string | null = null): void => {
    this.edits = structuredClone(edits) as Edits;
    this.sections = { ...sections };
    this.source = source;
    this.has = true;
  };

  snapshot = (): ClipboardPayload | null =>
    this.edits
      ? { edits: structuredClone(this.edits) as Edits, sections: { ...this.sections } }
      : null;

  clear = (): void => {
    this.edits = null;
    this.source = null;
    this.has = false;
  };
}

export const clipboard = new ClipboardStore();
