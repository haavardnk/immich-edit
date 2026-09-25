import { describe, expect, it } from 'vitest';
import { dispositionFilename, exportUrlPersisted, type ExportOptions } from './export';

const base: ExportOptions = {
  format: 'jpeg',
  quality: 90,
  includeExif: true,
  bitDepth: '8',
  pngCompression: 'default',
  tiffCompression: 'lzw',
  lossless: false,
  colorSpace: 'srgb',
  filenameTemplate: '{name}_edit'
};

describe('exportUrlPersisted', () => {
  it.each([
    ['srgb', 'color_space=srgb'],
    ['displayp3', 'color_space=displayp3']
  ] as const)('encodes %s color space', (colorSpace, expected) => {
    const url = exportUrlPersisted('a', { ...base, colorSpace });
    expect(url).toContain(expected);
  });

  it('encodes the filename template', () => {
    expect(exportUrlPersisted('a', base)).toContain('filename_template=%7Bname%7D_edit');
  });
});

describe('dispositionFilename', () => {
  it.each([
    [
      `attachment; filename="Fjord ___.jpg"; filename*=UTF-8''Fjord%20%C3%A6%C3%B8%C3%A5.jpg`,
      'Fjord æøå.jpg'
    ],
    ['attachment; filename="IMG_0001_edit.jpg"', 'IMG_0001_edit.jpg'],
    [`attachment; filename*=UTF-8''%E0%A4%A`, null],
    ['attachment', null],
    [null, null]
  ])('reads %s', (header, name) => {
    expect(dispositionFilename(header)).toBe(name);
  });
});
