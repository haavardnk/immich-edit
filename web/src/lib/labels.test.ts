import { describe, expect, it } from 'vitest';
import { isLabelTag, labelOf, labelTagValue, nextLabelFromKey, withLabel } from './labels';
import type { TagRef } from './types/asset';

const keep: TagRef = { id: 'k', name: 'Keep', value: 'Keep' };
const red: TagRef = { id: 'r', name: 'red', value: 'immich-edit/label/red' };
const blue: TagRef = { id: 'b', name: 'blue', value: 'immich-edit/label/blue' };

describe('labels', () => {
  it('names each colour under the managed namespace', () => {
    expect(labelTagValue('purple')).toBe('immich-edit/label/purple');
  });

  it('reads the label from the tags and ignores unknown colours', () => {
    expect(labelOf({ tags: [keep, blue] })).toBe('blue');
    expect(labelOf({ tags: [{ id: 'x', name: 'x', value: 'IMMICH-EDIT/LABEL/RED' }] })).toBe('red');
    expect(labelOf({ tags: [{ id: 'x', name: 'x', value: 'immich-edit/label/teal' }] })).toBeNull();
    expect(labelOf({ tags: null })).toBeNull();
    expect(isLabelTag(keep)).toBe(false);
  });

  it('keeps one label per photo', () => {
    expect(withLabel([keep, red], blue)).toEqual([keep, blue]);
    expect(withLabel([keep, red, blue], null)).toEqual([keep]);
  });

  it.each([
    ['6', null, 'red'],
    ['9', 'red', 'blue'],
    ['7', 'yellow', null],
    ['5', null, undefined]
  ] as const)('key %s on %s gives %s', (key, current, next) => {
    expect(nextLabelFromKey(key, current)).toBe(next);
  });
});
