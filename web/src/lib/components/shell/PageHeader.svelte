<script lang="ts">
  import TouchAppBar from '#lib/components/shell/touch/TouchAppBar.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import type { Snippet } from 'svelte';

  let {
    title,
    count,
    actions,
    compactActions,
    bar
  }: {
    title: string;
    count?: string;
    actions?: Snippet;
    compactActions?: Snippet;
    bar?: Snippet;
  } = $props();
</script>

{#if layout.compact}
  <TouchAppBar {title} {count} actions={compactActions} {bar} />
{:else}
  <header
    class="flex h-12 shrink-0 items-center gap-3 border-b border-hairline bg-light px-3 sm:px-5"
  >
    <div class="flex min-w-0 flex-1 items-baseline gap-2">
      <h1 class="truncate text-sm font-semibold text-light-950">{title}</h1>
      {#if count}
        <span class="shrink-0 text-light-950/20">·</span>
        <p class="shrink-0 text-[10px] text-dark/65">{count}</p>
      {/if}
    </div>
    {#if actions}
      <div class="flex shrink-0 items-center gap-1.5">
        {@render actions()}
      </div>
    {/if}
  </header>
{/if}
