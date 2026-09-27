import {
  decodeFlatOps,
  encodeFlatOps,
  neutralEdits,
  type EditManifest,
  type Edits
} from '$lib/types/edits';
import { decodeColor, encodeColor } from './manifest/color';
import { decodeTransform, encodeTransform } from './manifest/geometry';
import { decodeMasks } from './manifest/masks';
import { decodeRetouch } from './manifest/retouch';
import { decodeCurves, encodeCurves } from './manifest/tone';

export function editsToManifest(edits: Edits): EditManifest {
  const ops: Record<string, unknown> = {};
  encodeFlatOps(edits, ops);
  encodeCurves(edits, ops);
  encodeColor(edits, ops);
  encodeTransform(edits, ops);
  if (edits.masks.length > 0) ops.masks = { layers: edits.masks };
  if (edits.retouch.length > 0) ops.retouch = { strokes: edits.retouch };
  return { schema_version: 4, ops };
}

export function manifestToEdits(doc: EditManifest): Edits {
  const edits = neutralEdits();
  const ops = doc.ops ?? {};
  decodeFlatOps(ops, edits);
  if ((doc.schema_version ?? 0) < 4 && edits.lens.profile_enabled === null) {
    edits.lens.profile_enabled = false;
  }
  decodeCurves(ops, edits);
  decodeColor(ops, edits);
  decodeTransform(ops, edits);
  decodeMasks(ops, edits);
  decodeRetouch(ops, edits);
  return edits;
}
