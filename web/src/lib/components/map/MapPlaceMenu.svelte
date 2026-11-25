<script lang="ts">
  import { coordinateLabel } from '#lib/browse/info/photoInfo.ts';
  import { placeAt } from '#lib/browse/locationOps.ts';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import type { PlaceMenuAt } from '#lib/map/placeEditor.svelte.ts';
  import { locationClipboard } from '#lib/stores/locationClipboard.svelte.ts';
  import { selection } from '#lib/stores/selection.svelte.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { mdiContentCopy, mdiMapMarkerOutline, mdiStarPlusOutline } from '@mdi/js';

  let {
    menu,
    onSave,
    onClose
  }: {
    menu: PlaceMenuAt;
    onSave: (menu: PlaceMenuAt) => void;
    onClose: () => void;
  } = $props();

  let anchor = $state<HTMLElement | null>(null);

  const spot = $derived(coordinateLabel(menu.at.lat, menu.at.lon));
  const chosen = $derived([...selection.selected]);

  function copy(): void {
    onClose();
    locationClipboard.copy({ ...menu.at, place: null });
    toasts.push('success', `Copied ${spot}`);
  }

  function placeHere(): void {
    const ids = chosen;
    onClose();
    void placeAt(ids, menu.at.lat, menu.at.lon);
  }
</script>

<span
  bind:this={anchor}
  class="pointer-events-none absolute size-px"
  style:left="{menu.x}px"
  style:top="{menu.y}px"
  aria-hidden="true"
></span>
{#if anchor}
  <Popover
    open
    customAnchor={anchor}
    title="This spot"
    subtitle={spot}
    contentClass="w-56 py-1"
    onOpenChange={(open) => {
      if (!open) onClose();
    }}
  >
    <div role="group" aria-label="This spot" data-place-menu>
      <MenuRow icon={mdiStarPlusOutline} label="Save place here" onclick={() => onSave(menu)} />
      {#if chosen.length > 0}
        <MenuRow
          icon={mdiMapMarkerOutline}
          label="Place {countOf(chosen.length, 'selected photo')} here"
          onclick={placeHere}
        />
      {/if}
      <MenuRow icon={mdiContentCopy} label="Copy coordinates" onclick={copy} />
    </div>
  </Popover>
{/if}
