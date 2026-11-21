import type { MenuActionInfo, MenuGroups } from '#lib/browse/menuActions.ts';
import type { AlbumRole } from '#lib/types/album.ts';
import {
  mdiDeleteOutline,
  mdiImageMultipleOutline,
  mdiLogout,
  mdiPencilOutline,
  mdiText
} from '@mdi/js';

export type AlbumAction = 'open' | 'rename' | 'describe' | 'delete' | 'leave';

export type AlbumMenuPlace = 'list' | 'page';

export const ALBUM_ACTIONS: Readonly<Record<AlbumAction, MenuActionInfo>> = {
  open: { label: 'Open album', icon: mdiImageMultipleOutline, danger: false },
  rename: { label: 'Rename', icon: mdiPencilOutline, danger: false },
  describe: { label: 'Edit description', icon: mdiText, danger: false },
  delete: { label: 'Delete album', icon: mdiDeleteOutline, danger: true },
  leave: { label: 'Leave album', icon: mdiLogout, danger: true }
};

const BY_ROLE: Readonly<Record<AlbumRole, MenuGroups<AlbumAction>>> = {
  owner: [['rename', 'describe'], ['delete']],
  editor: [['rename', 'describe'], ['leave']],
  viewer: [['leave']]
};

export function albumMenu(role: AlbumRole, place: AlbumMenuPlace): MenuGroups<AlbumAction> {
  const groups = BY_ROLE[role];
  return place === 'list' ? [['open'], ...groups] : groups;
}
