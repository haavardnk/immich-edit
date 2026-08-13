import { describe, it, expect } from 'vitest';
import { generatedLabel, kindLabel, manualKind, numberRepeats, visibleSceneClasses } from './masks';
import type { MaskKind } from '$lib/api/masks';
import type { ManualTool } from './masks';

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

describe('manualKind', () => {
  it.each([
    { tool: 'linear', expected: 'linear' },
    { tool: 'radial', expected: 'radial' },
    { tool: 'luma_range', expected: 'luma_range' },
    { tool: 'color_range', expected: 'color_range' },
    { tool: 'brush', expected: null },
    { tool: 'polygon', expected: null }
  ])('maps $tool to a default shape', ({ tool, expected }) => {
    expect(manualKind(tool as ManualTool)?.kind ?? null).toBe(expected);
  });
});

describe('labels', () => {
  it.each([
    { kind: 'linear', expected: 'Linear gradient' },
    { kind: 'brush', expected: 'Brush' },
    { kind: 'color_range', expected: 'Color range' }
  ])('names the $kind shape', ({ kind, expected }) => {
    expect(kindLabel(manualKind(kind as ManualTool) ?? { kind: 'brush', raster_id: '' })).toBe(
      expected
    );
  });

  it.each([
    { kind: 'semantic', expected: 'Scene' },
    { kind: 'sky', expected: 'Sky' },
    { kind: 'unknown', expected: 'unknown' }
  ])('names the $kind model', ({ kind, expected }) => {
    expect(generatedLabel(kind)).toBe(expected);
  });
});
