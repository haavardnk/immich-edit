import { describe, expect, it } from 'vitest';
import {
  baseOptions,
  changeFormat,
  defaultExportForm,
  formatLabel,
  formInvalid,
  immichOptions,
  restoreExportForm,
  type ExportForm
} from './settings';
import type { ExportFormat } from '$lib/api/export';

function form(patch: Partial<ExportForm> = {}): ExportForm {
  return { ...defaultExportForm(), ...patch };
}

describe('restoreExportForm', () => {
  it('round-trips a stored form', () => {
    const saved = form({
      format: 'avif',
      quality: 70,
      qualities: { jpeg: 95, avif: 70 },
      albumIds: ['al'],
      filenameTemplate: '{date}_{name}',
      resizeMode: 'dimensions',
      resizeWidth: 1350,
      resizeHeight: null,
      resizeEnlarge: true,
      sharpenMedia: 'matte',
      sharpenAmount: 'high',
      sharpenPpi: 240,
      watermarkId: 'wm',
      watermarkSize: 35,
      watermarkOpacity: 50,
      watermarkAnchor: 'top',
      watermarkInset: 0
    });
    expect(restoreExportForm(JSON.parse(JSON.stringify(saved)))).toEqual(saved);
  });

  it.each<[Record<string, unknown>, Partial<ExportForm>]>([
    [
      { resizeMode: 'percent', resizePercent: -3 },
      { resizeMode: 'percent', resizePercent: 50 }
    ],
    [
      { resizeMode: 'megapixels', resizeMegapixels: 'lots' },
      { resizeMode: 'megapixels', resizeMegapixels: 12 }
    ],
    [
      { resizeMode: 'long_edge', resizeWidth: 900, resizeHeight: 0 },
      { resizeMode: 'none', resizeWidth: 900, resizeHeight: 2048 }
    ],
    [
      { resizeMode: 'dimensions', resizeWidth: 12.5, resizeHeight: 'big' },
      { resizeMode: 'dimensions', resizeWidth: 2048, resizeHeight: 2048 }
    ]
  ])('restores resize %o as %o', (stored, expected) => {
    expect(restoreExportForm(stored)).toMatchObject(expected);
  });

  it.each([
    [{ filenameSuffix: '_warm' }, '{name}_warm'],
    [{ filenameSuffix: '  ' }, '{name}_edit'],
    [{ filenameSuffix: '_warm', filenameTemplate: '{seq}' }, '{seq}'],
    [{}, '{name}_edit']
  ])('turns stored naming %o into template %s', (stored, template) => {
    expect(restoreExportForm(stored).filenameTemplate).toBe(template);
  });

  it.each([
    [{ metadata: 'no-location' }, 'no-location'],
    [{ metadata: 'gps' }, 'all'],
    [{ includeExif: false }, 'none'],
    [{ includeExif: true }, 'all'],
    [{ metadata: 'none', includeExif: true }, 'none']
  ])('restores metadata from %o as %s', (stored, metadata) => {
    expect(restoreExportForm(stored).metadata).toBe(metadata);
  });

  it('falls back per field on values it does not know', () => {
    const restored = restoreExportForm({
      format: 'bmp',
      quality: 400,
      qualities: { jpeg: 250, png: 50, bmp: 40, webp: 'high', avif: 12.4 },
      bitDepth: '12',
      includeExif: 'yes',
      albumIds: ['al', 3],
      stackPrimary: 'both',
      watermarkId: 7,
      watermarkSize: 250,
      watermarkOpacity: 'full',
      watermarkAnchor: 'middle',
      watermarkInset: -4
    });
    expect(restored).toEqual(
      form({
        quality: 100,
        qualities: { jpeg: 100, avif: 12 },
        albumIds: ['al'],
        watermarkSize: 100,
        watermarkInset: 0
      })
    );
  });
});

