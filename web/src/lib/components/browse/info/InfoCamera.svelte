<script lang="ts">
  import InfoSection from '#lib/components/browse/info/InfoSection.svelte';
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import CaptureTimeQuick from '#lib/components/metadata/CaptureTimeQuick.svelte';
  import { ownsAsset } from '#lib/browse/ownership.ts';
  import { detailRows, exposureCells } from '#lib/browse/info/photoInfo.ts';
  import type { AssetSummary } from '#lib/types/album.ts';

  let { asset }: { asset: AssetSummary } = $props();

  const cells = $derived(exposureCells(asset));
  const owned = $derived(ownsAsset(asset));
  const rows = $derived.by(() => {
    const details = detailRows(asset);
    const dated = details.some((row) => row.key === 'Taken');
    return owned && !dated ? [...details, { key: 'Taken', value: 'Not set' }] : details;
  });
</script>

<InfoSection>
  {#if cells.length > 0}
    <dl
      class="grid grid-cols-4 divide-x divide-hairline rounded-lg border border-hairline bg-light-950/4"
    >
      {#each cells as cell (cell.label)}
        <div class="flex min-w-0 flex-col-reverse items-center gap-0.5 px-1 py-2 touch:py-2.5">
          <dt class="section-label text-light-950/65">
            {cell.label}
          </dt>
          <dd
            class="max-w-full truncate text-label font-semibold text-light-950/90 tabular-nums touch:text-base"
          >
            {cell.value}
          </dd>
        </div>
      {/each}
    </dl>
  {/if}
  {#if rows.length > 0}
    <dl class="flex flex-col gap-1.5 touch:gap-0">
      {#each rows as row (row.key)}
        <div class="flex min-w-0 items-center justify-between gap-4 touch:min-h-9">
          <dt class="shrink-0 text-xs text-light-950/65 touch:text-body">{row.key}</dt>
          {#if row.key === 'Taken' && owned}
            <dd class="min-w-0">
              <CaptureTimeQuick {asset} value={row.value} />
            </dd>
          {:else}
            <TruncationTooltip text={row.value}>
              {#snippet child({ props })}
                <dd
                  {...props}
                  class="min-w-0 truncate text-right text-xs font-medium text-light-950/90 tabular-nums touch:text-body"
                >
                  {row.value}
                </dd>
              {/snippet}
            </TruncationTooltip>
          {/if}
        </div>
      {/each}
    </dl>
  {:else if cells.length === 0}
    <p class="text-xs text-light-950/65 italic touch:text-body">No EXIF data</p>
  {/if}
</InfoSection>
