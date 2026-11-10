import { describe, expect, it } from 'vitest';
import {
  availableMode,
  compareLabel,
  compareName,
  comparePane,
  compareViewport,
  nextCompareMode,
  ORIGINAL_SOURCE,
  preferredMode,
  sameSource,
  type CompareMode
} from '#lib/editor/compare.ts';

const SNAPSHOTS = [{ id: 'snap-1', name: 'Warm v1' }];
const VERSIONS = [{ id: 'asset-1_1', name: 'B&W' }];

describe('compare modes', () => {
  it('cycles split, side by side, off where side by side fits', () => {
    const seen: CompareMode[] = [];
    let mode: CompareMode = 'off';
    for (let step = 0; step < 4; step++) {
      mode = nextCompareMode(mode, true);
      seen.push(mode);
    }
    expect(seen).toEqual(['split', 'side', 'off', 'split']);
  });

  it('cycles only split and off without side by side', () => {
    expect(nextCompareMode('off', false)).toBe('split');
    expect(nextCompareMode('split', false)).toBe('off');
    expect(nextCompareMode('side', false)).toBe('off');
  });

  it('falls back to split and prefers side by side where it fits', () => {
    expect(availableMode('side', false)).toBe('split');
    expect(availableMode('side', true)).toBe('side');
    expect(availableMode('off', false)).toBe('off');
    expect(preferredMode(true)).toBe('side');
    expect(preferredMode(false)).toBe('split');
  });
});

describe('compare sources', () => {
  it('matches sources by kind and id', () => {
    expect(sameSource(ORIGINAL_SOURCE, { kind: 'original' })).toBe(true);
    expect(sameSource({ kind: 'snapshot', id: 'a' }, { kind: 'snapshot', id: 'a' })).toBe(true);
    expect(sameSource({ kind: 'snapshot', id: 'a' }, { kind: 'snapshot', id: 'b' })).toBe(false);
    expect(sameSource({ kind: 'snapshot', id: 'a' }, ORIGINAL_SOURCE)).toBe(false);
    expect(sameSource({ kind: 'snapshot', id: 'a' }, { kind: 'version', id: 'a' })).toBe(false);
  });

  it('names and labels each source', () => {
    const snapshot = { kind: 'snapshot', id: 'snap-1' } as const;
    const version = { kind: 'version', id: 'asset-1_1' } as const;
    expect(compareName(ORIGINAL_SOURCE, SNAPSHOTS, VERSIONS)).toBe('Original');
    expect(compareName(snapshot, SNAPSHOTS, VERSIONS)).toBe('Warm v1');
    expect(compareLabel(snapshot, SNAPSHOTS, VERSIONS)).toBe('Snapshot · Warm v1');
    expect(compareLabel({ kind: 'snapshot', id: 'gone' }, SNAPSHOTS, VERSIONS)).toBe(
      'Snapshot · Snapshot'
    );
    expect(compareName(version, SNAPSHOTS, VERSIONS)).toBe('Version · B&W');
    expect(compareLabel(version, SNAPSHOTS, VERSIONS)).toBe('Version · B&W');
  });
});

describe('compare panes', () => {
  const stage = { left: 100, top: 20, width: 1002, height: 600 };

  it('sizes the viewport to one pane in side by side', () => {
    expect(compareViewport({ w: 1002, h: 600 }, 'side')).toEqual({ w: 500, h: 600 });
    expect(compareViewport({ w: 1002, h: 600 }, 'split')).toEqual({ w: 1002, h: 600 });
  });

  it('anchors a point to the pane it falls in', () => {
    expect(comparePane(stage, { clientX: 150, clientY: 50 }, 'side')).toEqual({
      left: 100,
      top: 20,
      width: 500,
      height: 600
    });
    expect(comparePane(stage, { clientX: 900, clientY: 50 }, 'side')).toEqual({
      left: 602,
      top: 20,
      width: 500,
      height: 600
    });
    expect(comparePane(stage, { clientX: 900, clientY: 50 }, 'split')).toBe(stage);
  });
});
