import { describe, expect, it, vi } from 'vitest';

vi.mock('$app/navigation', () => ({ goto: vi.fn(() => Promise.resolve()) }));
vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/assets/a') } }));

import { stepBrush } from './editor';

const SIZE = { step: 0.01, min: 0.005, max: 0.5 };

describe('stepBrush', () => {
  it.each([
    ['[', 0.2, 0.19],
    ['{', 0.2, 0.19],
    [']', 0.2, 0.21],
    ['}', 0.2, 0.21]
  ])('%s steps %f to %f', (key, current, expected) => {
    expect(stepBrush(current, key, SIZE)).toBeCloseTo(expected, 6);
  });

  it.each([
    ['[', 0.005, 0.005],
    [']', 0.5, 0.5]
  ])('%s clamps at the bound', (key, current, expected) => {
    expect(stepBrush(current, key, SIZE)).toBe(expected);
  });
});
