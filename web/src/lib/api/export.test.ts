import { describe, expect, it } from 'vitest';
import { exportUrlPersisted, type ExportOptions } from './export';

const base: ExportOptions = {
  format: 'jpeg',
  quality: 90,
  includeExif: true,
  bitDepth: '8',
  pngCompression: 'default',
  tiffCompression: 'lzw',
  lossless: false,
  colorSpace: 'srgb'
};

describe('exportUrlPersisted', () => {
  it.each([
    ['srgb', 'color_space=srgb'],
    ['displayp3', 'color_space=displayp3']
  ] as const)('encodes %s color space', (colorSpace, expected) => {
    const url = exportUrlPersisted('a', { ...base, colorSpace });
    expect(url).toContain(expected);
  });
});
