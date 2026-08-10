import { describe, expect, it } from 'vitest';
import { previewColorSpace } from './color-gamut';

describe('previewColorSpace', () => {
  it.each([
    ['srgb', false, false, 'srgb'],
    ['srgb', false, true, 'displayp3'],
    ['srgb', true, true, 'srgb'],
    ['displayp3', false, false, 'displayp3']
  ] as const)('proof=%s warn=%s wide=%s -> %s', (proof, warn, wide, expected) => {
    expect(previewColorSpace(proof, warn, wide)).toBe(expected);
  });
});
