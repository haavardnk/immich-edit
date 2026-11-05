<script lang="ts">
  import ChoiceChips from '#lib/components/ChoiceChips.svelte';
  import type { SearchMode, SearchModeOption } from '#lib/browse/searchMode.ts';

  let {
    modes,
    mode,
    onSelect
  }: {
    modes: readonly SearchModeOption[];
    mode: SearchMode;
    onSelect: (mode: SearchMode) => void;
  } = $props();

  const options = $derived(modes.map((option) => ({ value: option.mode, label: option.label })));
</script>

<div
  class="flex flex-none items-center gap-2 border-b border-dark/10 px-3 py-1.5 touch:border-b-0 touch:py-0"
  data-testid="search-modes"
>
  <span class="shrink-0 text-[11px] text-dark/65 touch:hidden">Search by</span>
  <ChoiceChips {options} value={mode} label="Search by" {onSelect} />
</div>
