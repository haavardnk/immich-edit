import { describe, it, expect } from 'vitest';
import { visibleSceneClasses } from './masks';
import type { MaskKind } from '$lib/api/masks';

const CLASSES = [
  { id: 'sky', name: 'Sky' },
  { id: 'water', name: 'Water' },
  { id: 'person', name: 'People' }
];

describe('visibleSceneClasses', () => {
  it.each([
    { kinds: ['sky', 'people'], expected: ['water'] },
    { kinds: ['sky'], expected: ['water', 'person'] },
    { kinds: ['semantic'], expected: ['sky', 'water', 'person'] },
    { kinds: [], expected: ['sky', 'water', 'person'] }
  ])('hides $kinds duplicates', ({ kinds, expected }) => {
    const available = kinds.map((kind) => ({ kind: kind as MaskKind }));
    expect(visibleSceneClasses(CLASSES, available).map((c) => c.id)).toEqual(expected);
  });
});
