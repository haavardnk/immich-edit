import type { Edits } from '#lib/types/edits.ts';

export type EditsPreviewOwner = 'preset' | 'history' | 'lut';

export interface EditsPreview {
  owner: EditsPreviewOwner;
  edits: Edits;
}
