import {
  clampPanel,
  DEFAULT_MAP_PANEL,
  MAX_MAP_PANEL,
  MIN_MAP_PANEL
} from '#lib/map/mapPanel.svelte.ts';
import { describe, expect, it } from 'vitest';

describe('map panel width', () => {
  it('keeps the width between its bounds', () => {
    expect(clampPanel(100)).toBe(MIN_MAP_PANEL);
    expect(clampPanel(5000)).toBe(MAX_MAP_PANEL);
    expect(clampPanel(412.4)).toBe(412);
    expect(clampPanel(Number.NaN)).toBe(DEFAULT_MAP_PANEL);
  });
});
