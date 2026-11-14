import { describe, expect, it } from 'vitest';
import { fileBytes, fileMeasure, fileSpec } from './exportedFile';
import { baseOptions, defaultExportForm } from './settings';

describe('exported file details', () => {
  it('give the size in decimal units and the pixels', () => {
    expect(fileBytes(512)).toBe('512 B');
    expect(fileBytes(820_400)).toBe('820 KB');
    expect(fileBytes(4_212_000)).toBe('4.2 MB');
    const blob = new Blob([new Uint8Array(4_212_000)]);
    expect(fileMeasure({ blob, size: { w: 2048, h: 1365 } })).toBe('4.2 MB · 2048 × 1365');
    expect(fileMeasure({ blob, size: null })).toBe('4.2 MB');
  });

  it('give the format, colour space and render time', () => {
    const options = { ...baseOptions(defaultExportForm()), colorSpace: 'displayp3' as const };
    const renderedAt = new Date(2026, 10, 14, 14, 32);
    const time = renderedAt.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
    expect(fileSpec({ options, renderedAt })).toBe(`JPEG · Display P3 · rendered ${time}`);
  });
});
