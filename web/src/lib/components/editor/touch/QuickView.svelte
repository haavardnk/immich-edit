<script lang="ts">
  import { pinnedParams } from '#lib/editor/quickSettings.ts';
  import QuickRows from '#lib/panels/quick/QuickRows.svelte';
  import { preferences } from '#lib/stores/preferences.svelte.ts';
  import DevelopTools from './DevelopTools.svelte';

  const HINT = 'Hold a label to pin or unpin';

  const params = $derived(pinnedParams(preferences.pins));
</script>

<div class="@container flex min-h-0 flex-1 flex-col">
  <div class="flex h-10 shrink-0 items-center gap-3 border-b border-hairline pr-2 pl-4">
    <span class="shrink-0 text-[13.5px] font-medium text-light-950">{params.length} pinned</span>
    <span class="hidden min-w-0 flex-1 truncate text-right text-[12px] text-dark/65 @sm:block">
      {HINT}
    </span>
    <span class="flex-1 @sm:hidden"></span>
    <DevelopTools size="small" />
  </div>
  <div
    class="min-h-0 flex-1 overflow-y-auto overscroll-contain px-3 py-1.5 scrollbar-hidden"
    role="region"
    aria-label="Quick"
  >
    {#if params.length > 0}
      <QuickRows {params} />
      <p class="px-1 pt-3 pb-1 text-center text-[12px] text-dark/65 @sm:hidden">{HINT}</p>
    {:else}
      <p class="px-1 py-6 text-center text-[13px] text-dark/65">
        Nothing pinned yet. Hold a setting's label in any group and choose Pin to Quick.
      </p>
    {/if}
  </div>
</div>
