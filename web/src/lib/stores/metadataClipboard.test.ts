import { storedMetadata } from '#lib/stores/metadataClipboard.svelte.ts';
import { describe, expect, it } from 'vitest';

describe('copied metadata', () => {
  it('reads a stored copy and drops anything malformed', () => {
    expect(
      storedMetadata({
        name: 'DSCF4120.RAF',
        description: 'Fløyen',
        location: { lat: 60.39, lon: 5.32, place: { city: 'Bergen' } },
        tags: [{ id: 't1', name: 'Norway', value: 'Norway' }, { id: 2 }],
        rating: 3
      })
    ).toEqual({
      name: 'DSCF4120.RAF',
      description: 'Fløyen',
      location: { lat: 60.39, lon: 5.32, place: { city: 'Bergen', state: null, country: null } },
      tags: [{ id: 't1', name: 'Norway', value: 'Norway' }],
      rating: 3
    });
    expect(storedMetadata({ description: 'no name' })).toBeNull();
    expect(storedMetadata({ name: 'x', rating: -1 })?.rating).toBeNull();
    expect(storedMetadata({ name: 'x', tags: [{ id: 't', value: 'v' }] })?.tags).toEqual([]);
    expect(storedMetadata({ name: 'x', location: { lat: 200, lon: 0 } })?.location).toBeNull();
  });
});
