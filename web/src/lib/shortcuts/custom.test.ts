import { describe, expect, it, vi } from 'vitest';
import {
  bindKey,
  browserOwned,
  conflictOf,
  customCount,
  isCustomisable,
  keysOf,
  resetKey,
  swapKey,
  bindOf
} from './custom';

vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/') } }));

describe('custom shortcuts', () => {
  it('keeps ranges, arrows, Escape and gestures fixed', () => {
    for (const id of ['rate', 'gridMove', 'editorEscape', 'sliderReset', 'brushSize']) {
      expect(isCustomisable(bindOf(id)!), id).toBe(false);
    }
    expect(isCustomisable(bindOf('autoAdjust')!)).toBe(true);
  });

  it('reads custom keys over the defaults, but never for a fixed bind', () => {
    expect(keysOf('autoAdjust', {})).toEqual(['Mod+u']);
    expect(keysOf('autoAdjust', { autoAdjust: ['Mod+Shift+a'] })).toEqual(['Mod+Shift+a']);
    expect(keysOf('rate', { rate: ['q'] })).toEqual(['0', '1', '2', '3', '4', '5']);
  });

  it('replaces every key of an action and drops a binding equal to the default', () => {
    const custom = bindKey({}, 'trash', 'Mod+Backspace');
    expect(custom).toEqual({ trash: ['Mod+Backspace'] });
    expect(bindKey(custom, 'autoAdjust', 'Mod+u')).toEqual(custom);
    expect(customCount(bindKey(custom, 'undo', 'Mod+y'))).toBe(2);
  });

  it('finds a conflict only where the contexts meet', () => {
    expect(conflictOf('clipWarn', 'w', {})).toMatchObject({
      bind: { id: 'pickWhiteBalance' },
      context: 'editor'
    });
    expect(conflictOf('cropGrid', 'k', {})).toMatchObject({ bind: { id: 'addBrushLayer' } });
    expect(conflictOf('maskOverlay', 'h', {})).toBeNull();
    expect(conflictOf('autoAdjust', 'ArrowLeft', {})).toMatchObject({ bind: { id: 'editorNav' } });
  });

  it('swaps the old keys onto the other action', () => {
    const custom = swapKey({}, 'clipWarn', 'w', 'pickWhiteBalance');
    expect(custom).toEqual({ clipWarn: ['w'], pickWhiteBalance: ['j'] });
    expect(conflictOf('clipWarn', 'w', custom)).toBeNull();
    expect(swapKey(custom, 'pickWhiteBalance', 'w', 'clipWarn')).toEqual({});
  });

  it('resets one action', () => {
    expect(resetKey({ undo: ['Mod+y'], redo: ['Mod+Shift+y'] }, 'undo')).toEqual({
      redo: ['Mod+Shift+y']
    });
  });

  it('refuses chords the browser keeps', () => {
    expect(browserOwned('Mod+1')).toBe(true);
    expect(browserOwned('Mod+w')).toBe(true);
    expect(browserOwned('Mod+Shift+w')).toBe(true);
    expect(browserOwned('Mod+Shift+e')).toBe(false);
  });
});
