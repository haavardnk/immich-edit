import type { Edits } from '$lib/types/edits';
import { isIdentity } from '$lib/types/edits';

export function exportUrlPersisted(assetId: string): string {
  return `/api/assets/${assetId}/export`;
}

export async function downloadExport(assetId: string, edits: Edits): Promise<Blob> {
  const url = `/api/assets/${assetId}/export`;
  let resp: Response;
  if (isIdentity(edits)) {
    resp = await fetch(url);
  } else {
    resp = await fetch(url, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ edits })
    });
  }
  if (!resp.ok) throw new Error(`export failed: ${resp.status}`);
  return resp.blob();
}
