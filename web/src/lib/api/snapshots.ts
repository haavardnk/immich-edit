import { editsToManifest, manifestToEdits } from '#lib/edits/manifest.ts';
import { getJson, sendJson, url } from '#lib/api/client.ts';
import { writeEdits } from '#lib/api/edits.ts';
import type { EditManifest, EditRecord, Edits } from '#lib/types/edits.ts';

export interface EditSnapshot {
  id: string;
  name: string;
  manifest_hash: string;
  edits: Edits;
  created_at: string;
  updated_at: string;
}

interface SnapshotItem extends Omit<EditSnapshot, 'edits'> {
  manifest: EditManifest;
}

export interface SnapshotUpdate {
  name?: string;
  edits?: Edits;
}

function fromItem({ manifest, ...item }: SnapshotItem): EditSnapshot {
  return { ...item, edits: manifestToEdits(manifest) };
}

export async function listSnapshots(assetId: string): Promise<EditSnapshot[]> {
  const items = await getJson<SnapshotItem[]>(url`/api/assets/${assetId}/snapshots`);
  return items.map(fromItem);
}

export async function createSnapshot(
  assetId: string,
  name: string,
  edits: Edits
): Promise<EditSnapshot> {
  const item = await sendJson<SnapshotItem>('POST', url`/api/assets/${assetId}/snapshots`, {
    name,
    manifest: editsToManifest(edits)
  });
  return fromItem(item);
}

export async function updateSnapshot(
  assetId: string,
  id: string,
  update: SnapshotUpdate
): Promise<EditSnapshot> {
  const body: { name?: string; manifest?: EditManifest } = {};
  if (update.name !== undefined) body.name = update.name;
  if (update.edits !== undefined) body.manifest = editsToManifest(update.edits);
  const item = await sendJson<SnapshotItem>(
    'PATCH',
    url`/api/assets/${assetId}/snapshots/${id}`,
    body
  );
  return fromItem(item);
}

export async function deleteSnapshot(assetId: string, id: string): Promise<void> {
  await sendJson<void>('DELETE', url`/api/assets/${assetId}/snapshots/${id}`, undefined);
}

export function restoreSnapshot(
  assetId: string,
  id: string,
  baseHash?: string
): Promise<EditRecord> {
  return writeEdits(
    assetId,
    'POST',
    url`/api/assets/${assetId}/snapshots/${id}/restore`,
    undefined,
    baseHash
  );
}
