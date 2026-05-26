import { getJson } from './client';

export interface ProfileLensEdits {
  k1: number;
  k2: number;
  k3: number;
  vk1: number;
  vk2: number;
  vk3: number;
  ca_red_scale_x10000: number;
  ca_blue_scale_x10000: number;
}

export interface LensProfileMatch {
  matched: boolean;
  lens: string | null;
  focal_length: number | null;
  aperture: number | null;
  edits: ProfileLensEdits | null;
}

export function getLensProfile(assetId: string): Promise<LensProfileMatch> {
  return getJson(`/api/assets/${assetId}/lens-profile`);
}
