import { describe, it, expect } from 'vitest';
import { numberRepeats, visibleSceneClasses } from './masks';
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

describe('numberRepeats', () => {
  it.each([
    { labels: ['Brush', 'Sky', 'Brush'], expected: ['Brush 1', 'Sky', 'Brush 2'] },
    { labels: ['Brush', 'Sky'], expected: ['Brush', 'Sky'] },
    { labels: [], expected: [] }
  ])('numbers only repeated labels', ({ labels, expected }) => {
    expect(numberRepeats(labels)).toEqual(expected);
  });
});
