import type { Edits } from '#lib/types/edits.ts';
import {
  DEFAULT_COPY_SELECTION,
  type CopySelection,
  type CopyTool
} from '#lib/editor/copyTools.ts';

export const NOTHING_COPIED = 'Nothing copied';

export const NO_EDITS_TO_COPY = 'This photo has no edits to copy';

export interface ClipboardPayload {
  edits: Edits;
  tools: CopySelection;
  touched: CopyTool | null;
}

class ClipboardStore {
  private edits: Edits | null = null;
  private tools: CopySelection = { ...DEFAULT_COPY_SELECTION };
  private touched: CopyTool | null = null;
  has = $state(false);
  source = $state<string | null>(null);

  copy = (
    edits: Edits,
    tools: CopySelection,
    source: string | null = null,
    touched: CopyTool | null = null
  ): void => {
    this.edits = structuredClone(edits) as Edits;
    this.tools = { ...tools };
    this.touched = touched;
    this.source = source;
    this.has = true;
  };

  snapshot = (): ClipboardPayload | null =>
    this.edits
      ? {
          edits: structuredClone(this.edits) as Edits,
          tools: { ...this.tools },
          touched: this.touched
        }
      : null;

  clear = (): void => {
    this.edits = null;
    this.touched = null;
    this.source = null;
    this.has = false;
  };
}

export const clipboard = new ClipboardStore();
