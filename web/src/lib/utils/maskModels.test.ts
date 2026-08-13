import { describe, expect, it } from 'vitest';
import type { MaskModel } from '$lib/api/masks';
import { formatMb, formatSeconds, installPercent, kindLabel } from './maskModels';

function model(size_bytes: number, progress_bytes: number): MaskModel {
  return { size_bytes, progress_bytes } as MaskModel;
}

describe('maskModels', () => {
  it.each([
    ['semantic', 'Scene classes'],
    ['click', 'Click to select'],
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
