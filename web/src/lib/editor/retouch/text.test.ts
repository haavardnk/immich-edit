import { describe, expect, it } from 'vitest';
import { retouchSummary, strokeCount, strokeName, strokeTitle } from './text';

describe('retouch text', () => {
  it('names a stroke by its position and mode', () => {
    expect(strokeName(2)).toBe('Stroke 3');
    expect(strokeTitle('clone', 0)).toBe('Stroke 1 · Clone');
    expect(strokeTitle('heal', 4)).toBe('Stroke 5 · Heal');
  });

  it('counts strokes against the limit', () => {
    expect(strokeCount(3)).toBe('3/64');
  });

  it('summarises the stroke list and pauses at the limit', () => {
    expect(retouchSummary(0)).toBe('No strokes yet');
    expect(retouchSummary(1)).toBe('1 stroke');
    expect(retouchSummary(12)).toBe('12 strokes');
    expect(retouchSummary(64)).toBe('64 strokes · painting paused');
  });
});
