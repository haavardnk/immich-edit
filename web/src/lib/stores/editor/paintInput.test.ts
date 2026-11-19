import { afterEach, describe, expect, it } from 'vitest';
import { paintInput } from './paintInput.svelte';

describe('paintInput', () => {
  afterEach(() => paintInput.reset());

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
