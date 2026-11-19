<script lang="ts">
  import AssetTile from '#lib/components/browse/AssetTile.svelte';
  import type { AssetGridBox } from '#lib/browse/assetGridLayout.ts';
  import { isCopy } from '#lib/browse/assetKey.ts';
  import { browseTarget, setRating } from '#lib/browse/assetMeta.ts';
  import { tileOpen } from '#lib/browse/tileOpen.ts';
  import { keyHeld, latched } from '#lib/input/modifiers.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { selection } from '#lib/stores/selection.svelte.ts';
  import { trash } from '#lib/stores/trash.svelte.ts';
  import type { AssetSummary } from '#lib/types/album.ts';

  let {
    items,
    tiles,
    pad,
    selectingAll,
    dragPreview = null
  }: {
    items: readonly AssetSummary[];
    tiles: readonly { asset: AssetSummary; box: AssetGridBox }[];
    pad: number;
    selectingAll: boolean;
    dragPreview?: ReadonlySet<string> | null;
  } = $props();

  const opensLoupe = tileOpen() === 'loupe';

  let extendKey = $state(false);
  let hoveredId = $state<string | null>(null);

  const extending = $derived(extendKey || latched('extend'));

  const rangePreview = $derived.by(() => {
    if (selectingAll) return null;
    if (dragPreview) return dragPreview;
    if (!extending || !hoveredId) return null;
    return selection.rangeTarget(
      items.map((asset) => asset.id),
      hoveredId
    );
  });

  function trackExtend(e: KeyboardEvent): void {
    extendKey = keyHeld('extend', e);
  }

  function clearRangePreview(): void {
    extendKey = false;
    hoveredId = null;
  }
</script>

<svelte:window onkeydown={trackExtend} onkeyup={trackExtend} onblur={clearRangePreview} />

{#each tiles as item (item.asset.id)}
  <div
    class="absolute"
    style:top="{pad + item.box.top}px"
    style:left="{pad + item.box.left}px"
    style:width="{item.box.width}px"
    style:height="{item.box.height}px"
  >
    <AssetTile
      asset={item.asset}
      info={browseView.tileInfo}
      selected={selectingAll || selection.has(item.asset.id)}
      rangePreview={rangePreview?.has(item.asset.id) && !selection.has(item.asset.id)}
      selectionActive={selection.selecting || selectingAll}
      selectMode={selection.mode}
      onToggle={() => {
        if (selectingAll) return;
        selection.toggle(item.asset.id);
        if (selection.has(item.asset.id)) browseView.setActive(item.asset.id);
      }}
      onPreview={() => (hoveredId = item.asset.id)}
      onPreviewEnd={() => {
        if (hoveredId === item.asset.id) hoveredId = null;
      }}
      onRange={() => {
        if (selectingAll) return;
        selection.range(
          items.map((asset) => asset.id),
          item.asset.id
        );
      }}
      onActivate={() => browseView.setActive(item.asset.id)}
      onLoupe={() => browseView.openLoupe(item.asset.id)}
      onOpen={opensLoupe ? () => browseView.openLoupe(item.asset.id) : undefined}
      onRate={(rating) => void setRating(browseTarget(item.asset.id), rating)}
      onDeleteCopy={isCopy(item.asset.id) ? () => void trash.removeCopy(item.asset.id) : undefined}
    />
  </div>
{/each}
