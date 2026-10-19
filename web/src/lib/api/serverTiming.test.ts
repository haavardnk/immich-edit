import { describe, expect, it } from 'vitest';
import { parseServerTiming } from './serverTiming';

describe('parseServerTiming', () => {
  it('reads render and encode durations', () => {
    expect(parseServerTiming('render;dur=12.5, encode;dur=3.0', 4)).toEqual({
      renderMs: 12.5,
      encodeMs: 3,
      transferMs: 4
    });
  });

  it('ignores unrelated metrics and descriptions', () => {
    expect(
      parseServerTiming('cache;desc="hit", encode;dur=2;desc="jpeg", render;dur=9', 1)
    ).toEqual({ renderMs: 9, encodeMs: 2, transferMs: 1 });
  });

  it('returns null without both durations', () => {
    expect(parseServerTiming(null, 1)).toBeNull();
    expect(parseServerTiming('render;dur=9', 1)).toBeNull();
    expect(parseServerTiming('render;dur=abc, encode;dur=2', 1)).toBeNull();
  });
});
