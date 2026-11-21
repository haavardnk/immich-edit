import { describe, expect, it } from 'vitest';
import { removalText } from './albumRemoval';

describe('removalText', () => {
  it('names the album and the count when every photo left', () => {
    expect(removalText({ removed: ['a', 'b', 'c', 'd'], refused: 0, failed: 0 }, 'Norway')).toEqual(
      {
        kind: 'success',
        message: 'Removed 4 photos from Norway'
      }
    );
  });

  it('counts the photos Immich kept for their owner', () => {
    expect(removalText({ removed: ['a', 'b', 'c'], refused: 1, failed: 0 }, 'Norway')).toEqual({
      kind: 'warn',
      message: 'Removed 3 photos from Norway',
      description: '1 belongs to someone else and stays'
    });
  });

  it('lists refused and failed photos together', () => {
    expect(removalText({ removed: ['a'], refused: 2, failed: 1 }, 'Norway')).toEqual({
      kind: 'warn',
      message: 'Removed 1 photo from Norway',
      description: '2 belong to someone else and stay; 1 could not be removed'
    });
  });

  it('explains when nothing was removed', () => {
    expect(removalText({ removed: [], refused: 2, failed: 0 }, 'Norway')).toEqual({
      kind: 'info',
      message: 'Nothing removed: 2 belong to someone else and stay'
    });
    expect(removalText({ removed: [], refused: 0, failed: 0 }, 'Norway')).toEqual({
      kind: 'info',
      message: 'Nothing removed'
    });
  });
});
