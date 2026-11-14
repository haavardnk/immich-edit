import type { Version } from '#lib/editor/versions.ts';
import { beforeEach, describe, expect, it } from 'vitest';
import { VersionHold } from '#lib/stores/editor/versionHold.svelte.ts';

const ORIGINAL: Version = { id: 'm', name: 'Original', copyIndex: null, named: false };
const MONO: Version = { id: 'm_2', name: 'B&W', copyIndex: 2, named: true };

describe('VersionHold', () => {
  let current: Version | null;
  let hold: VersionHold;

  beforeEach(() => {
    current = ORIGINAL;
    hold = new VersionHold({ current: () => current, url: (id) => `/render/${id}` });
  });

  it('shows the held version and names the one it returns to', () => {
    expect(hold.start(MONO)).toBe(true);
    expect(hold.url).toBe('/render/m_2');
    expect(hold.heldId).toBe('m_2');
    expect(hold.pill).toEqual({
      lead: 'Holding',
      label: 'B&W',
      tail: '· release to return to Original',
      mono: true
    });
  });

  it('ignores the version that is already open', () => {
    expect(hold.canHold('m')).toBe(false);
    expect(hold.start(ORIGINAL)).toBe(false);
    expect(hold.url).toBeNull();
    expect(hold.pill).toBeNull();
  });

  it('does nothing before the current version is known', () => {
    current = null;
    expect(hold.start(MONO)).toBe(false);
    expect(hold.url).toBeNull();
  });

  it('ends on release of the held version only', () => {
    hold.start(MONO);
    hold.end('m_3');
    expect(hold.url).toBe('/render/m_2');
    hold.end('m_2');
    expect(hold.url).toBeNull();
    expect(hold.pill).toBeNull();
  });

  it('ends whatever is held when no version is given', () => {
    hold.start(MONO);
    hold.end();
    expect(hold.heldId).toBeNull();
  });
});
