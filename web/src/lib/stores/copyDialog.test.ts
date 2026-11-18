import { DEFAULT_COPY_SECTIONS, DEVELOP_KEYS, allSections } from '#lib/editor/copyPaste.ts';
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
