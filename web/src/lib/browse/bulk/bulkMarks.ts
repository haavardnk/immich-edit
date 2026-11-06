import { LABEL_TEXT, type LabelColor } from '#lib/stores/labels.ts';
import type { Coverage } from '#lib/utils/selectionState.ts';
import {
  mdiCircle,
  mdiCircleOutline,
  mdiCloseCircle,
  mdiCloseCircleOutline,
  mdiHeart,
  mdiHeartOutline,
  mdiStar,
  mdiStarOutline
} from '@mdi/js';

export type Mark = {
  icon: string;
  label: string;
  pressed: boolean | 'mixed';
  tone: string;
};

export type Look = { icon: string; tone: string };

export const NOT_OWNER_TRASH =
  'Shared by another Immich user. Only the owner can move a photo to the trash.';

export function coverageOf(on: boolean, mixed = false): Coverage {
  return mixed ? 'some' : on ? 'all' : 'none';
}

function toggleMark(
  state: Coverage,
  icons: { on: string; off: string },
  labels: { on: string; off: string },
  tone: { all: string; some: string }
): Mark {
  return {
    icon: state === 'none' ? icons.off : icons.on,
    label: state === 'all' ? labels.on : labels.off,
    pressed: state === 'some' ? 'mixed' : state === 'all',
    tone: state === 'all' ? tone.all : state === 'some' ? tone.some : ''
  };
}

export function favoriteMark(state: Coverage): Mark {
  return toggleMark(
    state,
    { on: mdiHeart, off: mdiHeartOutline },
    { on: 'Unfavorite', off: 'Favorite' },
    { all: 'text-red-400', some: 'text-red-400/50' }
  );
}

export function rejectMark(state: Coverage): Mark {
  return toggleMark(
    state,
    { on: mdiCloseCircle, off: mdiCloseCircleOutline },
    { on: 'Unreject', off: 'Reject' },
    { all: 'text-amber-400', some: 'text-amber-400/50' }
  );
}

export function ratingLook(rating: number | null): Look {
  return { icon: rating ? mdiStar : mdiStarOutline, tone: '' };
}

export function labelLook(label: LabelColor | null): Look {
  return label
    ? { icon: mdiCircle, tone: LABEL_TEXT[label] }
    : { icon: mdiCircleOutline, tone: '' };
}

export function trashReason(trashable: boolean): string | null {
  return trashable ? null : NOT_OWNER_TRASH;
}
