import { describe, expect, it } from 'vitest';
import { ALBUM_ACTIONS, albumMenu } from './albumActions';

describe('albumMenu', () => {
  it('limits each role to what Immich allows', () => {
    expect(albumMenu('owner', 'page')).toEqual([['rename', 'describe'], ['delete']]);
    expect(albumMenu('editor', 'page')).toEqual([['rename', 'describe'], ['leave']]);
    expect(albumMenu('viewer', 'page')).toEqual([['leave']]);
  });

  it('offers Open album only outside the album page', () => {
    expect(albumMenu('viewer', 'list')).toEqual([['open'], ['leave']]);
    expect(albumMenu('owner', 'list')[0]).toEqual(['open']);
  });

  it('marks only delete and leave as dangerous', () => {
    const danger = Object.entries(ALBUM_ACTIONS)
      .filter(([, info]) => info.danger)
      .map(([action]) => action);
    expect(danger).toEqual(['delete', 'leave']);
  });
});
