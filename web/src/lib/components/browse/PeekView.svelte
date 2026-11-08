<script lang="ts">
  import { immichPreviewUrl } from '#lib/api/assets.ts';
  import { persistedPreviewUrl } from '#lib/api/preview.ts';
  import { cullState } from '#lib/browse/cullState.ts';
  import { takenLabel } from '#lib/browse/info/photoInfo.ts';
  import { peek, peekMarks } from '#lib/browse/peek.svelte.ts';
  import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';
  import type { AssetSummary } from '#lib/types/album.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Icon } from '@immich/ui';
  import { mdiStar, mdiStarOutline } from '@mdi/js';

  const RENDER_EDGE = 2048;
  const STARS = [1, 2, 3, 4, 5];

  let { asset }: { asset: AssetSummary } = $props();

  let renderedId = $state<string | null>(null);

  const cull = $derived(cullState(asset));
  const edited = $derived(editedThumbs.has(asset.id));
  const taken = $derived(takenLabel(asset));
  const marks = $derived(peekMarks(cull, edited));
</script>

<svelte:window onblur={() => peek.hide()} />

<figure
  aria-label="Peek"
  class="fixed inset-0 z-50 m-0 flex flex-col items-center bg-black/90 pt-(--safe-top) pb-(--safe-bottom) text-white"
>
  <div class="relative my-4 min-h-0 w-full flex-1">
    {#key asset.id}
      <img
        src={immichPreviewUrl(asset)}
        alt={asset.originalFileName}
        draggable="false"
        class="absolute inset-0 h-full w-full object-contain select-none"
      />
      {#if edited}
        <img
          src={persistedPreviewUrl(asset.id, RENDER_EDGE)}
          alt=""
          draggable="false"
          data-testid="peek-render"
          class="absolute inset-0 h-full w-full object-contain select-none transition-opacity {renderedId ===
          asset.id
            ? 'opacity-100'
            : 'opacity-0'}"
          onload={() => (renderedId = asset.id)}
        />
      {/if}
    {/key}
  </div>
  <figcaption class="mb-4 flex h-12 max-w-full items-center gap-4 px-4 text-[13.5px] text-white/85">
    <span class="truncate font-semibold text-white">{asset.originalFileName}</span>
    {#if taken}
      <span class="shrink-0">{taken}</span>
    {/if}
    <span
      role="img"
      aria-label={cull.rating > 0 ? countOf(cull.rating, 'star') : 'No rating'}
      class="flex shrink-0 gap-px"
    >
      {#each STARS as n (n)}
        {@const lit = n <= cull.rating}
        <Icon
          icon={lit ? mdiStar : mdiStarOutline}
          size="17px"
          class={lit ? 'text-warning-400' : 'text-white/40'}
          aria-hidden="true"
        />
      {/each}
    </span>
    {#each marks as mark (mark)}
      <span class="shrink-0">{mark}</span>
    {/each}
  </figcaption>
</figure>
