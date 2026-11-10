import { PARAMS } from '#lib/editor/params/catalog.ts';
import { DEFAULT_PINNED_SETTINGS } from '#lib/stores/preferences.svelte.ts';
import { describe, expect, it } from 'vitest';
import { isPinnable, pinnedParams, rowCaption } from './quickSettings';

describe('quick settings', () => {
  it('resolves every default pin to a slider', () => {
    expect(pinnedParams(DEFAULT_PINNED_SETTINGS).map((param) => param.label)).toEqual([
      'Exposure',
      'Contrast',
      'Shadows',
      'Vibrance',
      'Clarity',
      'Amount'
    ]);
  });

  it('keeps the pin order and skips ids it does not know', () => {
    expect(pinnedParams(['shadows', 'white_balance', 'gone', 'exposure']).map((p) => p.id)).toEqual(
      ['shadows', 'exposure']
    );
    expect(isPinnable('white_balance')).toBe(false);
    expect(isPinnable('highlights')).toBe(true);
  });

  it('captions a row with its touch home', () => {
    const [highlights] = pinnedParams(['highlights']);
    expect(highlights && rowCaption(highlights)).toBe('Highlights · Light › Tone');
  });

  it('only offers ids the server accepts', () => {
    for (const param of PARAMS) expect(param.id).toMatch(/^[a-z0-9][a-z0-9._-]{0,63}$/);
  });
});
