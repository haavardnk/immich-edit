<script lang="ts">
  import InfoSection from '#lib/components/browse/info/InfoSection.svelte';
  import InfoPlaceActions from '#lib/components/browse/info/InfoPlaceActions.svelte';
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import MiniMap from '#lib/components/map/MiniMap.svelte';
  import { placeOf } from '#lib/browse/info/photoInfo.ts';
  import { ownsAsset } from '#lib/browse/ownership.ts';
  import type { AssetSummary } from '#lib/types/album.ts';

  let { asset }: { asset: AssetSummary } = $props();

  const place = $derived(placeOf(asset.exifInfo));
  const detail = $derived.by(() => {
    if (!place) return null;
    const parts = [place.region, place.coordinates === place.name ? null : place.coordinates];
    const text = parts.filter((part): part is string => part !== null).join(' · ');
    return text === '' ? null : text;
  });
</script>

{#if place || ownsAsset(asset)}
  <InfoSection title="Place">
    {#if place?.point}
      <div class="mb-2">
        <MiniMap point={place.point} label={place.name} photo={asset.id} />
      </div>
    {/if}
    {#if place}
      <div class="flex min-w-0 flex-col gap-0.5">
        <TruncationTooltip text={place.name}>
          {#snippet child({ props })}
            <span
              {...props}
              class="truncate text-label font-semibold text-light-950/90 touch:text-base"
            >
              {place.name}
            </span>
          {/snippet}
        </TruncationTooltip>
        {#if detail}
          <TruncationTooltip text={detail}>
            {#snippet child({ props })}
              <span {...props} class="truncate text-xs text-light-950/65 touch:text-sm">
                {detail}
              </span>
            {/snippet}
          </TruncationTooltip>
        {/if}
      </div>
    {:else}
      <p class="text-xs text-light-950/65 touch:text-sm">No location</p>
    {/if}
    <InfoPlaceActions {asset} />
  </InfoSection>
{/if}
