import { beforeEach, describe, expect, it, vi } from 'vitest';
import { addTagToAsset, removeTagFromAsset } from '#lib/api/tags.ts';
import {
  isLabelTag,
  labelOf,
  labelTagValue,
  nextLabelFromKey,
  withLabel,
  writeLabel
} from './labels';
import type { TagRef } from '#lib/types/asset.ts';

vi.mock('#lib/api/tags.ts', () => ({ addTagToAsset: vi.fn(), removeTagFromAsset: vi.fn() }));

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

describe('writeLabel', () => {
  const add = vi.mocked(addTagToAsset);
  const remove = vi.mocked(removeTagFromAsset);
  const calls = (): string[] => [
    ...add.mock.calls.map(([tag]) => `add ${tag}`),
    ...remove.mock.calls.map(([tag]) => `remove ${tag}`)
  ];

  beforeEach(() => {
    add.mockReset().mockResolvedValue([]);
    remove.mockReset().mockResolvedValue([]);
  });

  it('adds the new label before removing the old one', async () => {
    await writeLabel('a', [keep, red], blue);
    expect(add.mock.invocationCallOrder[0]).toBeLessThan(remove.mock.invocationCallOrder[0] ?? 0);
    expect(calls()).toEqual(['add b', 'remove r']);
  });

  it('keeps the old label when the new one cannot be added', async () => {
    add.mockRejectedValue(new Error('offline'));
    await expect(writeLabel('a', [red], blue)).rejects.toThrow('offline');
    expect(remove).not.toHaveBeenCalled();
  });

  it('takes the new label back off when the old one cannot be removed', async () => {
    remove.mockRejectedValueOnce(new Error('offline'));
    await expect(writeLabel('a', [red], blue)).rejects.toThrow('offline');
    expect(calls()).toEqual(['add b', 'remove r', 'remove b']);
  });

  it('only removes when the label is cleared', async () => {
    await writeLabel('a', [keep, red], null);
    expect(calls()).toEqual(['remove r']);
  });
});
