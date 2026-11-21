import type { TagSummary } from '#lib/api/tags.ts';
import {
  createdTagPath,
  siblingTags,
  tagDetailsPatch,
  tagNameProblem
} from '#lib/browse/tagDetails.ts';
import { DISPLAY_NAME_MAX } from '#lib/utils/displayName.ts';
import { describe, expect, it } from 'vitest';

function tag(id: string, value: string, color: string | null = null): TagSummary {
  return { id, name: value.split('/').at(-1) ?? value, value, color, createdAt: null };
}

const TAGS = [
  tag('travel', 'Travel'),
  tag('norway', 'Travel/Norway', '#3e8ed0'),
  tag('germany', 'Travel/Germany'),
  tag('bergen', 'Travel/Norway/Bergen')
];

describe('tagNameProblem', () => {
  const siblings = siblingTags(TAGS, 'Travel');

  it('finds siblings by parent path', () => {
    expect(siblings.map((t) => t.id)).toEqual(['norway', 'germany']);
    expect(siblingTags(TAGS, '').map((t) => t.id)).toEqual(['travel']);
  });

  it('refuses a slash, an overlong name and a sibling with the same name', () => {
    expect(tagNameProblem('Svalbard', siblings, 'Travel', null)).toBeNull();
    expect(tagNameProblem('  ', siblings, 'Travel', null)).toBeNull();
    expect(tagNameProblem('a/b', siblings, 'Travel', null)).toBe('Tag names cannot contain /');
    expect(tagNameProblem('x'.repeat(DISPLAY_NAME_MAX + 1), siblings, 'Travel', null)).toBe(
      'Use at most 80 characters'
    );
    expect(tagNameProblem(' germany ', siblings, 'Travel', 'norway')).toBe(
      'Travel / Germany already exists'
    );
    expect(tagNameProblem('Tab\there', siblings, 'Travel', null)).toBe(
      'Tag names cannot contain control characters'
    );
  });

  it('folds case the same way as the backend, beyond ASCII', () => {
    const nordic = [tag('al', 'Ål')];
    expect(tagNameProblem('ål', nordic, '', null)).toBe('Ål already exists');
  });

  it('lets a tag keep its own name and reserves the managed root at the top', () => {
    expect(tagNameProblem('NORWAY', siblings, 'Travel', 'norway')).toBeNull();
    expect(tagNameProblem('Immich-Edit', [], '', null)).toBe(
      "immich-edit is kept for immich-edit's own tags"
    );
    expect(tagNameProblem('immich-edit', [], 'Travel', null)).toBeNull();
  });
});

describe('tagDetailsPatch', () => {
  const norway = TAGS[1] as TagSummary;

  it('sends only what changed and clears a color with null', () => {
    expect(tagDetailsPatch(norway, ' Norway ', '#3E8ED0', true)).toBeNull();
    expect(tagDetailsPatch(norway, 'Norge', '#3e8ed0', true)).toEqual({ name: 'Norge' });
    expect(tagDetailsPatch(norway, 'Norway', null, true)).toEqual({ color: null });
  });

  it('never renames when the server cannot', () => {
    expect(tagDetailsPatch(norway, 'Norge', '#e5484d', false)).toEqual({ color: '#e5484d' });
  });
});

describe('createdTagPath', () => {
  it('names the tag that will be created', () => {
    expect(createdTagPath('Travel/Norway', ' Svalbard ')).toBe('Travel / Norway / Svalbard');
    expect(createdTagPath('', 'Film')).toBe('Film');
  });
});
