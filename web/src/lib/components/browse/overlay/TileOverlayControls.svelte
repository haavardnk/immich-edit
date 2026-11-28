<script lang="ts">
  import CheckboxRow from '#lib/components/CheckboxRow.svelte';
  import Notice from '#lib/components/Notice.svelte';
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import { OVERLAY_ICON_PATHS } from '#lib/browse/overlayIcons.ts';
  import {
    DETAIL_LABELS,
    detailsFull,
    ICON_LABELS,
    MAX_DETAILS,
    OVERLAY_DETAILS,
    OVERLAY_ICONS,
    withDetail,
    withIcon,
    type OverlayIcon
  } from '#lib/browse/tileOverlay.ts';
  import {
    DETAIL_SAMPLES,
    fitNotice,
    FORMAT_SAMPLE,
    ICON_COLUMNS
  } from '#lib/browse/tileOverlayText.ts';
  import { TILE_INFO_OPTIONS } from '#lib/browse/viewOptions.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { keysFor } from '#lib/shortcuts/labels.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { preferences } from '#lib/stores/preferences.svelte.ts';
  import { Icon } from '@immich/ui';

  const heading = 'section-label flex h-6 items-center justify-between text-dark/65 touch:h-8';
  const note = 'text-micro text-dark/65 touch:text-caption';

  const id = $props.id();
  const overlay = $derived(preferences.overlay);
  const capped = $derived(detailsFull(overlay));
  const cycleKeys = $derived(
    layout.mode === 'desktop' || layout.hasKeyboard ? keysFor('gridTileInfo') : ''
  );
  const notice = $derived(
    fitNotice(browseView.tileInfo, overlay, browseView.rowHeight, browseView.gridSize)
  );
</script>

{#snippet glyph(icon: OverlayIcon)}
  {#if icon === 'format'}
    <span
      aria-hidden="true"
      class="mr-1.5 inline-flex h-3.25 items-center rounded-xs px-0.75 align-[-1px] text-nano leading-none font-bold tracking-wide ring-1 ring-current ring-inset touch:mr-2.5 touch:h-4"
    >
      {FORMAT_SAMPLE}
    </span>
  {:else}
    <span aria-hidden="true" class="mr-1.5 inline-flex align-[-2px] text-dark/65 touch:mr-2.5">
      <Icon icon={OVERLAY_ICON_PATHS[icon]} size="14px" class="touch:size-4.5" />
    </span>
  {/if}
{/snippet}

<div class="flex flex-col gap-3">
  <div class="flex flex-col gap-1.5">
    <SegmentedChoice
      appearance="view"
      fill
      options={TILE_INFO_OPTIONS}
      value={browseView.tileInfo}
      label="Thumbnail info"
      onChange={(info) => browseView.setTileInfo(info)}
    />
    {#if cycleKeys}
      <p class="flex items-center gap-1.5 {note}">
        <kbd class="rounded border border-hairline px-1.5 py-0.5 font-mono text-micro text-dark/65">
          {cycleKeys}
        </kbd>
        cycles the modes
      </p>
    {/if}
  </div>

  <div role="group" aria-labelledby="{id}-icons" class="flex flex-col">
    <div class={heading}>
      <span id="{id}-icons">Icons</span>
      <span class="normal-case">{overlay.icons.length} of {OVERLAY_ICONS.length}</span>
    </div>
    <div class="grid grid-cols-2 gap-x-3">
      {#each ICON_COLUMNS as column, i (i)}
        <div class="flex min-w-0 flex-col">
          {#each column as icon (icon)}
            <CheckboxRow
              checked={overlay.icons.includes(icon)}
              label={ICON_LABELS[icon]}
              onChange={(on) => void preferences.setTileOverlay(withIcon(overlay, icon, on))}
            >
              {#snippet leading()}
                {@render glyph(icon)}
              {/snippet}
            </CheckboxRow>
          {/each}
        </div>
      {/each}
    </div>
  </div>

  <div role="group" aria-labelledby="{id}-details" class="flex flex-col">
    <div class={heading}>
      <span id="{id}-details">Details</span>
      <span class="normal-case" class:text-primary={capped}>
        {overlay.details.length} of {MAX_DETAILS}
      </span>
    </div>
    {#each OVERLAY_DETAILS as detail (detail)}
      {@const on = overlay.details.includes(detail)}
      <CheckboxRow
        checked={on}
        label={DETAIL_LABELS[detail]}
        disabled={capped && !on}
        onChange={(next) => void preferences.setTileOverlay(withDetail(overlay, detail, next))}
      >
        {#snippet trailing()}
          <span aria-hidden="true" class="ml-auto min-w-0 truncate pl-3 text-dark/50">
            {DETAIL_SAMPLES[detail]}
          </span>
        {/snippet}
      </CheckboxRow>
    {/each}
    {#if browseView.tileInfo !== 'full'}
      <p class="mt-1 {note}">Shown on hover, and always in Full.</p>
    {/if}
    {#if capped}
      <p class="mt-1 {note}">Up to {MAX_DETAILS}. Turn one off to choose another.</p>
    {/if}
  </div>

  {#if notice}
    <Notice color="info" message={notice} />
  {/if}
</div>
