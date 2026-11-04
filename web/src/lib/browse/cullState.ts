import { ownsAsset } from '#lib/browse/ownership.ts';
import { isRejected } from '#lib/browse/reject.ts';
import { labelOf, type LabelColor } from '#lib/stores/labels.ts';
import type { ExifInfo, TagRef } from '#lib/types/asset.ts';

export interface CullAsset {
  ownerId?: string;
  isFavorite: boolean;
  exifInfo: Pick<ExifInfo, 'rating'> | null;
  tags?: TagRef[] | null;
}

export interface CullState {
  rating: number;
  isFavorite: boolean;
  rejected: boolean;
  label: LabelColor | null;
  owned: boolean;
}

export interface CullActions {
  rate: (value: number | null) => void;
  favorite: () => void;
  reject: () => void;
  label: (color: LabelColor | null) => void;
}

export function cullState(asset: CullAsset): CullState {
  return {
    rating: asset.exifInfo?.rating ?? 0,
    isFavorite: asset.isFavorite,
    rejected: isRejected(asset),
    label: labelOf(asset),
    owned: ownsAsset(asset)
  };
}
