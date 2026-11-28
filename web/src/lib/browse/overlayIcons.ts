import type { OverlayIcon } from '#lib/browse/tileOverlay.ts';
import {
  mdiCircle,
  mdiCloseCircle,
  mdiCommentTextOutline,
  mdiContentDuplicate,
  mdiHeart,
  mdiImageAlbum,
  mdiMapMarkerOutline,
  mdiStarOutline,
  mdiTagOutline,
  mdiTuneVariant
} from '@mdi/js';

export const OVERLAY_ICON_PATHS: Readonly<Record<Exclude<OverlayIcon, 'format'>, string>> = {
  edited: mdiTuneVariant,
  favorite: mdiHeart,
  rejected: mdiCloseCircle,
  label: mdiCircle,
  rating: mdiStarOutline,
  copy: mdiContentDuplicate,
  location: mdiMapMarkerOutline,
  tags: mdiTagOutline,
  description: mdiCommentTextOutline,
  albums: mdiImageAlbum
};
