import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { GRID_SIZE_HUD_MS, gridSizeHud } from './gridSizeHud.svelte';

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  gridSizeHud.hide();
  vi.useRealTimers();
});

describe('grid size HUD', () => {
  it('shows at a point and hides after the delay', () => {
    gridSizeHud.show({ x: 10, y: 20 });
    expect(gridSizeHud.visible).toBe(true);
    expect(gridSizeHud.at).toEqual({ x: 10, y: 20 });

    vi.advanceTimersByTime(GRID_SIZE_HUD_MS);
    expect(gridSizeHud.visible).toBe(false);
  });

  it('restarts the delay on every change and centres without a point', () => {
    gridSizeHud.show({ x: 10, y: 20 });
    vi.advanceTimersByTime(GRID_SIZE_HUD_MS - 100);
    gridSizeHud.show();
    vi.advanceTimersByTime(GRID_SIZE_HUD_MS - 100);

    expect(gridSizeHud.visible).toBe(true);
    expect(gridSizeHud.at).toBeNull();
    vi.advanceTimersByTime(100);
    expect(gridSizeHud.visible).toBe(false);
  });
});
