import type { CullState } from '#lib/browse/cullState.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ADVANCE_MS, ratingHud } from './ratingHud.svelte';

const STATE: CullState = {
  rating: 3,
  isFavorite: false,
  rejected: false,
  label: null,
  owned: true
};

describe('rating HUD', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    ratingHud.clear();
  });

  afterEach(() => {
    ratingHud.clear();
    vi.useRealTimers();
  });

  it('lingers without a step and then hides', () => {
    ratingHud.show('a', 'rating', STATE, null);
    expect(ratingHud.shown).toMatchObject({ id: 'a', advancing: false });
    vi.advanceTimersByTime(ADVANCE_MS);
    expect(ratingHud.shown).not.toBeNull();
    vi.advanceTimersByTime(1000);
    expect(ratingHud.shown).toBeNull();
  });

  it('steps once when the countdown completes', () => {
    const step = vi.fn();
    ratingHud.show('a', 'rating', STATE, step);
    expect(ratingHud.shown).toMatchObject({ advancing: true });
    vi.advanceTimersByTime(ADVANCE_MS - 1);
    expect(step).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(step).toHaveBeenCalledOnce();
    expect(ratingHud.shown).toBeNull();
    vi.advanceTimersByTime(ADVANCE_MS * 4);
    expect(step).toHaveBeenCalledOnce();
  });

  it('restarts the countdown when the same photo is rated again', () => {
    const first = vi.fn();
    const second = vi.fn();
    ratingHud.show('a', 'rating', STATE, first);
    const serial = ratingHud.shown?.serial;
    vi.advanceTimersByTime(ADVANCE_MS - 100);
    ratingHud.show('a', 'rating', { ...STATE, rating: 4 }, second);
    expect(ratingHud.shown?.serial).not.toBe(serial);
    vi.advanceTimersByTime(ADVANCE_MS - 1);
    expect(first).not.toHaveBeenCalled();
    expect(second).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(first).not.toHaveBeenCalled();
    expect(second).toHaveBeenCalledOnce();
  });

  it('cancels the step when the loupe moves to another photo', () => {
    const step = vi.fn();
    ratingHud.show('a', 'rating', STATE, step);
    ratingHud.follow('a');
    expect(ratingHud.shown).not.toBeNull();
    ratingHud.follow('b');
    expect(ratingHud.shown).toBeNull();
    vi.advanceTimersByTime(ADVANCE_MS * 2);
    expect(step).not.toHaveBeenCalled();
  });
});
