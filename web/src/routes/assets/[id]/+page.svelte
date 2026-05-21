<script lang="ts">
  import { page } from '$app/state';
  import { onMount, onDestroy } from 'svelte';
  import Navbar from '$lib/components/layout/Navbar.svelte';
  import Viewer from '$lib/components/editor/Viewer.svelte';
  import EditDrawer from '$lib/components/editor/EditDrawer.svelte';
  import Histogram from '$lib/components/editor/Histogram.svelte';
  import { getEdits, putEdits, deleteEdits } from '$lib/api/edits';
  import { livePreview, persistedPreviewUrl, getPreviewMeta } from '$lib/api/preview';
  import { downloadExport } from '$lib/api/export';
  import { getAsset } from '$lib/api/assets';
  import { NEUTRAL_EDITS, isIdentity } from '$lib/types/edits';
  import type { Edits } from '$lib/types/edits';
  import type { AssetDetail } from '$lib/types/asset';
  import type { PreviewMeta } from '$lib/types/preview';
  import { SingleFlight } from '$lib/utils/single-flight';
  import { makeObjectUrl, revoke } from '$lib/utils/object-url';
  import { downloadBlob } from '$lib/utils/download';

  const assetId = page.params.id as string;
  const MAX_EDGE = 1600;

  let asset = $state<AssetDetail | null>(null);
  let edits = $state<Edits>({ ...NEUTRAL_EDITS });
  let previewUrl = $state<string | null>(null);
  let pending = $state(false);
  let saving = $state(false);
  let exporting = $state(false);
  let meta = $state<PreviewMeta | null>(null);
  let error = $state<string | null>(null);
  let initialised = $state(false);

  type LiveArgs = { edits: Edits };
  type LiveResult = { url: string; metaId: string | null };

  const flight = new SingleFlight<LiveArgs, LiveResult>(
    async (args, signal) => {
      pending = true;
      const { blob, metaId } = await livePreview(assetId, args.edits, MAX_EDGE, signal);
      const url = makeObjectUrl(blob);
      return { url, metaId };
    },
    (_args, result) => {
      const prev = previewUrl;
      previewUrl = result.url;
      revoke(prev);
      pending = false;
      if (result.metaId) {
        void loadMeta(result.metaId);
      }
    },
    (err) => {
      pending = false;
      error = (err as Error).message;
    }
  );

  async function loadMeta(metaId: string): Promise<void> {
    try {
      meta = await getPreviewMeta(assetId, metaId);
    } catch {
      meta = null;
    }
  }

  function loadPersisted(): void {
    const prev = previewUrl;
    const url = persistedPreviewUrl(assetId, MAX_EDGE);
    previewUrl = url + `&_=${Date.now()}`;
    if (prev && prev.startsWith('blob:')) revoke(prev);
  }

  function onLiveChange(): void {
    if (!initialised) return;
    flight.submit({ edits: $state.snapshot(edits) });
  }

  async function persist(): Promise<void> {
    if (!initialised) return;
    saving = true;
    try {
      if (isIdentity(edits)) {
        await deleteEdits(assetId);
      } else {
        const saved = await putEdits(assetId, $state.snapshot(edits));
        edits = { ...edits, ...saved.edits };
      }
    } catch (e) {
      error = (e as Error).message;
    } finally {
      saving = false;
    }
  }

  async function onCommit(): Promise<void> {
    onLiveChange();
    await persist();
  }

  async function onReset(): Promise<void> {
    edits = { ...NEUTRAL_EDITS };
    saving = true;
    try {
      await deleteEdits(assetId);
    } finally {
      saving = false;
    }
    loadPersisted();
  }

  async function onExport(): Promise<void> {
    exporting = true;
    try {
      const blob = await downloadExport(assetId, $state.snapshot(edits));
      const name = (asset?.originalFileName ?? assetId).replace(/\.[^.]+$/, '') + '_edit.jpg';
      downloadBlob(blob, name);
    } catch (e) {
      error = (e as Error).message;
    } finally {
      exporting = false;
    }
  }

  onMount(async () => {
    try {
      const [a, s] = await Promise.all([getAsset(assetId), getEdits(assetId)]);
      asset = a;
      edits = { ...NEUTRAL_EDITS, ...s.edits };
      loadPersisted();
      initialised = true;
    } catch (e) {
      error = (e as Error).message;
    }
  });

  onDestroy(() => {
    flight.cancel();
    if (previewUrl && previewUrl.startsWith('blob:')) revoke(previewUrl);
  });
</script>

<div class="min-h-screen h-screen flex flex-col">
  <Navbar title={asset?.originalFileName ?? assetId} back="/" />
  {#if error}
    <div class="alert alert-error rounded-none text-sm">{error}</div>
  {/if}
  <div class="flex-1 flex overflow-hidden">
    <Viewer
      src={previewUrl}
      pending={pending}
      width={meta?.width ?? 0}
      height={meta?.height ?? 0}
      label={asset?.originalFileName ?? ''}
    />
    <div class="flex flex-col w-72 bg-base-200">
      <EditDrawer
        bind:edits
        onLiveChange={onLiveChange}
        onCommit={onCommit}
        onReset={onReset}
        onExport={onExport}
        saving={saving}
        exporting={exporting}
      />
      <div class="p-3 border-t border-base-300">
        <span class="text-xs opacity-60 block mb-1">Histogram</span>
        <Histogram hist={meta?.histogram ?? null} />
        {#if meta}
          <p class="text-xs opacity-50 mt-1 font-mono">renderer: {meta.renderer}</p>
        {/if}
      </div>
    </div>
  </div>
</div>
