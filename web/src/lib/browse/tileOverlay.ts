export type TileInfo = 'none' | 'icons' | 'full';

export const TILE_INFO_ORDER: readonly TileInfo[] = ['none', 'icons', 'full'];

export const OVERLAY_ICONS = [
  'edited',
  'favorite',
  'rejected',
  'label',
  'rating',
  'copy',
  'location',
  'tags',
  'description',
  'albums',
  'format'
] as const;

export const OVERLAY_DETAILS = [
  'filename',
  'taken',
  'place',
  'tags',
  'description',
  'albums',
  'exposure',
  'camera',
  'lens',
  'size'
] as const;

export type OverlayIcon = (typeof OVERLAY_ICONS)[number];
export type OverlayDetail = (typeof OVERLAY_DETAILS)[number];

export interface TileOverlay {
  icons: readonly OverlayIcon[];
  details: readonly OverlayDetail[];
}

export const MAX_DETAILS = 4;

export const DEFAULT_TILE_OVERLAY: TileOverlay = {
  icons: ['edited', 'favorite', 'rejected', 'label', 'rating', 'copy'],
  details: ['filename', 'taken']
};

export const ICON_LABELS: Readonly<Record<OverlayIcon, string>> = {
  edited: 'Edited',
  favorite: 'Favorite',
  rejected: 'Rejected',
  label: 'Label',
  rating: 'Rating',
  copy: 'Virtual copy',
  location: 'Location',
  tags: 'Tags',
  description: 'Description',
  albums: 'Albums',
  format: 'File format'
};

export const DETAIL_LABELS: Readonly<Record<OverlayDetail, string>> = {
  filename: 'File name',
  taken: 'Taken',
  place: 'Place',
  tags: 'Tags',
  description: 'Description',
  albums: 'Albums',
  exposure: 'Exposure',
  camera: 'Camera',
  lens: 'Lens',
  size: 'Size'
};

const DETAIL_FIT: readonly { below: number; lines: number }[] = [
  { below: 90, lines: 0 },
  { below: 140, lines: 1 },
  { below: 200, lines: 2 },
  { below: 280, lines: 3 }
];

export function tileInfoFrom(value: unknown): TileInfo | null {
  if (value === 'badges') return 'icons';
  return TILE_INFO_ORDER.find((info) => info === value) ?? null;
}

export function nextTileInfo(info: TileInfo): TileInfo {
  return TILE_INFO_ORDER[(TILE_INFO_ORDER.indexOf(info) + 1) % TILE_INFO_ORDER.length] ?? info;
}

function inCatalog<K extends string>(catalog: readonly K[], value: unknown): K[] {
  if (!Array.isArray(value)) return [];
  return catalog.filter((key) => value.includes(key));
}

export function tileOverlayFrom(value: unknown): TileOverlay | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null;
  const { icons, details } = value as Record<string, unknown>;
  return {
    icons: inCatalog(OVERLAY_ICONS, icons),
    details: inCatalog(OVERLAY_DETAILS, details).slice(0, MAX_DETAILS)
  };
}

function sameKeys(a: readonly string[], b: readonly string[]): boolean {
  return a.length === b.length && a.every((key, i) => key === b[i]);
}

export function isDefaultOverlay(overlay: TileOverlay): boolean {
  return (
    sameKeys(overlay.icons, DEFAULT_TILE_OVERLAY.icons) &&
    sameKeys(overlay.details, DEFAULT_TILE_OVERLAY.details)
  );
}

function toggled<K extends string>(catalog: readonly K[], keys: readonly K[], key: K, on: boolean) {
  return catalog.filter((entry) => (entry === key ? on : keys.includes(entry)));
}

export function withIcon(overlay: TileOverlay, icon: OverlayIcon, on: boolean): TileOverlay {
  return { ...overlay, icons: toggled(OVERLAY_ICONS, overlay.icons, icon, on) };
}

export function detailsFull(overlay: TileOverlay): boolean {
  return overlay.details.length >= MAX_DETAILS;
}

export function withDetail(overlay: TileOverlay, detail: OverlayDetail, on: boolean): TileOverlay {
  if (on && !overlay.details.includes(detail) && detailsFull(overlay)) return overlay;
  return { ...overlay, details: toggled(OVERLAY_DETAILS, overlay.details, detail, on) };
}

export function detailFit(height: number): number {
  return DETAIL_FIT.find((step) => height < step.below)?.lines ?? MAX_DETAILS;
}

export interface MembershipNeed {
  albums: boolean;
  tags: boolean;
}

export function membershipNeed(info: TileInfo, overlay: TileOverlay): MembershipNeed {
  if (info === 'none') return { albums: false, tags: false };
  const wants = (key: OverlayIcon & OverlayDetail) =>
    overlay.icons.includes(key) || overlay.details.includes(key);
  return { albums: wants('albums'), tags: wants('tags') };
}
