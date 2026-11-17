<script lang="ts">
  import SearchField from '#lib/components/SearchField.svelte';
  import {
    shownCheck,
    stageNote,
    stageSummary,
    type PickCheck,
    type Stage
  } from '#lib/browse/bulk/pickStage.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiCheck, mdiMinus } from '@mdi/js';
  import type { Snippet } from 'svelte';

  export type PickItem = {
    id: string;
    label: string;
    detail?: string | null;
    thumb?: string | null;
    icon?: string;
  };

  let {
    items,
    total,
    holding,
    stages,
    searchLabel,
    allTitle,
    busy = false,
    lead,
    onToggle,
    onApply
  }: {
    items: readonly PickItem[];
    total: number;
    holding: (id: string) => number;
    stages: ReadonlyMap<string, Stage>;
    searchLabel: string;
    allTitle: string;
    busy?: boolean;
    lead?: Snippet;
    onToggle: (id: string) => void;
    onApply: () => void;
  } = $props();

  let query = $state('');

  const shown = $derived.by(() => {
    const needle = query.trim().toLowerCase();
    return needle ? items.filter((item) => item.label.toLowerCase().includes(needle)) : items;
  });
  const held = $derived(shown.filter((item) => holding(item.id) > 0));
  const rest = $derived(shown.filter((item) => holding(item.id) <= 0));
  const summary = $derived(stageSummary(stages));
  const heading = 'section-label px-3 pt-3 pb-1 text-dark/65 touch:px-4';

  function ariaChecked(check: PickCheck): 'true' | 'false' | 'mixed' {
    return check === 'mixed' ? 'mixed' : check ? 'true' : 'false';
  }

  function applyOnEnter(event: KeyboardEvent): void {
    if (event.key !== 'Enter' || summary.count === 0 || busy) return;
    event.preventDefault();
    onApply();
  }
</script>

{#snippet row(item: PickItem)}
  {@const count = holding(item.id)}
  {@const stage = stages.get(item.id)}
  {@const check = shownCheck(count, total, stage)}
  {@const note = stageNote(count, total, stage) ?? item.detail ?? null}
  <button
    type="button"
    role="checkbox"
    aria-checked={ariaChecked(check)}
    class="flex h-10 w-full items-center gap-2.5 border-b border-hairline px-3 text-left outline-none hover:bg-subtle focus-visible:bg-subtle touch:h-14 touch:gap-3 touch:px-4"
    onclick={() => onToggle(item.id)}
  >
    {#if item.thumb}
      <img src={item.thumb} alt="" class="size-7 shrink-0 rounded-md object-cover touch:size-10" />
    {:else if item.icon}
      <Icon icon={item.icon} size="20px" class="shrink-0 text-dark/65" aria-hidden="true" />
    {/if}
    <span class="flex min-w-0 flex-1 flex-col leading-tight">
      <span class="truncate text-label text-light-950 touch:text-body">{item.label}</span>
      {#if note}
        <span
          class="truncate text-caption touch:text-label {stage ? 'text-primary' : 'text-dark/65'}"
          >{note}</span
        >
      {/if}
    </span>
    <span
      class="grid size-4 shrink-0 place-items-center rounded touch:size-5.5 touch:rounded-md {check ===
      true
        ? 'bg-primary text-light'
        : check === 'mixed'
          ? 'bg-primary/20 text-primary ring-2 ring-primary ring-inset'
          : 'ring-2 ring-dark/30 ring-inset'}"
      aria-hidden="true"
    >
      {#if check === true}
        <Icon icon={mdiCheck} size="14px" />
      {:else if check === 'mixed'}
        <Icon icon={mdiMinus} size="14px" />
      {/if}
    </span>
  </button>
{/snippet}

<div class="flex flex-1 flex-col" data-bulk-pick>
  <div class="sticky top-0 z-10 bg-light-100 touch:bg-light-50 px-3 pt-2 pb-1 touch:px-0">
    <SearchField bind:value={query} label={searchLabel} compact={false} onkeydown={applyOnEnter} />
  </div>
  {@render lead?.()}
  {#if held.length > 0}
    <div class={heading}>In this selection</div>
    {#each held as item (item.id)}
      {@render row(item)}
    {/each}
  {/if}
  {#if rest.length > 0}
    <div class={heading}>{allTitle}</div>
    {#each rest as item (item.id)}
      {@render row(item)}
    {/each}
  {/if}
  <div
    class="sticky bottom-0 z-10 mt-auto flex items-center gap-3 border-t border-hairline bg-light-100 touch:bg-light-50 px-3 py-2 touch:px-0 touch:py-3"
  >
    <span class="flex min-w-0 flex-1 flex-col leading-tight">
      <span class="text-label text-light-950 touch:text-sm">
        {summary.count === 0
          ? 'No changes'
          : summary.count === 1
            ? '1 change'
            : `${summary.count} changes`}
      </span>
      {#if summary.detail}
        <span class="truncate text-caption text-dark/65 touch:text-label">{summary.detail}</span>
      {/if}
    </span>
    <Button
      size="small"
      color="primary"
      class="h-8 px-4 touch:h-12 touch:flex-1"
      loading={busy}
      disabled={summary.count === 0 || busy}
      onclick={onApply}
    >
      Apply
    </Button>
  </div>
</div>