describe('baseOptions', () => {
  it.each<[string, Partial<ExportForm>, boolean]>([
    ['webp keeping metadata', { format: 'webp', metadata: 'all', lossless: false }, true],
    [
      'webp keeping all but location',
      { format: 'webp', metadata: 'no-location', lossless: false },
      true
    ],
    ['webp dropping metadata', { format: 'webp', metadata: 'none', lossless: false }, false],
    ['webp asked for lossless', { format: 'webp', metadata: 'none', lossless: true }, true],
    ['jpeg keeping metadata', { format: 'jpeg', metadata: 'all', lossless: false }, false]
  ])('resolves lossless for %s', (_name, patch, expected) => {
    expect(baseOptions(form(patch)).lossless).toBe(expected);
  });

  it('passes the encoder settings through unchanged', () => {
    const opts = baseOptions(form({ format: 'tiff', quality: 70, bitDepth: '16' }));
    expect(opts).toMatchObject({
      format: 'tiff',
      quality: 70,
      bitDepth: '16',
      pngCompression: 'default',
      tiffCompression: 'lzw',
      colorSpace: 'srgb'
    });
  });

  it.each<[Partial<ExportForm>, unknown]>([
    [{ resizeMode: 'none', resizeWidth: 900 }, null],
    [
      { resizeMode: 'dimensions', resizeWidth: 1600, resizeHeight: null, resizeEnlarge: true },
      { mode: 'dimensions', width: 1600, height: null, enlarge: true }
    ],
    [
      { resizeMode: 'percent', resizePercent: 25 },
      { mode: 'percent', percent: 25 }
    ],
    [
      { resizeMode: 'megapixels', resizeMegapixels: 24, resizeEnlarge: true },
      { mode: 'megapixels', megapixels: 24, enlarge: true }
    ]
  ])('turns %o into resize %o', (patch, resize) => {
    expect(baseOptions(form(patch)).resize).toEqual(resize);
  });

  it.each<[Partial<ExportForm>, unknown]>([
    [{ sharpenMedia: 'none', sharpenAmount: 'high' }, null],
    [
      { sharpenMedia: 'glossy', sharpenAmount: 'low', sharpenPpi: 600 },
      { media: 'glossy', amount: 'low', ppi: 600 }
    ]
  ])('turns %o into sharpen %o', (patch, sharpen) => {
    expect(baseOptions(form(patch)).sharpen).toEqual(sharpen);
  });

  it.each<[Partial<ExportForm>, unknown]>([
    [{ watermarkId: null, watermarkSize: 40 }, null],
    [
      { watermarkId: 'wm', watermarkAnchor: 'left' },
      { id: 'wm', size: 20, opacity: 80, anchor: 'left', inset: 3 }
    ]
  ])('turns %o into watermark %o', (patch, watermark) => {
    expect(baseOptions(form(patch)).watermark).toEqual(watermark);
  });
});

describe('formInvalid', () => {
  it.each<[Partial<ExportForm>, boolean]>([
    [{}, false],
    [{ sharpenMedia: 'matte', sharpenPpi: 50 }, true],
    [{ sharpenMedia: 'glossy', sharpenPpi: 300.5 }, true],
    [{ sharpenMedia: 'screen', sharpenPpi: Number.NaN }, false],
    [{ resizeMode: 'dimensions', resizeWidth: 0 }, true],
    [{ filenameTemplate: '{bogus}' }, true]
  ])('flags %o as %s', (patch, invalid) => {
    expect(formInvalid(form(patch))).toBe(invalid);
  });
});

describe('immichOptions', () => {
  it('adds the destination fields to the encoder settings', () => {
    const opts = immichOptions(
      form({ format: 'webp', metadata: 'all', albumIds: ['al'], tagIds: ['t'], favorite: true })
    );
    expect(opts).toMatchObject({
      format: 'webp',
      lossless: true,
      albumIds: ['al'],
      tagIds: ['t'],
      favorite: true,
      stackWithOriginal: false,
      stackPrimary: 'edited',
      filenameTemplate: '{name}_edit'
    });
  });
});

describe('formatLabel', () => {
  it.each([
    ['jpeg', 'JPEG'],
    ['jxl', 'JPEG XL']
  ] as const)('labels %s as %s', (format, label) => {
    expect(formatLabel(format)).toBe(label);
  });
});

describe('changeFormat', () => {
  it.each<[string, number, ExportFormat[], number]>([
    ['starts avif at its default', 90, ['avif'], 60],
    ['starts heic at its default', 90, ['avif', 'heic'], 65],
    ['gives webp its default after a hand-set jpeg', 95, ['png', 'webp'], 85],
    ['gives avif its default after a hand-set jpeg', 95, ['avif'], 60],
    ['brings a hand-set jpeg back', 95, ['avif', 'jpeg'], 95],
    ['brings a hand-set jpeg back through png', 95, ['png', 'webp', 'jpeg'], 95],
    ['leaves quality alone on png', 95, ['png'], 95]
  ])('%s', (_name, jpegQuality, path, want) => {
    const f = form({ format: 'jpeg', quality: jpegQuality });
    for (const next of path) changeFormat(f, next);
    expect(f.format).toBe(path.at(-1));
    expect(f.quality).toBe(want);
  });

  it('remembers a quality set on the new format', () => {
    const f = form();
    changeFormat(f, 'avif');
    f.quality = 45;
    changeFormat(f, 'jpeg');
    changeFormat(f, 'avif');
    expect(f.quality).toBe(45);
    expect(f.qualities).toEqual({ jpeg: 90, avif: 45 });
  });
});
