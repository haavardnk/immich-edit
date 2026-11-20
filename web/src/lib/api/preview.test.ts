import { describe, expect, it } from 'vitest';
import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';
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

  it('changes with every saved edit so a mounted image refetches', () => {
    editedThumbs.upsert('edited', 'h1', '2026-11-20T00:00:00Z');
    const first = persistedPreviewUrl('edited', 512);
    editedThumbs.upsert('edited', 'h2', '2026-11-20T00:00:01Z');
    expect(first).toBe(`/api/assets/edited/preview?max=512&clip=false&v=h1&r=`);
    expect(persistedPreviewUrl('edited', 512)).toBe(
      `/api/assets/edited/preview?max=512&clip=false&v=h2&r=`
    );
    editedThumbs.remove('edited');
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
