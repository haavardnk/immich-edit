import {
  COPY_TOOLS,
  DEFAULT_COPY_SELECTION,
  DEVELOP_TOOLS,
  selectedTools,
  selectionOf
} from '#lib/editor/copyTools.ts';
import { clipboard } from '#lib/stores/clipboard.svelte.ts';
import { neutralEdits } from '#lib/types/edits.ts';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { copyDialog } from './copyDialog.svelte';

afterEach(() => {
  copyDialog.cancel();
  copyDialog.selection = { ...DEFAULT_COPY_SELECTION };
  clipboard.clear();
});

describe('copyDialog tool setters', () => {
  it('sets tools without touching the others', () => {
    copyDialog.setAll(false);
    copyDialog.setTools(['masks', 'hsl'], true);
    expect(selectedTools(copyDialog.selection)).toEqual(['hsl', 'masks']);
  });

  it('sets every tool at once', () => {
    copyDialog.setAll(true);
    expect(copyDialog.selection).toEqual(selectionOf(COPY_TOOLS));
    copyDialog.setAll(false);
    expect(copyDialog.count).toBe(0);
  });

  it('picks exactly the given tools', () => {
    copyDialog.pickOnly(['curves']);
    expect(selectedTools(copyDialog.selection)).toEqual(['curves']);
  });
});

describe('copyDialog copy', () => {
  it('names the photo the edits came from on the clipboard', () => {
    copyDialog.show(neutralEdits(), undefined, 'IMG_4120.ARW');
    copyDialog.confirm();
    expect(clipboard.has).toBe(true);
    expect(clipboard.source).toBe('IMG_4120.ARW');
  });

  it('forgets the source photo when the clipboard clears', () => {
    copyDialog.show(neutralEdits(), undefined, 'IMG_4120.ARW');
    copyDialog.confirm();
    clipboard.clear();
    expect(clipboard.has).toBe(false);
    expect(clipboard.source).toBeNull();
  });

  it('does not copy with nothing checked', () => {
    copyDialog.show(neutralEdits());
    copyDialog.setAll(false);
    copyDialog.confirm();
    expect(clipboard.has).toBe(false);
    expect(copyDialog.open).toBe(true);
  });

  it('keeps the choice for the next copy', () => {
    copyDialog.show(neutralEdits());
    copyDialog.pickOnly(['hsl']);
    copyDialog.confirm();
    copyDialog.show(neutralEdits());
    expect(selectedTools(copyDialog.checked)).toEqual(['hsl']);
  });
});

describe('copyDialog paste only', () => {
  it('offers only the copied tools and pastes the checked ones', () => {
    clipboard.copy(neutralEdits(), selectionOf(['hsl', 'curves', 'masks']), 'IMG_1.ARW', 'hsl');
    const snap = clipboard.snapshot();
    if (!snap) throw new Error('clipboard empty');
    const onPaste = vi.fn();

    copyDialog.showPaste(snap, onPaste, clipboard.source);
    expect(copyDialog.available).toEqual(['curves', 'hsl', 'masks']);
    expect(copyDialog.touched).toBe('hsl');

    copyDialog.setAll(true);
    copyDialog.setTools(['curves', 'grain'], false);
    copyDialog.confirm();

    expect(onPaste).toHaveBeenCalledWith(selectionOf(['hsl', 'masks']));
    expect(copyDialog.open).toBe(false);
    expect(selectedTools(copyDialog.selection)).toEqual(DEVELOP_TOOLS);
  });
});
