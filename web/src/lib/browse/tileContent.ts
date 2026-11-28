import { copyIndex, isCopy } from '#lib/browse/assetKey.ts';
import { coordinates, fileType, placeOf } from '#lib/browse/info/photoInfo.ts';
import { isManagedTag } from '#lib/browse/managedTags.ts';
import { isRejected } from '#lib/browse/reject.ts';
import type { OverlayDetail, TileOverlay } from '#lib/browse/tileOverlay.ts';
import { labelOf } from '#lib/stores/labels.ts';
import type { AlbumSummary, AssetSummary } from '#lib/types/album.ts';
import type { TagRef } from '#lib/types/asset.ts';
import {
  captureTime,
  fmtAperture,
  fmtCamera,
  fmtDay,
  fmtDim,
  fmtFocal,
  fmtShutter,
  fmtSize
} from '#lib/utils/exif.ts';

export interface TileFacts {
  edited: boolean;
  albums: readonly AlbumSummary[] | null;
  tags: readonly TagRef[] | null;
}

export type TopMark = 'edited' | 'favorite' | 'rejected' | 'label';

export type ClusterItem =
  | { key: 'location' }
  | { key: 'tags'; count: number }
  | { key: 'description' }
  | { key: 'albums'; count: number }
  | { key: 'format'; text: string }
  | { key: 'copy'; text: string }
  | { key: 'rating'; rating: number };

export interface TileDetail {
  key: OverlayDetail;
  text: string;
}

function userTags(facts: TileFacts): readonly TagRef[] {
  return (facts.tags ?? []).filter((tag) => !isManagedTag(tag));
}

function description(asset: AssetSummary): string | null {
  const text = asset.exifInfo?.description?.replace(/\s+/g, ' ').trim();
  return text ? text : null;
}

function joined(parts: readonly (string | null | undefined)[], separator = ' · '): string | null {
  const kept = parts.filter((part): part is string => typeof part === 'string' && part !== '');
  return kept.length > 0 ? kept.join(separator) : null;
}

function copyText(asset: AssetSummary): string | null {
  if (!isCopy(asset.id)) return null;
  return asset.copyLabel ?? `Copy ${copyIndex(asset.id)}`;
}

function marked(asset: AssetSummary, facts: TileFacts, mark: TopMark): boolean {
  switch (mark) {
    case 'edited':
      return facts.edited;
    case 'favorite':
      return asset.isFavorite;
    case 'rejected':
      return isRejected(asset);
    case 'label':
      return labelOf(asset) !== null;
  }
}

const TOP_MARKS: readonly TopMark[] = ['edited', 'favorite', 'rejected', 'label'];

export function topMarks(overlay: TileOverlay, asset: AssetSummary, facts: TileFacts): TopMark[] {
  return TOP_MARKS.filter((mark) => overlay.icons.includes(mark) && marked(asset, facts, mark));
}

function clusterItem(
  key: ClusterItem['key'],
  asset: AssetSummary,
  facts: TileFacts
): ClusterItem | null {
  switch (key) {
    case 'location':
      return asset.exifInfo && coordinates(asset.exifInfo) ? { key } : null;
    case 'tags': {
      const count = userTags(facts).length;
      return count > 0 ? { key, count } : null;
    }
    case 'description':
      return description(asset) ? { key } : null;
    case 'albums': {
      const count = facts.albums?.length ?? 0;
      return count > 0 ? { key, count } : null;
    }
    case 'format': {
      const text = fileType(asset.originalFileName);
      return text ? { key, text } : null;
    }
    case 'copy': {
      const text = copyText(asset);
      return text ? { key, text } : null;
    }
    case 'rating': {
      const rating = asset.exifInfo?.rating ?? 0;
      return rating > 0 ? { key, rating } : null;
    }
  }
}

const CLUSTER_ORDER: readonly ClusterItem['key'][] = [
  'location',
  'tags',
  'description',
  'albums',
  'format',
  'copy',
  'rating'
];

export function clusterItems(
  overlay: TileOverlay,
  asset: AssetSummary,
  facts: TileFacts
): ClusterItem[] {
  return CLUSTER_ORDER.flatMap((key) => {
    if (!overlay.icons.includes(key)) return [];
    const item = clusterItem(key, asset, facts);
    return item ? [item] : [];
  });
}

function detailText(detail: OverlayDetail, asset: AssetSummary, facts: TileFacts): string | null {
  const exif = asset.exifInfo;
  switch (detail) {
    case 'filename':
      return asset.originalFileName || null;
    case 'taken':
      return fmtDay(captureTime(asset));
    case 'place': {
      const place = placeOf(exif);
      return place ? joined([place.name, place.region], ', ') : null;
    }
    case 'tags':
      return joined(
        userTags(facts).map((tag) => tag.name),
        ', '
      );
    case 'description':
      return description(asset);
    case 'albums':
      return joined(
        (facts.albums ?? []).map((album) => album.albumName),
        ', '
      );
    case 'exposure':
      return exif
        ? joined([
            fmtFocal(exif.focalLength),
            fmtAperture(exif.fNumber),
            fmtShutter(exif.exposureTime),
            exif.iso != null ? `ISO ${exif.iso}` : null
          ])
        : null;
    case 'camera':
      return exif ? fmtCamera(exif.make, exif.model) : null;
    case 'lens':
      return exif?.lensModel || null;
    case 'size':
      return exif
        ? joined([fmtDim(exif.exifImageWidth, exif.exifImageHeight), fmtSize(exif.fileSizeInByte)])
        : null;
  }
}

export function tileDetails(
  overlay: TileOverlay,
  asset: AssetSummary,
  facts: TileFacts,
  lines: number
): TileDetail[] {
  const details: TileDetail[] = [];
  for (const key of overlay.details) {
    if (details.length >= lines) break;
    const text = detailText(key, asset, facts);
    if (text) details.push({ key, text });
  }
  return details;
}
