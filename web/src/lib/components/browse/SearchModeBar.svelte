<script lang="ts">
  import type { SearchMode, SearchModeOption } from '#lib/browse/searchMode.ts';
  import { Button } from '@immich/ui';

  let {
    modes,
    mode,
    onSelect
  }: {
    modes: readonly SearchModeOption[];
    mode: SearchMode;
    onSelect: (mode: SearchMode) => void;
  } = $props();
</script>

<div
  class="flex flex-none items-center gap-2 border-b border-dark/10 px-3 py-1.5 touch:border-b-0 touch:py-0"
  data-testid="search-modes"
>
  <span class="shrink-0 text-[11px] text-dark/65 touch:hidden">Search by</span>
  <div
    class="flex min-w-0 flex-wrap gap-0.5 rounded-2xl bg-light-100 p-0.5 text-[11px] touch:-mx-3 touch:flex-nowrap touch:overflow-x-auto touch:scrollbar-hidden touch:gap-1.5 touch:rounded-none touch:bg-transparent touch:px-3 touch:py-1.5"
  >
    {#each modes as option (option.mode)}
      {@const selected = mode === option.mode}
      <Button
        type="button"
        size="tiny"
        variant={selected ? 'filled' : 'ghost'}
        color={selected ? 'primary' : 'secondary'}
        class="relative shrink-0 whitespace-nowrap touch:h-8 touch:rounded-full touch:px-3 touch:text-[13.5px] touch:font-normal touch:before:absolute touch:before:inset-x-0 touch:before:-inset-y-1.5 {selected
          ? 'touch:bg-primary/22 touch:text-light-950'
          : 'touch:bg-ghost touch:text-dark'}"
        aria-pressed={selected}
        onclick={() => onSelect(option.mode)}
      >
        {option.label}
      </Button>
    {/each}
  </div>
</div>
