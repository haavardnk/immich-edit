import type { ColorEdits } from './color';
import type { DetailEdits } from './detail';
import type { EffectsEdits } from './effects';
import type { GeometryEdits } from './geometry';
import type { LensEdits } from './lens';
import type { MaskLayer } from './masks';
import type { RetouchStroke } from './retouch';
import type { BasicEdits, ToneEdits } from './tone';

export interface Edits {
  basic: BasicEdits;
  tone: ToneEdits;
  color: ColorEdits;
  detail: DetailEdits;
  effects: EffectsEdits;
  lens: LensEdits;
  geometry: GeometryEdits;
  masks: MaskLayer[];
  retouch: RetouchStroke[];
}

export interface EditManifest {
  schema_version: number;
  ops: Record<string, unknown>;
}

export interface EditRecord {
  schema_version: number;
  asset_id: string;
  immich_updated_at: string | null;
  immich_checksum: string | null;
  renderer_version: string;
  manifest: EditManifest;
  updated_at: string;
  hash: string;
}
