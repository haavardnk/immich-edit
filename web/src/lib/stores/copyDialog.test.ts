import { DEFAULT_COPY_SECTIONS, DEVELOP_KEYS, allSections } from '#lib/editor/copyPaste.ts';
import { clipboard } from '#lib/stores/clipboard.svelte.ts';
import { neutralEdits } from '#lib/types/edits.ts';
import { afterEach, describe, expect, it } from 'vitest';
import { copyDialog } from './copyDialog.svelte';

describe('copyDialog section setters', () => {
  afterEach(() => {
    copyDialog.sections = { ...DEFAULT_COPY_SECTIONS };
  });

  it('sets one section without touching the others', () => {
    copyDialog.setAll(false);
    copyDialog.setSection('masks', true);
    expect(copyDialog.sections).toEqual({ ...allSections(false), masks: true });
  });

  it('sets every develop section together', () => {
    copyDialog.setAll(false);
    copyDialog.setDevelop(true);
    for (const key of DEVELOP_KEYS) expect(copyDialog.sections[key]).toBe(true);
    expect(copyDialog.sections.geometry).toBe(false);
    expect(copyDialog.sections.masks).toBe(false);
    expect(copyDialog.sections.retouch).toBe(false);
  });

  it('sets all sections at once', () => {
    copyDialog.setAll(true);
    expect(copyDialog.sections).toEqual(allSections(true));
    copyDialog.setAll(false);
    expect(copyDialog.sections).toEqual(allSections(false));
  });
});

describe('copyDialog source', () => {
  afterEach(() => clipboard.clear());

  it('names the photo the edits came from on the clipboard', () => {
    copyDialog.show(neutralEdits(), undefined, 'IMG_4120.ARW');
    copyDialog.confirm();
    expect(clipboard.has).toBe(true);
    expect(clipboard.source).toBe('IMG_4120.ARW');
  });

  it('forgets the source when the clipboard clears', () => {
    copyDialog.show(neutralEdits(), undefined, 'IMG_4120.ARW');
    copyDialog.confirm();
    clipboard.clear();
    expect(clipboard.source).toBeNull();
  });

  it('counts the copied sections until the clipboard clears', () => {
    copyDialog.show(neutralEdits());
    copyDialog.confirm();
    expect(clipboard.sectionCount).toBe(DEVELOP_KEYS.length);
    clipboard.clear();
    expect(clipboard.sectionCount).toBe(0);
  });
});
