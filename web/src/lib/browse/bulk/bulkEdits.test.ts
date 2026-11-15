import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { DEFAULT_COPY_SELECTION } from '#lib/editor/copyTools.ts';
import { clipboard, NO_EDITS_TO_COPY, NOTHING_COPIED } from '#lib/stores/clipboard.svelte.ts';
import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import { asset } from '#lib/testing/assets.ts';
import { neutralEdits } from '#lib/types/edits.ts';
import { bulkEdits, SELECT_ONE_PHOTO } from './bulkEdits.svelte';
import { bulkScope } from './bulkScope.svelte';

const copyPaste = vi.hoisted(() => ({
  copyEditsFrom: vi.fn(async () => {}),
  pasteEditsTo: vi.fn(async () => {}),
  pasteOnlyTo: vi.fn()
}));

vi.mock('#lib/browse/copyPaste.ts', () => copyPaste);

const ITEMS = ['a', 'b', 'c'].map((id) => asset(id));
let detach = (): void => {};

beforeEach(() => {
  selection.clear();
  clipboard.clear();
  editedThumbs.remove('a');
  detach = bulkScope.attach({
    items: () => ITEMS,
    selectingAll: () => false,
    selectAll: async () => true,
    cancelSelectAll: () => {},
    partial: () => false,
    loadingMore: () => false,
    total: () => ITEMS.length
  });
});

afterEach(() => {
  detach();
  vi.clearAllMocks();
});

describe('bulkEdits', () => {
  it('blocks every row while nothing is selected', () => {
    for (const row of [
      bulkEdits.applyPreset,
      bulkEdits.paste,
      bulkEdits.pasteOnly,
      bulkEdits.copy,
      bulkEdits.reset
    ]) {
      expect(row.disabled).toBe(true);
    }
  });

  it('says nothing is copied on both paste rows until edits are copied', () => {
    selection.replace(['a', 'b']);
    expect(bulkEdits.paste).toMatchObject({ disabled: true, detail: NOTHING_COPIED });
    expect(bulkEdits.pasteOnly).toMatchObject({ disabled: true, detail: NOTHING_COPIED });

    clipboard.copy(neutralEdits(), DEFAULT_COPY_SELECTION, 'DSCF4120.RAF');
    expect(bulkEdits.paste).toMatchObject({ disabled: false, detail: 'from DSCF4120' });
    expect(bulkEdits.pasteOnly).toMatchObject({ disabled: false, detail: null });
  });

  it('copies only from one selected photo that has edits', () => {
    selection.replace(['a', 'b']);
    expect(bulkEdits.copy).toMatchObject({ disabled: true, detail: SELECT_ONE_PHOTO });

    selection.replace(['a']);
    expect(bulkEdits.copy).toMatchObject({ disabled: true, detail: NO_EDITS_TO_COPY });

    editedThumbs.upsert('a', 'h1', '2024-05-01T00:00:00Z');
    expect(bulkEdits.copy).toMatchObject({ disabled: false, detail: null });
    bulkEdits.copy.run();
    expect(copyPaste.copyEditsFrom).toHaveBeenCalledWith('a');
  });

  it('pastes onto the whole selection', async () => {
    selection.replace(['a', 'c']);
    clipboard.copy(neutralEdits(), DEFAULT_COPY_SELECTION);
    bulkEdits.paste.run();
    await vi.waitFor(() => expect(copyPaste.pasteEditsTo).toHaveBeenCalledWith(['a', 'c']));
    bulkEdits.pasteOnly.run();
    expect(copyPaste.pasteOnlyTo).toHaveBeenCalledWith(['a', 'c']);
  });
});
