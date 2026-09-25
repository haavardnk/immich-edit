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
  filenameTemplate: '{name}_edit',
  resize: null,
  sharpen: null
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

  it('encodes a resize only when one is set', () => {
    expect(exportUrlPersisted('a', base)).not.toContain('resize_mode');
    const boxed = exportUrlPersisted('a', {
      ...base,
      resize: { mode: 'dimensions', width: 1600, height: null, enlarge: true }
    });
    expect(boxed).toContain('resize_mode=dimensions&resize_width=1600&resize_enlarge=true');
    expect(boxed).not.toContain('resize_height');
    const scaled = exportUrlPersisted('a', { ...base, resize: { mode: 'percent', percent: 25 } });
    expect(scaled).toContain('resize_mode=percent&resize_percent=25');
    const sized = exportUrlPersisted('a', {
      ...base,
      resize: { mode: 'megapixels', megapixels: 12, enlarge: false }
    });
    expect(sized).toContain('resize_mode=megapixels&resize_megapixels=12&resize_enlarge=false');
  });

  it.each([
    [null, ''],
    [
      { media: 'screen', amount: 'low', ppi: 300 },
      'output_sharpen_media=screen&output_sharpen_amount=low'
    ],
    [
      { media: 'matte', amount: 'standard', ppi: 240 },
      'output_sharpen_media=matte&output_sharpen_amount=standard&output_sharpen_ppi=240'
    ]
  ] as const)('encodes output sharpening %o', (sharpen, expected) => {
    const url = exportUrlPersisted('a', { ...base, sharpen });
    if (!sharpen) expect(url).not.toContain('output_sharpen');
    else expect(url.endsWith(expected)).toBe(true);
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
