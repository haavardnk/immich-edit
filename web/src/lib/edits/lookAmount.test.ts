import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { neutralEdits, type Edits } from '$lib/types/edits';
import { withLookAmount } from './lookAmount';

interface AmountCase {
  name: string;
  amount: number;
  preset: Record<string, unknown>;
  expect: Record<string, unknown>;
}

const CASES_PATH = fileURLToPath(
  new URL('../../../../crates/raw-pipeline/src/edits/amount_cases.json', import.meta.url)
);
const CASES: AmountCase[] = JSON.parse(readFileSync(CASES_PATH, 'utf8'));

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function merged(base: unknown, patch: unknown): unknown {
  if (!isRecord(base) || !isRecord(patch)) return patch;
  const out: Record<string, unknown> = { ...base };
  for (const [key, value] of Object.entries(patch)) out[key] = merged(base[key], value);
  return out;
}

function at(value: unknown, pointer: string): unknown {
  let node = value;
  for (const key of pointer.split('/').slice(1)) {
    if (Array.isArray(node)) node = node[Number(key)];
    else if (isRecord(node)) node = node[key];
    else return undefined;
  }
  return node;
}

describe('withLookAmount', () => {
  it.each(CASES)('$name matches the shared case', (c) => {
    const preset = merged(neutralEdits(), c.preset) as Edits;
    const scaled = withLookAmount(preset, c.amount);
    for (const [pointer, want] of Object.entries(c.expect)) {
      expect(at(scaled, pointer), pointer).toEqual(want);
    }
  });

  it('keeps geometry, masks and lens outside the scaled look', () => {
    const preset = neutralEdits();
    preset.geometry.rotate = 90;
    preset.lens.profile_enabled = true;
    preset.basic.contrast = 40;
    const scaled = withLookAmount(preset, 50);
    expect(scaled.geometry).toBe(preset.geometry);
    expect(scaled.masks).toBe(preset.masks);
    expect(scaled.lens).toBe(preset.lens);
    expect(scaled.basic.contrast).toBe(20);
  });
});
