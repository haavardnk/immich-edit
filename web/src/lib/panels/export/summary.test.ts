import { describe, expect, it } from 'vitest';
import { defaultExportForm, type ExportFormValues } from './settings';
import { fileSummary, nameSummary, outputSummary, sizeSummary, watermarkSummary } from './summary';

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
    expect(sizeSummary(form({ resizeMode: 'percent' }), null)).toBe('Percentage');
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
});
