import { charCount } from '#lib/utils/charCount.ts';
import { describe, expect, it } from 'vitest';

describe('charCount', () => {
  it('counts characters, not UTF-16 units', () => {
    expect(charCount('🏔️')).toBe(2);
    expect(charCount('Ålesund')).toBe(7);
  });
});
