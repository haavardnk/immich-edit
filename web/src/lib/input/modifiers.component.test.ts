import { effectLog } from '#lib/testing/effectLog.svelte.ts';
import { flushSync } from 'svelte';
import { afterEach, describe, expect, it } from 'vitest';
import { clear, held, latch, latched, type ModifierKeys } from './modifiers.svelte.ts';

const NONE: ModifierKeys = { shiftKey: false, altKey: false, ctrlKey: false, metaKey: false };

afterEach(() => clear());

describe('modifier latches', () => {
  it('notify effects when a role is latched, released or cleared', () => {
    const log = effectLog(() => latched('sample'));
    flushSync();
    latch('sample', true);
    flushSync();
    latch('sample', false);
    flushSync();
    latch('sample', true);
    flushSync();
    clear();
    flushSync();
    log.stop();
    expect(log.values).toEqual([false, true, false, true, false]);
  });

  it('notify effects that read held', () => {
    const log = effectLog(() => held('extend', NONE));
    flushSync();
    latch('extend', true);
    flushSync();
    log.stop();
    expect(log.values).toEqual([false, true]);
  });
});
