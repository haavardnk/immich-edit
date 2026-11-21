import {
  mdiDeleteOutline,
  mdiPaletteOutline,
  mdiPencilOutline,
  mdiTagOutline,
  mdiTagPlusOutline
} from '@mdi/js';
import type { MenuActionInfo, MenuGroups } from '#lib/browse/menuActions.ts';

export type TagAction = 'open' | 'rename' | 'color' | 'subtag' | 'delete';

export type TagMenuPlace = 'list' | 'page';

export const TAG_ACTIONS: Readonly<Record<TagAction, MenuActionInfo>> = {
  open: { label: 'Open tag', icon: mdiTagOutline, danger: false },
  rename: { label: 'Rename', icon: mdiPencilOutline, danger: false },
  color: { label: 'Change color', icon: mdiPaletteOutline, danger: false },
  subtag: { label: 'New sub-tag', icon: mdiTagPlusOutline, danger: false },
  delete: { label: 'Delete tag', icon: mdiDeleteOutline, danger: true }
};

const EDIT_GROUPS: MenuGroups<TagAction> = [['rename', 'color', 'subtag'], ['delete']];

export function tagMenu(place: TagMenuPlace): MenuGroups<TagAction> {
  return place === 'list' ? [['open'], ...EDIT_GROUPS] : EDIT_GROUPS;
}
