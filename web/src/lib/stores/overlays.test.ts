import { holdModalSheet, isAnyModalOpen } from '#lib/stores/overlays.ts';
import { describe, expect, it, vi } from 'vitest';

const h = vi.hoisted(() => ({ modalOpen: false }));

vi.mock('@immich/ui', () => ({ isModalOpen: () => h.modalOpen }));

describe('overlays', () => {
  it('reports a package modal', () => {
    h.modalOpen = true;
    expect(isAnyModalOpen()).toBe(true);
    h.modalOpen = false;
    expect(isAnyModalOpen()).toBe(false);
  });

  it('counts held sheets until each one is released', () => {
    const first = holdModalSheet();
    const second = holdModalSheet();
    expect(isAnyModalOpen()).toBe(true);
    first();
    expect(isAnyModalOpen()).toBe(true);
    second();
    expect(isAnyModalOpen()).toBe(false);
  });

  it('ignores a repeated release', () => {
    const first = holdModalSheet();
    const second = holdModalSheet();
    first();
    first();
    expect(isAnyModalOpen()).toBe(true);
    second();
    expect(isAnyModalOpen()).toBe(false);
  });
});
