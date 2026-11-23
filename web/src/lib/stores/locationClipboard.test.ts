import { coordinateText, storedLocation } from '#lib/stores/locationClipboard.svelte.ts';
import { describe, expect, it } from 'vitest';

describe('location clipboard', () => {
  it('restores a stored location and rejects anything malformed', () => {
    expect(storedLocation({ lat: 60.39, lon: 5.32, place: { city: 'Bergen', state: 7 } })).toEqual({
      lat: 60.39,
      lon: 5.32,
      place: { city: 'Bergen', state: null, country: null }
    });
    expect(storedLocation({ lat: 60.39, lon: 5.32 })).toEqual({
      lat: 60.39,
      lon: 5.32,
      place: null
    });
    expect(storedLocation({ lat: '60', lon: 5 })).toBeNull();
    expect(storedLocation({ lat: 91, lon: 5 })).toBeNull();
    expect(storedLocation({ lat: Number.NaN, lon: 5 })).toBeNull();
    expect(storedLocation(null)).toBeNull();
    expect(storedLocation([60, 5])).toBeNull();
  });

  it('writes coordinates as plain text other apps can read', () => {
    expect(coordinateText({ lat: 60.3975, lon: -5.324 })).toBe('60.397500, -5.324000');
  });
});
