import { describe, expect, it } from 'vitest';
import {
  baseOptions,
  defaultExportForm,
  formatLabel,
  immichOptions,
  restoreExportForm,
  type ExportForm
} from './settings';

function form(patch: Partial<ExportForm> = {}): ExportForm {
  return { ...defaultExportForm(), ...patch };
}

describe('restoreExportForm', () => {
  it('round-trips a stored form', () => {
    const saved = form({
      format: 'avif',
      quality: 70,
      albumIds: ['al'],
      filenameTemplate: '{date}_{name}',
      resizeMode: 'dimensions',
      resizeWidth: 1350,
      resizeHeight: null,
      resizeEnlarge: true
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

  it('falls back per field on values it does not know', () => {
    const restored = restoreExportForm({
      format: 'bmp',
      quality: 400,
      bitDepth: '12',
      includeExif: 'yes',
      albumIds: ['al', 3],
      stackPrimary: 'both'
    });
    expect(restored).toEqual(form({ quality: 100, albumIds: ['al'] }));
  });
});

describe('baseOptions', () => {
  it.each<[string, Partial<ExportForm>, boolean]>([
    ['webp keeping exif', { format: 'webp', includeExif: true, lossless: false }, true],
    ['webp dropping exif', { format: 'webp', includeExif: false, lossless: false }, false],
    ['webp asked for lossless', { format: 'webp', includeExif: false, lossless: true }, true],
    ['jpeg keeping exif', { format: 'jpeg', includeExif: true, lossless: false }, false]
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
});

describe('immichOptions', () => {
  it('adds the destination fields to the encoder settings', () => {
    const opts = immichOptions(
      form({ format: 'webp', includeExif: true, albumIds: ['al'], tagIds: ['t'], favorite: true })
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
