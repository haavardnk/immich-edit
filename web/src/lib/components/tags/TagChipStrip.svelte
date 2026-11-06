<script lang="ts">
  import TagChip from '#lib/components/tags/TagChip.svelte';
  import type { TagRef } from '#lib/types/asset.ts';
  import { fitCount } from '#lib/utils/fitCount.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Button } from '@immich/ui';
  import type { Snippet } from 'svelte';

  let {
    tags,
    disabled = false,
    onRemove,
    onMore,
    children
  }: {
    tags: TagRef[];
    disabled?: boolean;
    onRemove: (tagId: string) => void;
    onMore: () => void;
    children: Snippet<[number]>;
  } = $props();

  let available = $state(0);
  let gap = $state(0);
  let widths = $state<number[]>([]);
  let overflow = $state(0);

  const shown = $derived(available > 0 ? fitCount(widths, available, gap, overflow) : tags.length);
  const hidden = $derived(Math.max(0, tags.length - shown));

  function measure(strip: HTMLElement): () => void {
    const update = (): void => {
      const ruler = strip.querySelector<HTMLElement>('[data-chip-ruler]');
      const trail = strip.querySelector<HTMLElement>('[data-chip-trail]');
      if (!ruler || !trail) return;
      const space = Number.parseFloat(getComputedStyle(strip).columnGap) || 0;
      const sizes = Array.from(ruler.children, (child) => child.getBoundingClientRect().width);
      gap = space;
      overflow = Math.ceil(sizes.pop() ?? 0);
      widths = sizes.map((size) => Math.ceil(size));
      available = Math.floor(strip.clientWidth - trail.getBoundingClientRect().width - space);
    };
    const observer = new ResizeObserver(update);
    observer.observe(strip);
    for (const child of strip.querySelectorAll('[data-chip-ruler], [data-chip-trail]')) {
      observer.observe(child);
    }
    update();
    return () => observer.disconnect();
  }
</script>

<div class="flex min-w-0 flex-1 items-center gap-1 overflow-hidden" {@attach measure}>
  <div class="relative flex min-w-0 items-center gap-1 overflow-hidden">
    {#each tags.slice(0, shown) as tag (tag.id)}
      <TagChip {tag} {disabled} {onRemove} />
    {/each}
    {#if hidden > 0}
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        shape="round"
        class="h-6 shrink-0 bg-light-200 px-2 text-[11px] font-medium tabular-nums touch:h-9 touch:px-3 touch:text-sm"
        title={countOf(hidden, 'more tag')}
        aria-label={countOf(hidden, 'more tag')}
        {disabled}
        onclick={onMore}
      >
        +{hidden}
      </Button>
    {/if}
    <div
      data-chip-ruler
      aria-hidden="true"
      inert
      class="pointer-events-none invisible absolute top-0 left-0 flex w-max gap-1"
    >
      {#each tags as tag (tag.id)}
        <TagChip {tag} ruler onRemove={() => {}} />
      {/each}
      <span
        class="inline-flex h-6 items-center px-2 text-[11px] font-medium tabular-nums touch:h-9 touch:px-3 touch:text-sm"
      >
        +{tags.length}
      </span>
    </div>
  </div>
  <div data-chip-trail class="flex shrink-0 items-center">
    {@render children(hidden)}
  </div>
</div>
