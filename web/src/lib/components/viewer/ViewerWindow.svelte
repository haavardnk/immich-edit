<script lang="ts">
  import { observeSize } from '#lib/actions/observeSize.ts';
  import ViewerImage from '#lib/components/viewer/ViewerImage.svelte';
  import ViewerSide from '#lib/components/viewer/ViewerSide.svelte';
  import ViewerSplit from '#lib/components/viewer/ViewerSplit.svelte';
  import ViewerStatus from '#lib/components/viewer/ViewerStatus.svelte';
  import { compareViewport } from '#lib/editor/compare.ts';
  import { displayGamutIsWide } from '#lib/utils/colorGamut.ts';
  import { placement } from '#lib/utils/viewGeometry.ts';
  import { viewerFrame } from '#lib/viewer/geometry.ts';
  import { leftTarget, viewerJobs, viewerProof } from '#lib/viewer/jobs.ts';
  import { ViewerRenders } from '#lib/viewer/renders.svelte.ts';
  import { ViewerSync } from '#lib/viewer/sync.svelte.ts';
  import { devicePixelRatio } from 'svelte/reactivity/window';

  const sync = new ViewerSync();
  const renders = new ViewerRenders();
  const wide = displayGamutIsWide();

  let container = $state<HTMLDivElement | null>(null);
  let stage = $state({ w: 0, h: 0 });

  const dpr = $derived(devicePixelRatio.current || 1);
  const render = $derived(sync.render);
  const view = $derived(sync.view);
  const compare = $derived(render?.compare ?? null);
  const pane = $derived(compareViewport(stage, compare?.mode ?? 'off'));
  const srcLong = $derived(view?.srcLong ?? Number.POSITIVE_INFINITY);
  const base = $derived(render && renders.base?.assetId === render.assetId ? renders.base : null);
  const placed = $derived(
    base && view ? viewerFrame(pane, { w: base.w, h: base.h }, view.view, srcLong, dpr) : null
  );
  const left = $derived.by(() => {
    if (!render || !compare) return null;
    const target = leftTarget(render, viewerProof(render, wide));
    return renders.left?.target === target ? renders.left.url : null;
  });
  const tile = $derived.by(() => {
    const landed = renders.tile;
    if (!landed?.roi || !base || !placed || !render || compare || render.dragging) return null;
    if (landed.revision !== base.revision || landed.assetId !== base.assetId) return null;
    const rect = placement(landed.roi, placed.frame, landed.w, landed.h, dpr);
    return rect ? { url: landed.url, rect } : null;
  });
  const edge = $derived(
    view?.greyCanvas ? 'ring-1 ring-white/10' : 'shadow-image ring-1 ring-white/10'
  );
  const title = $derived(render?.name ? `${render.name} · Viewer` : 'Viewer');

  function measure(): void {
    if (!container) return;
    stage = { w: container.clientWidth, h: container.clientHeight };
  }

  $effect(() => sync.connect());

  $effect(() => () => renders.dispose());

  $effect(() => {
    if (!render) return;
    renders.plan(
      viewerJobs({
        render,
        revision: sync.revision,
        pane,
        dpr,
        wide,
        srcLong,
        frame: placed?.frame ?? null,
        base,
        tile: renders.tile
      })
    );
  });

  $effect(() => {
    const root = document.documentElement;
    root.classList.toggle('grey-canvas', view?.greyCanvas ?? false);
    return () => root.classList.remove('grey-canvas');
  });
</script>

<svelte:head>
  <title>{title}</title>
</svelte:head>

<div
  bind:this={container}
  use:observeSize={measure}
  data-testid="viewer-window"
  class="relative flex h-full w-full items-center justify-center overflow-hidden bg-image-canvas"
>
  {#if render && base && placed && view}
    {#if compare?.mode === 'split'}
      <ViewerSplit
        frame={placed.frame}
        {left}
        current={base.url}
        alt={render.name}
        split={view.split}
        labels={compare.labels}
        {edge}
      />
    {:else if compare?.mode === 'side'}
      <ViewerSide
        frame={placed.frame}
        {left}
        current={base.url}
        alt={render.name}
        labels={compare.labels}
        {edge}
      />
    {:else}
      <ViewerImage
        url={base.url}
        rect={placed.frame}
        alt={render.name}
        testid="viewer-image"
        class={edge}
      />
      {#if tile}
        <ViewerImage url={tile.url} rect={tile.rect} alt="" testid="viewer-tile" />
      {/if}
    {/if}
  {:else if !render}
    <p class="text-sm text-muted" role="status">Waiting for the editor</p>
  {/if}
  <ViewerStatus linked={sync.linked} zoom={placed?.zoom ?? null} />
</div>
