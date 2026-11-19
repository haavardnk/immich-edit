import {
  liveRecentTags,
  MAX_RECENT_TAGS,
  recentTagChord,
  recentTagSlot,
  sameRecentTags,
  storedRecentTags,
  withRecentTag
} from '#lib/browse/recentTags.ts';
import type { TagRef } from '#lib/types/asset.ts';
import { describe, expect, it } from 'vitest';

function tag(id: string, value = id): TagRef {
  return { id, name: value, value };
}

describe('withRecentTag', () => {
  it('puts the newest first without repeating it', () => {
    expect(withRecentTag([tag('a'), tag('b')], tag('b')).map((t) => t.id)).toEqual(['b', 'a']);
  });

  it('keeps at most eight', () => {
    let list: TagRef[] = [];
    for (let i = 0; i < 10; i++) list = withRecentTag(list, tag(`t${i}`));
    expect(list).toHaveLength(MAX_RECENT_TAGS);
    expect(list[0]?.id).toBe('t9');
  });

  it('never records a tag the app manages', () => {
    expect(withRecentTag([tag('a')], tag('r', 'immich-edit/reject'))).toEqual([tag('a')]);
  });
});

describe('liveRecentTags', () => {
  it('drops deleted tags and takes the current value of the rest', () => {
    const live = [{ id: 'b', name: 'Beach', value: 'Coast/Beach', createdAt: null }];
    expect(liveRecentTags([tag('a'), tag('b')], live)).toEqual([
      { id: 'b', name: 'Beach', value: 'Coast/Beach', parentId: undefined, color: undefined }
    ]);
  });
});

describe('storedRecentTags', () => {
  it('keeps only well-formed entries', () => {
    expect(storedRecentTags([{ id: 'a', value: 'Sky' }, { id: 7 }, 'x', null])).toEqual([
      { id: 'a', name: 'Sky', value: 'Sky' }
    ]);
    expect(storedRecentTags('nope')).toEqual([]);
  });
});

describe('sameRecentTags', () => {
  it('compares ids and values in order', () => {
    expect(sameRecentTags([tag('a')], [tag('a')])).toBe(true);
    expect(sameRecentTags([tag('a')], [tag('a', 'A2')])).toBe(false);
    expect(sameRecentTags([tag('a'), tag('b')], [tag('b'), tag('a')])).toBe(false);
  });
});

describe('recentTagSlot', () => {
  it('maps Digit1 to Digit8 onto the eight slots', () => {
    expect(recentTagSlot('Digit1')).toBe(0);
    expect(recentTagSlot('Digit8')).toBe(7);
    expect(recentTagSlot('Digit9')).toBeNull();
    expect(recentTagSlot('Digit0')).toBeNull();
    expect(recentTagSlot('KeyA')).toBeNull();
  });

  it('labels each slot with its own shortcut', () => {
    expect(recentTagChord(0, false)).toBe('Alt+1');
    expect(recentTagChord(7, true)).toBe('⌥8');
    expect(recentTagChord(8, false)).toBeNull();
  });
});
