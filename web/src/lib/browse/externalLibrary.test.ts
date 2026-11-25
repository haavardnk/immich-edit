import { describe, expect, it } from 'vitest';
import { externalLibraryNote, inExternalLibrary } from './externalLibrary';

describe('externalLibraryNote', () => {
  it.each([
    [0, 1, null],
    [
      1,
      1,
      'This photo is from an external library. Immich can only delete its file with write access to the library folder; without it, the next library scan imports it again.'
    ],
    [
      1,
      3,
      '1 photo is from an external library. Immich can only delete its file with write access to the library folder; without it, the next library scan imports it again.'
    ],
    [
      2,
      3,
      '2 photos are from an external library. Immich can only delete their files with write access to the library folder; without it, the next library scan imports them again.'
    ]
  ])('words %i external of %i', (external, total, note) => {
    expect(externalLibraryNote(external, total)).toBe(note);
  });
});

describe('inExternalLibrary', () => {
  it('reads the Immich library id', () => {
    expect(inExternalLibrary({ libraryId: '22222222-2222-4222-8222-222222222222' })).toBe(true);
    expect(inExternalLibrary({})).toBe(false);
  });
});
