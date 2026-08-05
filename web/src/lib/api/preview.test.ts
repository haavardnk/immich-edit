import { describe, expect, it } from 'vitest';
import { persistedPreviewUrl } from './preview';

describe('persistedPreviewUrl', () => {
  it.each([
    [undefined, 'clip=false'],
    [false, 'clip=false'],
    [true, 'clip=true']
  ] as const)('encodes clip warning %s', (clipWarn, expected) => {
    const url = clipWarn === undefined
      ? persistedPreviewUrl('a', 512)
      : persistedPreviewUrl('a', 512, clipWarn);
    expect(url).toBe(`/api/assets/a/preview?max=512&${expected}`);
  });
});
