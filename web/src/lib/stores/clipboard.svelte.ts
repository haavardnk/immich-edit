import type { Edits } from '$lib/types/edits';
import { ALL_COPY_SECTIONS, type CopySections } from '$lib/copyPaste';

export interface ClipboardPayload {
  edits: Edits;
  sections: CopySections;
}

class ClipboardStore {
  private edits: Edits | null = null;
  private sections: CopySections = { ...ALL_COPY_SECTIONS };
  has = $state(false);

  copy = (edits: Edits, sections: CopySections): void => {
    this.edits = structuredClone(edits) as Edits;
    this.sections = { ...sections };
    this.has = true;
  };

  snapshot = (): ClipboardPayload | null =>
    this.edits
      ? { edits: structuredClone(this.edits) as Edits, sections: { ...this.sections } }
      : null;

  clear = (): void => {
    this.edits = null;
    this.has = false;
  };
}

export const clipboard = new ClipboardStore();
