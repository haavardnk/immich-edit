import type { CullState } from '#lib/browse/cullState.ts';
import { ADVANCE_MS, ratingHud } from '#lib/browse/ratingHud.svelte.ts';
import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import RatingHud from './RatingHud.svelte';

const STATE: CullState = {
  rating: 3,
  isFavorite: false,
  rejected: false,
  label: null,
  owned: true
};

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function status(): HTMLElement {
  const element = target.querySelector<HTMLElement>('[role="status"]');
  if (!element) throw new Error('no status');
  return element;
}

beforeEach(() => {
  vi.useFakeTimers();
  target = document.createElement('div');
  document.body.appendChild(target);
  component = mount(RatingHud, { target });
  flushSync();
});

afterEach(() => {
  ratingHud.clear();
  if (component) void unmount(component);
  component = null;
  target.remove();
  vi.useRealTimers();
});

describe('RatingHud', () => {
  it('keeps the live region mounted while hidden', () => {
    expect(status().textContent?.trim()).toBe('');
  });

  it('lights the rated stars and names the rating', () => {
    ratingHud.show('a', 'rating', STATE, null);
    flushSync();
    expect(status().textContent?.replace(/\s+/g, ' ').trim()).toBe('3 stars');
    expect(status().querySelectorAll('.text-warning-400')).toHaveLength(3);
    expect(target.querySelector('[data-testid="rating-hud-progress"]')).toBeNull();
  });

  it('runs a progress bar for the advance delay when it will step', () => {
    ratingHud.show('a', 'rating', { ...STATE, rating: 0 }, () => {});
    flushSync();
    expect(status().textContent?.replace(/\s+/g, ' ').trim()).toBe('No rating · next photo');
    const bar = target.querySelector<HTMLElement>('[data-testid="rating-hud-progress"]');
    expect(bar?.style.animationDuration).toBe(`${ADVANCE_MS}ms`);
  });

  it('shows flags without stars', () => {
    ratingHud.show('a', 'reject', { ...STATE, rejected: true }, () => {});
    flushSync();
    expect(status().textContent?.replace(/\s+/g, ' ').trim()).toBe('Rejected · next photo');
    expect(status().querySelectorAll('svg')).toHaveLength(0);
  });
});
