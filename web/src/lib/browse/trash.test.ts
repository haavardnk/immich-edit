import { beforeEach, describe, expect, it } from 'vitest';
import { session } from '#lib/stores/session.svelte.ts';
import {
  NOT_OWNER_TRASH,
  TRASH_OFF,
  chunked,
  planTrash,
  trashBlocked,
  trashDialogText,
  trashedMessage
} from './trash';

const A = '11111111-1111-1111-1111-111111111111';
const B = '22222222-2222-2222-2222-222222222222';
const C = '33333333-3333-3333-3333-333333333333';

beforeEach(() => {
  session.set({ id: 'me', email: 'me@x', name: 'Me', is_admin: false, auth_kind: 'password' });
});

describe('planTrash', () => {
  it('splits owned masters, copies and shared photos', () => {
    const plan = planTrash(
      [{ id: A, ownerId: 'me' }, { id: `${B}_1` }, { id: C, ownerId: 'partner' }],
      [{ id: A }, { id: `${A}_1`, copyOf: A }, { id: `${B}_1`, copyOf: B }]
    );
    expect(plan).toEqual({ masters: [A], copies: [`${B}_1`], versions: [`${A}_1`], skipped: 1 });
  });

  it('lets a picked copy travel with its trashed master', () => {
    const plan = planTrash(
      [{ id: A, ownerId: 'me' }, { id: `${A}_2` }],
      [{ id: A }, { id: `${A}_2`, copyOf: A }]
    );
    expect(plan.copies).toEqual([]);
    expect(plan.versions).toEqual([`${A}_2`]);
  });
});

describe('trashBlocked', () => {
  it('refuses shared photos and a disabled trash', () => {
    const shared = planTrash([{ id: C, ownerId: 'partner' }], []);
    expect(trashBlocked(shared, true)).toBe(NOT_OWNER_TRASH);
    const owned = planTrash([{ id: A, ownerId: 'me' }], []);
    expect(trashBlocked(owned, false)).toBe(TRASH_OFF);
    expect(trashBlocked(owned, true)).toBeNull();
  });

  it('lets copies go while the Immich trash is off', () => {
    expect(trashBlocked(planTrash([{ id: `${A}_1` }], []), false)).toBeNull();
  });
});

describe('trashDialogText', () => {
  it('names the trash period and what goes with the photos', () => {
    const plan = { masters: [A, B], copies: [`${C}_1`], versions: [`${A}_1`], skipped: 1 };
    const text = trashDialogText(plan, 30);
    expect(text.title).toBe('Move to Immich trash');
    expect(text.lines).toEqual([
      '2 photos move to the Immich trash. Immich deletes them for good after 30 days; restore them from the Immich trash until then.',
      '1 virtual copy goes with them and comes back on restore.',
      '1 virtual copy is deleted from immich-edit. This cannot be undone.',
      '1 photo shared by another Immich user is skipped.'
    ]);
  });

  it('asks to delete when only copies are picked', () => {
    const text = trashDialogText({ masters: [], copies: [`${A}_1`], versions: [], skipped: 0 }, 30);
    expect(text).toMatchObject({ title: 'Delete virtual copy', confirm: 'Delete' });
  });
});

describe('helpers', () => {
  it('chunks requests', () => {
    expect(chunked([1, 2, 3], 2)).toEqual([[1, 2], [3]]);
  });

  it('summarises what happened', () => {
    expect(trashedMessage(2, 0)).toBe('Moved 2 photos to the Immich trash');
    expect(trashedMessage(1, 2)).toBe(
      'Moved 1 photo to the Immich trash and deleted 2 virtual copies'
    );
    expect(trashedMessage(0, 1)).toBe('Deleted 1 virtual copy');
  });
});
