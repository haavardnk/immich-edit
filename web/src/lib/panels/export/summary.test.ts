import { describe, expect, it } from 'vitest';
import { defaultExportForm, type ExportFormValues } from './settings';
import {
  fileSummary,
  nameSummary,
  outputSummary,
  presetSummary,
  sizeSummary,
  watermarkSummary
} from './summary';

function form(patch: Partial<ExportFormValues> = {}): ExportFormValues {
  return { ...defaultExportForm(), ...patch };
}

describe('export section summaries', () => {
  it('describe the file by format, quality or depth, space and metadata', () => {
    expect(fileSummary(form())).toBe('JPEG · Quality 90 · sRGB · Metadata All');
    expect(fileSummary(form({ format: 'tiff', bitDepth: '16', colorSpace: 'displayp3' }))).toBe(
      'TIFF · 16-bit · Display P3 · Metadata All'
    );
    expect(fileSummary(form({ format: 'webp', lossless: true, metadata: 'no-location' }))).toBe(
      'WebP · Lossless · sRGB · Metadata All but location'
    );
  });

  it('give the output pixels and sharpening', () => {
    const crop = { w: 6000, h: 4000 };
    expect(sizeSummary(form(), crop)).toBe('6000 × 4000');
    expect(sizeSummary(form({ resizeMode: 'dimensions', sharpenMedia: 'screen' }), crop)).toBe(
      '2048 × 1365 · Sharpen Screen'
    );
    expect(sizeSummary(form({ resizeMode: 'percent', resizePercent: 50 }), null)).toBe('50%');
    expect(sizeSummary(form({ resizeMode: 'megapixels', resizeMegapixels: 12 }), null)).toBe(
      '12 MP'
    );
    expect(sizeSummary(form({ resizeMode: 'dimensions', resizeHeight: null }), null)).toBe(
      '2048 px wide'
    );
    expect(sizeSummary(form(), null)).toBe('Full size');
  });

  it('name the watermark with its size and corner', () => {
    expect(watermarkSummary(form(), 'Logo')).toBe('None');
    expect(watermarkSummary(form({ watermarkId: 'w1', watermarkSize: 12 }), 'Logo')).toBe(
      'Logo · 12% · Bottom right'
    );
    expect(watermarkSummary(form({ watermarkId: 'w1' }), null)).toMatch(/^Watermark · /);
  });

  it('show the template, or the default when blank', () => {
    expect(nameSummary(form({ filenameTemplate: '{date}_{name}' }))).toBe('{date}_{name}');
    expect(nameSummary(form({ filenameTemplate: '  ' }))).toBe('{name}_edit');
  });

  it('sum up the output for the touch footer', () => {
    expect(outputSummary(form(), { w: 6000, h: 4000 })).toEqual({
      size: '6000 × 4000 px',
      spec: 'JPEG · 90 · sRGB'
    });
    expect(outputSummary(form({ format: 'png', bitDepth: '16' }), null)).toEqual({
      size: null,
      spec: 'PNG · 16-bit · sRGB'
    });
  });

  it('describe a preset on one line, with its upload albums', () => {
    expect(presetSummary(form({ resizeMode: 'dimensions', sharpenMedia: 'screen' }), null)).toBe(
      'JPEG 90 · 2048 × 2048 · sRGB · Sharpen Screen'
    );
    expect(
      presetSummary(
        form({ format: 'jxl', bitDepth: '16', colorSpace: 'displayp3', metadata: 'none' }),
        null
      )
    ).toBe('JPEG XL 16-bit · Full size · Display P3 · Metadata None');
    expect(
      presetSummary(form({ format: 'webp', lossless: true, watermarkId: 'w1' }), {
        albums: ['Family 2026']
      })
    ).toBe('WebP Lossless · Full size · sRGB · Watermark · to Immich, Family 2026');
    expect(presetSummary(form(), { albums: [] })).toBe('JPEG 90 · Full size · sRGB · to Immich');
  });
});
