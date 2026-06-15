import { describe, it, expect, beforeEach } from 'vitest';
import { browseControls } from './browseControls.svelte';

describe('browseControls excludeRejected', () => {
  beforeEach(() => browseControls.reset());

  it('is off by default', () => {
    expect(browseControls.excludeRejected).toBe(false);
    expect(browseControls.isDefault).toBe(true);
    expect(browseControls.isFiltered).toBe(false);
  });

  it('counts as a filter without changing server query', () => {
    const keyBefore = browseControls.serverFilterKey;
    const bodyBefore = browseControls.searchBody({});
    browseControls.excludeRejected = true;
    expect(browseControls.isDefault).toBe(false);
    expect(browseControls.isFiltered).toBe(true);
    expect(browseControls.serverFilterKey).toBe(keyBefore);
    expect(browseControls.searchBody({})).toEqual(bodyBefore);
  });

  it('is cleared by reset', () => {
    browseControls.excludeRejected = true;
    browseControls.reset();
    expect(browseControls.excludeRejected).toBe(false);
  });
});
