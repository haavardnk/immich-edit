import { DropConfirm, dropQuestion, placeLine } from '#lib/map/dropConfirm.svelte.ts';
import { describe, expect, it, vi } from 'vitest';

vi.mock('#lib/api/map.ts', () => ({
  reverseGeocode: vi.fn(() => Promise.resolve({ city: 'Bergen', state: null, country: 'Norway' }))
}));

const DROP = { ids: ['a', 'b'], lat: 60, lon: 5, verb: 'move' as const, pin: false };

describe('drop confirmation', () => {
  it('words the question for one photo or many', () => {
    expect(dropQuestion({ ids: ['a'], verb: 'place' })).toBe('Place this photo here?');
    expect(dropQuestion(DROP)).toBe('Move 2 photos here?');
    expect(placeLine({ city: 'Bergen', state: null, country: 'Norway' })).toBe('Bergen, Norway');
    expect(placeLine(null)).toBeNull();
  });

  it('resolves with the named place on confirm and false on cancel', async () => {
    const confirm = new DropConfirm();
    const first = confirm.ask(DROP);
    await vi.waitFor(() => expect(confirm.pending?.named).toBe(true));
    confirm.confirm();
    await expect(first).resolves.toEqual({ city: 'Bergen', state: null, country: 'Norway' });
    const second = confirm.ask(DROP);
    confirm.cancel();
    await expect(second).resolves.toBe(false);
    expect(confirm.pending).toBeNull();
  });

  it('cancels an earlier drop when a new one starts', async () => {
    const confirm = new DropConfirm();
    const first = confirm.ask(DROP);
    void confirm.ask(DROP);
    await expect(first).resolves.toBe(false);
  });
});
