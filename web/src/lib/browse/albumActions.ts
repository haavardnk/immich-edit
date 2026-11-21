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

export interface AlbumActionInfo {
  label: string;
  icon: string;
  danger: boolean;
}

export const ALBUM_ACTIONS: Readonly<Record<AlbumAction, AlbumActionInfo>> = {
  open: { label: 'Open album', icon: mdiImageMultipleOutline, danger: false },
  rename: { label: 'Rename', icon: mdiPencilOutline, danger: false },
  describe: { label: 'Edit description', icon: mdiText, danger: false },
  delete: { label: 'Delete album', icon: mdiDeleteOutline, danger: true },
  leave: { label: 'Leave album', icon: mdiLogout, danger: true }
};

const BY_ROLE: Readonly<Record<AlbumRole, readonly (readonly AlbumAction[])[]>> = {
  owner: [['rename', 'describe'], ['delete']],
  editor: [['rename', 'describe'], ['leave']],
  viewer: [['leave']]
};

export function albumMenu(
  role: AlbumRole,
  place: AlbumMenuPlace
): readonly (readonly AlbumAction[])[] {
  const groups = BY_ROLE[role];
  return place === 'list' ? [['open'], ...groups] : groups;
}
