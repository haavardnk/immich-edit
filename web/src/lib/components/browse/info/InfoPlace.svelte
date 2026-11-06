<script lang="ts">
  import InfoSection from '#lib/components/browse/info/InfoSection.svelte';
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import { placeOf } from '#lib/browse/info/photoInfo.ts';
  import type { AssetSummary } from '#lib/types/album.ts';
  import { Icon } from '@immich/ui';
  import { mdiMapOutline } from '@mdi/js';

  let { asset }: { asset: AssetSummary } = $props();

  const place = $derived(placeOf(asset.exifInfo));
</script>

{#if place}
  <InfoSection title="Place">
    <div class="flex min-w-0 flex-col gap-0.5">
      <TruncationTooltip text={place.name}>
        {#snippet child({ props })}
          <span
            {...props}
            class="truncate text-[13px] font-semibold text-light-950/90 touch:text-base"
          >
            {place.name}
          </span>
        {/snippet}
      </TruncationTooltip>
      {#if place.region}
        <TruncationTooltip text={place.region}>
          {#snippet child({ props })}
            <span {...props} class="truncate text-xs text-light-950/60 touch:text-sm">
              {place.region}
            </span>
          {/snippet}
        </TruncationTooltip>
      {/if}
      {#if place.mapUrl}
        <a
          href={place.mapUrl}
          target="_blank"
          rel="noopener noreferrer"
          class="mt-1 inline-flex w-fit items-center gap-1 rounded text-xs text-primary hover:underline touch:min-h-11 touch:text-sm"
        >
          <Icon icon={mdiMapOutline} size="14px" aria-hidden="true" />
          Open in map
        </a>
      {/if}
    </div>
  </InfoSection>
{/if}
