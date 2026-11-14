import { describe, expect, it } from 'vitest';
import { MIN_RENDER_EDGE, persistedPreviewUrl, renderEdge } from './preview';

describe('persistedPreviewUrl', () => {
  it.each([
    [undefined, 'clip=false'],
    [false, 'clip=false'],
    [true, 'clip=true']
  ] as const)('encodes clip warning %s', (clipWarn, expected) => {
    const url =
      clipWarn === undefined
        ? persistedPreviewUrl('a', 512)
        : persistedPreviewUrl('a', 512, clipWarn);
    expect(url).toBe(`/api/assets/a/preview?max=512&${expected}`);
  });
});

describe('renderEdge', () => {
  it.each([
    [1, MIN_RENDER_EDGE],
    [48, MIN_RENDER_EDGE],
    [64, 64],
    [96.4, 96],
    [4096, 4096]
  ])('sends %s as %s, never below the server minimum', (edge, expected) => {
    expect(renderEdge(edge)).toBe(expected);
  });
});
