<script lang="ts">
  import { rowHold } from '#lib/gestures/rowHold.svelte.ts';
  import MaskOverlayMenu from '#lib/panels/MaskOverlayMenu.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { maskThumbs } from '#lib/stores/editor/maskThumbs.svelte.ts';
  import { maskedEditsIsZero } from '#lib/types/edits.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Button, Icon, IconButton } from '@immich/ui';
  import { mdiDotsHorizontal, mdiEye, mdiEyeOff, mdiLayersOutline, mdiPlus } from '@mdi/js';

  let { onadd, onmenu }: { onadd: () => void; onmenu: (layerId: string) => void } = $props();

  const layers = $derived(editor.edits.masks);
  const cap = $derived(editor.maskCapacityFor(null));
  const hold = rowHold({ attribute: 'data-layer-hold', start: (id) => onmenu(id), end: () => {} });
</script>

<div class="flex shrink-0 items-center gap-2 px-3 py-1.5">
  <IconButton
    size="medium"
    variant="ghost"
    color="secondary"
    shape="round"
    class="size-12 shrink-0 rounded-xl border border-hairline bg-ghost"
    icon={mdiPlus}
    aria-label="New mask"
    title=""
    disabled={cap.layersFull || cap.totalFull}
    onclick={onadd}
  />
  <div
    role="list"
    aria-label="Masks"
    class="flex min-w-0 flex-1 gap-2 overflow-x-auto scrollbar-hidden"
    {@attach hold.attachment}
  >
    {#each layers as layer (layer.id)}
      {@const active = layer.id === editor.activeLayerId}
      {@const thumb = maskThumbs.urls[layer.id]}
      <div
        role="listitem"
        data-layer-hold={layer.id}
        data-testid="mask-layer-card"
        class="flex h-12 w-36 shrink-0 items-center rounded-xl border {active
          ? 'border-primary bg-primary/12'
          : 'border-hairline bg-ghost'} {layer.enabled ? '' : 'opacity-60'}"
      >
        <Button
          type="button"
          variant="ghost"
          color="secondary"
          class="h-full min-w-0 flex-1 justify-start gap-2 rounded-xl py-0 pr-0 pl-1 font-normal"
          aria-label={layer.name}
          aria-pressed={active}
          onclick={() => editor.setActiveLayer(layer.id)}
        >
          {#if thumb}
            <img
              src={thumb}
              alt=""
              class="size-9 shrink-0 rounded-md bg-image-canvas object-cover"
              data-testid="mask-thumb"
            />
          {:else}
            <span
              class="flex size-9 shrink-0 items-center justify-center rounded-md text-dark/40 ring-1 ring-hairline"
              data-testid="mask-thumb-pending"
            >
              <Icon icon={mdiLayersOutline} size="18px" aria-hidden="true" />
            </span>
          {/if}
          <span class="flex min-w-0 flex-col text-left">
            <span class="flex items-center gap-1.5 text-[13.5px] font-medium text-dark">
              <span class="truncate">{layer.name}</span>
              {#if !maskedEditsIsZero(layer.edits)}
                <span class="size-1.5 shrink-0 rounded-full bg-primary" aria-hidden="true"></span>
              {/if}
            </span>
            <span class="truncate text-[11px] text-dark/65">
              {countOf(layer.components.length, 'shape')}
            </span>
          </span>
        </Button>
        <IconButton
          size="small"
          variant="ghost"
          color="secondary"
          class="me-0.5 size-8 shrink-0"
          icon={mdiDotsHorizontal}
          aria-label="{layer.name} actions"
          title=""
          onclick={() => onmenu(layer.id)}
        />
      </div>
    {/each}
  </div>
  {#if layers.length > 0}
    <MaskOverlayMenu>
      {#snippet trigger(props)}
        <IconButton
          size="medium"
          variant="ghost"
          color={editor.maskOverlayVisible ? 'primary' : 'secondary'}
          shape="round"
          class="size-11 shrink-0 bg-ghost"
          icon={editor.maskOverlayVisible ? mdiEye : mdiEyeOff}
          aria-label="Mask overlay"
          title=""
          {...props}
        />
      {/snippet}
    </MaskOverlayMenu>
  {/if}
</div>
