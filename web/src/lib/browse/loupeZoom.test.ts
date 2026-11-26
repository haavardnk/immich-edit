import { describe, expect, it, vi } from 'vitest';

const device = vi.hoisted(() => ({ mode: 'desktop' as 'desktop' | 'touch' }));

vi.mock('#lib/device/layout.svelte.ts', () => ({
  layout: {
    get mode() {
      return device.mode;
    }
  }
}));

import { flightDuration } from './loupeZoom';

describe('flightDuration', () => {
  it('flies the photo faster on desktop than on touch', () => {
    device.mode = 'desktop';
    expect(flightDuration()).toBe('--transition-duration-base');
    device.mode = 'touch';
    expect(flightDuration()).toBe('--transition-duration-slow');
  });
});
