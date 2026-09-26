import { getEdits } from '$lib/api/edits';
import { createPasteEditsJob } from '$lib/api/jobs';
import { runBulkJob } from '$lib/api/bulkJob';
import { manifestToEdits, editsToManifest } from '$lib/edits/manifest';
import { clipboard } from '$lib/stores/clipboard.svelte';
import { copyDialog } from '$lib/stores/copyDialog.svelte';
import { editedThumbs } from '$lib/stores/editedThumbs.svelte';
import { toasts } from '$lib/stores/toasts.svelte';

export async function copyEditsFrom(assetId: string): Promise<void> {
  if (editedThumbs.getHash(assetId) === undefined) {
    toasts.push('info', 'This photo has no edits to copy', 3000);
    return;
  }
  try {
    const record = await getEdits(assetId);
    copyDialog.show(manifestToEdits(record.manifest), () =>
      toasts.push('success', 'Copied settings', 3000)
    );
  } catch (e) {
    toasts.fail('Failed to copy edits', e, 6000);
  }
}

export async function pasteEditsTo(assetIds: string[]): Promise<void> {
  const snap = clipboard.snapshot();
  if (!snap) {
    toasts.push('info', 'Nothing copied', 3000);
    return;
  }
  await runBulkJob(
    (target) => createPasteEditsJob(target, editsToManifest(snap.edits), snap.sections),
    {
      success: (count) => `Queued paste on ${count} asset${count === 1 ? '' : 's'}`,
      error: 'Failed to queue paste'
    },
    assetIds
  );
}
