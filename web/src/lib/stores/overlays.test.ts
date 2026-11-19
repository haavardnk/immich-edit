import { escapeTaken, holdEscape, holdModalSheet, isAnyModalOpen } from '#lib/stores/overlays.ts';
import { afterEach, describe, expect, it, vi } from 'vitest';

const h = vi.hoisted(() => ({ modalOpen: false }));

vi.mock('@immich/ui', () => ({ isModalOpen: () => h.modalOpen }));

function keydown(key: string): KeyboardEvent {
  const event = Object.assign(new Event('keydown'), { key }) as KeyboardEvent;
  document.dispatchEvent(event);
  return event;
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('overlays', () => {
  it('marks an Escape as taken while a popover holds it', () => {
    vi.stubGlobal('document', new EventTarget());
    const first = holdEscape();
    const second = holdEscape();
    expect(escapeTaken(keydown('Escape'))).toBe(true);
    expect(escapeTaken(keydown('Enter'))).toBe(false);
    first();
    first();
    expect(escapeTaken(keydown('Escape'))).toBe(true);
    second();
    expect(escapeTaken(keydown('Escape'))).toBe(false);
  });

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
