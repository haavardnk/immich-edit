import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { HSL_BAND_COLORS, HSL_BAND_HUES } from './color';

const PIPELINE = fileURLToPath(
  new URL('../../../../../crates/raw-pipeline/src/ops/hsl.rs', import.meta.url)
);

function pipelineCentres(): number[] {
  const source = readFileSync(PIPELINE, 'utf8');
  const list = /HSL_BAND_CENTERS_DEG[^=]*=\s*\[([^\]]+)\]/.exec(source)?.[1] ?? '';
  return list.split(',').map((value) => Number.parseFloat(value));
}

describe('HSL bands', () => {
  it('centre each band on the hue the pipeline uses', () => {
    expect(HSL_BAND_HUES).toEqual(pipelineCentres());
  });

  it('paint Purple and Magenta in their own hues', () => {
    expect(HSL_BAND_COLORS[6]).toBe('hsl(270, 70%, 65%)');
    expect(HSL_BAND_COLORS[7]).toBe('hsl(300, 70%, 65%)');
  });
});
