import type { MenuActionInfo, MenuGroups } from '#lib/browse/menuActions.ts';
import {
  mdiArrowAll,
  mdiContentCopy,
  mdiDeleteOutline,
  mdiImageMultipleOutline,
  mdiPencilOutline
} from '@mdi/js';

export type PlaceAction = 'rename' | 'move' | 'copy' | 'photos' | 'remove';

export const PLACE_ACTIONS: Readonly<Record<PlaceAction, MenuActionInfo>> = {
  rename: { label: 'Rename', icon: mdiPencilOutline, danger: false },
  move: { label: 'Move on the map', icon: mdiArrowAll, danger: false },
  copy: { label: 'Copy location', icon: mdiContentCopy, danger: false },
  photos: { label: 'Show its photos', icon: mdiImageMultipleOutline, danger: false },
  remove: { label: 'Remove', icon: mdiDeleteOutline, danger: true }
};

export const PLACE_MENU: MenuGroups<PlaceAction> = [
  ['rename', 'move', 'copy', 'photos'],
  ['remove']
];

export type PickerPlaceAction = 'pin' | 'rename' | 'remove';

export function pickerPlaceActions(
  name: string
): Readonly<Record<PickerPlaceAction, MenuActionInfo>> {
  return {
    pin: { label: `Move ${name} to the pin`, icon: mdiArrowAll, danger: false },
    rename: PLACE_ACTIONS.rename,
    remove: PLACE_ACTIONS.remove
  };
}

export function pickerPlaceMenu(canMove: boolean): MenuGroups<PickerPlaceAction> {
  return canMove ? [['pin', 'rename'], ['remove']] : [['rename'], ['remove']];
}
