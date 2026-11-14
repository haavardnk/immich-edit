import { neutralEdits } from '#lib/types/edits.ts';
import { FIT_VIEW } from '#lib/utils/imageViewport.ts';
import type { ViewerRender } from '#lib/viewer/protocol.ts';
import { ViewerSync } from '#lib/viewer/sync.svelte.ts';
import { describe, expect, it } from 'vitest';

function render(dragging: boolean): ViewerRender {
  return {
    assetId: 'a1',
    name: 'IMG_1.CR3',
    edits: neutralEdits(),
    dragging,
    proofSpace: 'srgb',
    gamutWarn: false,
    clipWarn: false,
    compare: null
  };
}

describe('ViewerSync', () => {
  it('starts unlinked with nothing to show', () => {
    const sync = new ViewerSync();
    expect(sync.linked).toBe(false);
    expect(sync.render).toBeNull();
    expect(sync.revision).toBe(0);
  });

  it('bumps the revision only when the render changes', () => {
    const sync = new ViewerSync();
    sync.receive({ kind: 'render', render: render(false) });
    sync.receive({ kind: 'render', render: render(false) });
    expect(sync.revision).toBe(1);
    sync.receive({ kind: 'render', render: render(true) });
    expect(sync.revision).toBe(2);
    expect(sync.linked).toBe(true);
  });

  it('tracks the view without a new revision', () => {
    const sync = new ViewerSync();
    const view = { view: FIT_VIEW, srcLong: 6000, split: 0.5, greyCanvas: false };
    sync.receive({ kind: 'view', view });
    expect(sync.view).toEqual(view);
    expect(sync.revision).toBe(0);
    expect(sync.linked).toBe(true);
  });

  it('keeps the last photo but unlinks when the editor leaves', () => {
    const sync = new ViewerSync();
    sync.receive({ kind: 'render', render: render(false) });
    sync.receive({ kind: 'editor-bye' });
    expect(sync.linked).toBe(false);
    expect(sync.render?.assetId).toBe('a1');
  });
});
