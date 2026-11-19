import { afterEach, describe, expect, it } from 'vitest';
import { paintInput } from './paintInput.svelte';
import { ui } from '#lib/stores/ui.svelte.ts';

describe('paintInput', () => {
  afterEach(() => {
    paintInput.reset();
    ui.editorTab = 'develop';
  });

  it('lets fingers paint in Retouch and names its tool in the pill', () => {
    expect(paintInput.fingersPaint).toBe(false);
    ui.editorTab = 'retouch';
    expect(paintInput.fingersPaint).toBe(true);
    paintInput.paints('pen');
    paintInput.begin('pen');
    expect(paintInput.pill?.lead).toBe('Pencil · Heal');
    expect(paintInput.fingersPaint).toBe(false);
  });

  it('lets fingers paint until a pen is seen', () => {
    expect(paintInput.paints('touch')).toBe(true);
    expect(paintInput.paints('pen')).toBe(true);
    expect(paintInput.paints('touch')).toBe(false);
    expect(paintInput.paints('mouse')).toBe(true);
  });

  it('forgets the pen when the tool resets', () => {
    paintInput.paints('pen');
    paintInput.reset();
    expect(paintInput.paints('touch')).toBe(true);
  });

  it('only tracks touch and pen strokes', () => {
    paintInput.begin('mouse');
    expect(paintInput.painting).toBeNull();
    paintInput.begin('pen');
    expect(paintInput.painting).toBe('pen');
    paintInput.end();
    expect(paintInput.painting).toBeNull();
  });
});
