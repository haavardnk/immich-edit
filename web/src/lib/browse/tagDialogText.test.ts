import type { TagSummary } from '#lib/api/tags.ts';
import type { DialogTextPart } from '#lib/browse/dialogText.ts';
import { deleteTagText, tagCaption } from '#lib/browse/tagDialogText.ts';
import { describe, expect, it } from 'vitest';

function plain(lines: DialogTextPart[][]): string[] {
  return lines.map((line) =>
    line.map((part) => (typeof part === 'string' ? part : part.strong)).join('')
  );
}

function tag(value: string, assetCount: number): TagSummary {
  return { id: 't', name: value.split('/').at(-1) ?? value, value, assetCount, createdAt: null };
}

describe('deleteTagText', () => {
  it('names the path, the branch and the photos it comes off', () => {
    const text = deleteTagText(tag('Travel/Norway', 640), 3);
    expect(text.title).toBe('Delete tag');
    expect(text.confirm).toBe('Delete tag');
    expect(plain(text.lines)).toEqual([
      'Immich deletes the tag Travel / Norway and its 3 sub-tags.',
      'They come off 640 photos. The photos stay in your library with their other tags.',
      'This cannot be undone.'
    ]);
    expect(text.lines[0]?.[1]).toEqual({ strong: 'Travel / Norway' });
  });

  it('drops the branch for a leaf and says when no photo has it', () => {
    expect(plain(deleteTagText(tag('Film', 1), 0).lines).slice(0, 2)).toEqual([
      'Immich deletes the tag Film.',
      'It comes off 1 photo. The photos stay in your library with their other tags.'
    ]);
    expect(plain(deleteTagText(tag('Film', 0), 0).lines)[1]).toBe('No photos have it.');
  });
});

describe('tagCaption', () => {
  it('lists the parent path, photos and sub-tags that exist', () => {
    expect(tagCaption(tag('Travel/Norway', 640), 3)).toBe('Travel · 640 photos · 3 sub-tags');
    expect(tagCaption(tag('Film', 1), 0)).toBe('1 photo');
    expect(tagCaption({ ...tag('Film', 0), assetCount: null }, 0)).toBe('');
  });
});
