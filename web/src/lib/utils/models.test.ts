import type { Model } from '#lib/api/models.ts';
import { describe, expect, it } from 'vitest';
import { formatMb, formatSeconds, installPercent, kindLabel } from './models';

function model(size_bytes: number, progress_bytes: number): Model {
  return { size_bytes, progress_bytes } as Model;
}

describe('models', () => {
  it.each([
    ['semantic', 'Scene classes'],
    ['click', 'Click to select'],
    ['denoise', 'AI Denoise'],
    ['unknown-kind', 'unknown-kind']
  ])('labels %s as %s', (kind, expected) => {
    expect(kindLabel(kind)).toBe(expected);
  });

  it.each([
    [0, 0, 0],
    [200, 100, 50],
    [200, 400, 100]
  ])('reports %i/%i bytes as %i%%', (size, progress, expected) => {
    expect(installPercent(model(size, progress))).toBe(expected);
  });

  it.each([
    [999, '999 ms'],
    [1000, '1.0 s'],
    [3200, '3.2 s']
  ])('formats %i ms as %s', (ms, expected) => {
    expect(formatSeconds(ms)).toBe(expected);
  });

  it('formats bytes as whole megabytes', () => {
    expect(formatMb(1_500_000)).toBe('2 MB');
  });
});
