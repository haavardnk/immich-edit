import type { Edits } from '#lib/types/edits.ts';

export type EditsPreviewOwner = 'preset' | 'history';

export interface EditsPreview {
  owner: EditsPreviewOwner;
  edits: Edits;
}
