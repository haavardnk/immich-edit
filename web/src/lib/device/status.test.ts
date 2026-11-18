import { layoutStatus } from '#lib/device/status.ts';
import { describe, expect, it } from 'vitest';

describe('layoutStatus', () => {
  it('says auto picked the layout for this screen', () => {
    expect(layoutStatus('auto', 'touch')).toBe('Using the touch layout, picked for this screen.');
  });

  it('states a forced layout plainly', () => {
    expect(layoutStatus('desktop', 'desktop')).toBe('Using the desktop layout.');
  });
});
