import { flushSync } from 'svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { installPointerKind, kindOf, pointerKind } from './pointerKind.svelte';

function pointer(pointerType: string): PointerEvent {
  return { pointerType } as PointerEvent;
}

function watchWrites(): { writes: () => number; stop: () => void } {
  let runs = 0;
  const stop = $effect.root(() => {
    $effect(() => {
      void pointerKind.kind;
      runs += 1;
    });
  });
  flushSync();
  return { writes: () => runs - 1, stop };
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('kindOf', () => {
  it.each([
    ['mouse', 'mouse'],
    ['touch', 'touch'],
    ['pen', 'touch'],
    ['', 'touch']
  ])('treats %s as %s', (type, kind) => {
    expect(kindOf(type)).toBe(kind);
  });
});

describe('pointerKind', () => {
  it('writes only when the kind changes', () => {
    pointerKind.note(pointer('mouse'));
    const { writes, stop } = watchWrites();

    pointerKind.note(pointer('touch'));
    flushSync();
    expect(pointerKind.touch).toBe(true);
    expect(writes()).toBe(1);

    pointerKind.note(pointer('touch'));
    pointerKind.note(pointer('pen'));
    flushSync();
    expect(writes()).toBe(1);

    pointerKind.note(pointer('mouse'));
    pointerKind.note(pointer('mouse'));
    flushSync();
    expect(pointerKind.kind).toBe('mouse');
    expect(writes()).toBe(2);
    stop();
  });

  it('listens to pointerdown and pointermove in the capture phase without blocking', () => {
    const win = { addEventListener: vi.fn(), removeEventListener: vi.fn() };
    vi.stubGlobal('window', win);

    const cleanup = installPointerKind();
    const options = { capture: true, passive: true };
    expect(win.addEventListener).toHaveBeenCalledWith('pointerdown', pointerKind.note, options);
    expect(win.addEventListener).toHaveBeenCalledWith('pointermove', pointerKind.note, options);

    cleanup();
    expect(win.removeEventListener).toHaveBeenCalledWith('pointerdown', pointerKind.note, options);
    expect(win.removeEventListener).toHaveBeenCalledWith('pointermove', pointerKind.note, options);
  });
});
