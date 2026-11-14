import { describe, expect, it } from 'vitest';
import { strokeCount, strokeName, strokeTitle } from './text';

describe('retouch text', () => {
  it('names a stroke by its position and mode', () => {
    expect(strokeName(2)).toBe('Stroke 3');
    expect(strokeTitle('clone', 0)).toBe('Stroke 1 · Clone');
    expect(strokeTitle('heal', 4)).toBe('Stroke 5 · Heal');
  });

  it('counts strokes against the limit', () => {
    expect(strokeCount(3)).toBe('3/64');
  });
});
