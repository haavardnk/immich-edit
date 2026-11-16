import { layoutHint } from '#lib/device/status.ts';
import { describe, expect, it } from 'vitest';

describe('layoutHint', () => {
  it('names the layout Auto picked', () => {
    expect(layoutHint('auto', 'touch')).toBe('Touch on this screen');
  });

  it('says nothing when the choice already names the layout', () => {
    expect(layoutHint('desktop', 'desktop')).toBeNull();
  });
});
