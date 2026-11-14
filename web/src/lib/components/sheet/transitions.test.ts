import { transitionMs } from '#lib/components/sheet/transitions.ts';
import { describe, expect, it } from 'vitest';

describe('transitionMs', () => {
  it('is zero without a transition', () => {
    expect(transitionMs('', '')).toBe(0);
    expect(transitionMs('0s', '0s')).toBe(0);
  });

  it('reads seconds and milliseconds', () => {
    expect(transitionMs('0.3s', '0s')).toBeCloseTo(300);
    expect(transitionMs('250ms', '0s')).toBeCloseTo(250);
  });

  it('takes the longest property including its delay', () => {
    expect(transitionMs('0.3s, 0.1s', '0s, 0.5s')).toBeCloseTo(600);
  });

  it('repeats a single delay for every duration', () => {
    expect(transitionMs('0.2s, 0.4s', '0.1s')).toBeCloseTo(500);
  });
});
