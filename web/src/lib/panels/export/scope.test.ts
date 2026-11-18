import { NONE_OWNED_STACK, NOT_OWNER_STACK } from '#lib/browse/ownership.ts';
import { session } from '#lib/stores/session.svelte.ts';
import { beforeEach, describe, expect, it } from 'vitest';
import {
  REPLACE_NONE_OWNED,
  REPLACE_NOT_OWNER,
  REPLACE_TRASH_OFF,
  REPLACE_TWO_VERSIONS
} from './replace';
import { exportScope, uploadOptions, type ExportAsset } from './scope';
import { defaultExportForm } from './settings';

const MASTER = '11111111-2222-3333-4444-555555555555';

function asset(id: string, ownerId = 'me'): ExportAsset {
  return {
    id,
    originalFileName: `${id}.ARW`,
    ownerId,
    localDateTime: '2024-05-06T07:08:09.000Z',
    exifInfo: null,
    fileCreatedAt: null
  };
}

beforeEach(() => {
  session.set({ id: 'me', email: 'me@x', name: 'Me', is_admin: false, auth_kind: 'password' });
});

describe('exportScope', () => {
  it('describes one owned photo', () => {
    expect(exportScope({ assets: [asset('a')], count: 1, trashOn: true })).toEqual({
      stackable: true,
      stackReason: NOT_OWNER_STACK,
      replaceReason: null,
      nameExample: { original: 'a.ARW', date: '2024-05-06', position: 1, total: 1 }
    });
  });

  it('allows stacking before the photo has loaded', () => {
    const scope = exportScope({ assets: [], count: 1, trashOn: true });
    expect(scope.stackable).toBe(true);
    expect(scope.nameExample).toBeNull();
  });

  it('blocks a photo shared by someone else', () => {
    const scope = exportScope({ assets: [asset('a', 'other')], count: 1, trashOn: true });
    expect(scope.stackable).toBe(false);
    expect(scope.replaceReason).toBe(REPLACE_NOT_OWNER);
  });

  it('explains a selection with no owned photo', () => {
    const scope = exportScope({
      assets: [asset('a', 'other'), asset('b', 'other')],
      count: 2,
      trashOn: true
    });
    expect(scope.stackReason).toBe(NONE_OWNED_STACK);
    expect(scope.replaceReason).toBe(REPLACE_NONE_OWNED);
  });

  it('counts unloaded selected photos as stackable', () => {
    expect(exportScope({ assets: [asset('a', 'other')], count: 3, trashOn: true }).stackable).toBe(
      true
    );
  });

  it('blocks replacing two versions of one photo', () => {
    const scope = exportScope({
      assets: [asset(MASTER), asset(`${MASTER}_1`)],
      count: 2,
      trashOn: true
    });
    expect(scope.replaceReason).toBe(REPLACE_TWO_VERSIONS);
  });

  it('blocks replacing while the Immich trash is off', () => {
    expect(exportScope({ assets: [asset('a')], count: 1, trashOn: false }).replaceReason).toBe(
      REPLACE_TRASH_OFF
    );
  });

  it('numbers the filename sample against the selection', () => {
    expect(
      exportScope({ assets: [asset('a'), asset('b')], count: 5, trashOn: true }).nameExample?.total
    ).toBe(5);
  });
});

describe('uploadOptions', () => {
  const form = { ...defaultExportForm(), stackWithOriginal: true };

  it('replaces the original when nothing blocks it', () => {
    const scope = exportScope({ assets: [asset('a')], count: 1, trashOn: true });
    expect(uploadOptions(form, scope, true)).toMatchObject({
      replaceOriginal: true,
      stackWithOriginal: false
    });
  });

  it('ignores a blocked replace and keeps stacking', () => {
    const scope = exportScope({ assets: [asset('a')], count: 1, trashOn: false });
    expect(uploadOptions(form, scope, true)).toMatchObject({
      replaceOriginal: false,
      stackWithOriginal: true
    });
  });

  it('does not stack photos the user cannot stack', () => {
    const scope = exportScope({ assets: [asset('a', 'other')], count: 1, trashOn: true });
    expect(uploadOptions(form, scope, false)).toMatchObject({
      replaceOriginal: false,
      stackWithOriginal: false
    });
  });
});
