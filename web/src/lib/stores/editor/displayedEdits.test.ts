import { displayedEdits, type DisplayedCtx } from '#lib/stores/editor/displayedEdits.ts';
import { neutralEdits, originalPreviewEdits, type Edits } from '#lib/types/edits.ts';
import { describe, expect, it } from 'vitest';
import { neutraliseSection } from '#lib/editor/params/neutralise.ts';

function edited(): Edits {
  const edits = neutralEdits();
  edits.basic.exposure_ev = 1.2;
  edits.basic.contrast = 30;
  return edits;
}

function ctx(patch: Partial<DisplayedCtx> = {}): DisplayedCtx {
  return {
    colorPicker: null,
    showingOriginal: false,
    bypassedSection: null,
    editsPreview: null,
    isRaw: true,
    ...patch
  };
}

describe('displayedEdits', () => {
  it('passes the working edits through', () => {
    const edits = edited();
    expect(displayedEdits(ctx(), edits)).toBe(edits);
  });

  it('shows the original while held', () => {
    const edits = edited();
    expect(displayedEdits(ctx({ showingOriginal: true }), edits)).toEqual(
      originalPreviewEdits(edits)
    );
  });

  it('neutralises a bypassed section only', () => {
    const edits = edited();
    const shown = displayedEdits(ctx({ bypassedSection: 'tone', isRaw: false }), edits);
    expect(shown).toEqual(neutraliseSection(edits, 'tone', false));
    expect(shown).not.toEqual(edits);
  });

  it('shows a hovered preview', () => {
    const preview = neutralEdits();
    expect(
      displayedEdits(ctx({ editsPreview: { owner: 'preset', edits: preview } }), edited())
    ).toBe(preview);
  });

  it('drops masks while the colour picker samples', () => {
    const edits = edited();
    const shown = displayedEdits(ctx({ colorPicker: {}, showingOriginal: true }), edits);
    expect(shown.masks).toEqual([]);
    expect(shown.basic.exposure_ev).toBe(1.2);
  });
});
