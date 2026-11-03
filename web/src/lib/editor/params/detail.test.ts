import { neutralEdits, neutralSharpenAmount } from '#lib/types/edits.ts';
import { describe, expect, it } from 'vitest';
import { sharpenAmount } from './detail';

describe('sharpen amount', () => {
  it.each([false, true])('stores null only at the neutral amount (raw %s)', (isRaw) => {
    const edits = neutralEdits();
    sharpenAmount.set(edits, neutralSharpenAmount(isRaw), isRaw);
    expect(edits.detail.sharpen_amount).toBeNull();
    const other = neutralSharpenAmount(!isRaw);
    sharpenAmount.set(edits, other, isRaw);
    expect(edits.detail.sharpen_amount).toBe(other);
    expect(sharpenAmount.get(edits, !isRaw)).toBe(other);
  });
});
