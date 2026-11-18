import { flushSync } from 'svelte';
import { describe, expect, it } from 'vitest';
import { presence, type Presence } from './presence.svelte';

function track(initial: boolean): { state: { open: boolean }; sheet: Presence; stop: () => void } {
  const state = $state({ open: initial });
  let sheet: Presence | null = null;
  const stop = $effect.root(() => {
    sheet = presence(() => state.open);
  });
  flushSync();
  if (!sheet) throw new Error('no presence');
  return { state, sheet, stop };
}

describe('presence', () => {
  it('stays mounted after closing until the exit finishes', () => {
    const { state, sheet, stop } = track(true);
    expect(sheet.mounted).toBe(true);
    state.open = false;
    flushSync();
    expect(sheet.mounted).toBe(true);
    sheet.exited();
    expect(sheet.mounted).toBe(false);
    stop();
  });

  it('ignores an exit that lands after reopening', () => {
    const { state, sheet, stop } = track(false);
    expect(sheet.mounted).toBe(false);
    state.open = true;
    flushSync();
    sheet.exited();
    state.open = false;
    flushSync();
    expect(sheet.mounted).toBe(true);
    stop();
  });
});
