import { neutraliseSection } from '#lib/editor/params/neutralise.ts';
import { originalPreviewEdits, type DevelopSection, type Edits } from '#lib/types/edits.ts';
import type { EditsPreview } from '#lib/stores/editor/editsPreview.ts';

export interface DisplayedCtx {
  readonly rangePicker: unknown;
  readonly showingOriginal: boolean;
  readonly bypassedSection: DevelopSection | null;
  readonly editsPreview: EditsPreview | null;
  readonly isRaw: boolean;
}

export function displayedEdits(ctx: DisplayedCtx, edits: Edits): Edits {
  if (ctx.rangePicker) return { ...edits, masks: [] };
  if (ctx.showingOriginal) return originalPreviewEdits(edits);
  if (ctx.bypassedSection) return neutraliseSection(edits, ctx.bypassedSection, ctx.isRaw);
  if (ctx.editsPreview) return ctx.editsPreview.edits;
  return edits;
}
