<script lang="ts">
  import { stepLoupe } from '#lib/browse/loupeNav.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { browsing } from '#lib/stores/browsing.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiChevronLeft, mdiChevronRight } from '@mdi/js';

  let { currentId, multi }: { currentId: string | null; multi: boolean } = $props();

  const hasPrev = $derived(
    !multi && currentId ? browsing.prevOf(currentId) !== null || browsing.hasPrevious : false
  );
  const atLoadedEnd = $derived(currentId ? browsing.nextOf(currentId) === null : false);
  const hasNext = $derived(!multi && currentId !== null && (!atLoadedEnd || browsing.hasMore));
  const edge =
    'absolute top-1/2 -translate-y-1/2 bg-black/40 text-white hover:bg-black/70 touch:size-11';
</script>

{#if hasPrev}
  <IconButton
    type="button"
    size="medium"
    variant="ghost"
    color="secondary"
    shape="round"
    class="{edge} left-2 touch:left-1"
    icon={mdiChevronLeft}
    title={hint('Previous', 'loupeNav')}
    aria-label="Previous"
    onclick={() => stepLoupe(-1)}
  />
{/if}
{#if hasNext}
  <IconButton
    type="button"
    size="medium"
    variant="ghost"
    color="secondary"
    shape="round"
    class="{edge} right-2 touch:right-1"
    icon={mdiChevronRight}
    title={hint('Next', 'loupeNav')}
    aria-label="Next"
    loading={atLoadedEnd && browsing.loadingMore}
    onclick={() => stepLoupe(1)}
  />
{/if}
