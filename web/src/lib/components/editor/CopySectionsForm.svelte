<script lang="ts">
  import CopyQuickPicks from '#lib/components/editor/CopyQuickPicks.svelte';
  import CopyToolRow from '#lib/components/editor/CopyToolRow.svelte';
  import {
    copyHints,
    rowDot,
    rowNote,
    splitColumns,
    visibleRows,
    type CopyHints
  } from '#lib/editor/copyRows.ts';
  import {
    DEVELOP_ROWS,
    PHOTO_ROWS,
    toolsState,
    type CopyToolRow as Row
  } from '#lib/editor/copyTools.ts';
  import { copyDialog } from '#lib/stores/copyDialog.svelte.ts';

  const hints: CopyHints = $derived(copyHints(copyDialog.source, copyDialog.touched));
  const develop = $derived(visibleRows(DEVELOP_ROWS, copyDialog.available));
  const photo = $derived(visibleRows(PHOTO_ROWS, copyDialog.available));
  const columns = $derived(splitColumns(develop, photo));
  const developTools = $derived(develop.flatMap((row) => row.tools));
  const developOn = $derived(developTools.filter((tool) => copyDialog.checked[tool]).length);
</script>

{#snippet toolRow(row: Row, depth: number)}
  {@const note = rowNote(row, hints, depth > 1)}
  <CopyToolRow
    label={row.label}
    state={toolsState(copyDialog.checked, row.tools)}
    {depth}
    {note}
    muted={note === 'not modified'}
    dot={rowDot(row, hints, depth > 1)}
    onChange={(on) => copyDialog.setTools(row.tools, on)}
  />
  {#each row.children as child (child.key)}
    {@render toolRow(child, depth + 1)}
  {/each}
{/snippet}

<div class="@container flex flex-col gap-3 touch:gap-2">
  <CopyQuickPicks {hints} />
  <div class="grid grid-cols-1 gap-x-8 @min-[640px]:grid-cols-2">
    <div class="flex flex-col">
      {#if develop.length > 0}
        <CopyToolRow
          label="Develop"
          strong
          state={toolsState(copyDialog.checked, developTools)}
          note="{developOn} of {developTools.length}"
          onChange={(on) => copyDialog.setTools(developTools, on)}
        />
      {/if}
      {#each columns[0] as row (row.key)}
        {@render toolRow(row, 1)}
      {/each}
    </div>
    <div class="flex flex-col">
      {#each columns[1] as row (row.key)}
        {@render toolRow(row, 1)}
      {/each}
      {#if photo.length > 0}
        <div
          class="mt-2 flex items-center border-t border-hairline pt-3 pb-1 text-[10px] font-semibold tracking-wider text-dark/55 uppercase touch:mt-4 touch:border-t-0 touch:pt-2 touch:text-[11px]"
        >
          <span>Off by default</span>
          <span class="ms-auto font-normal tracking-normal normal-case">Photo-specific</span>
        </div>
        {#each photo as row (row.key)}
          {@render toolRow(row, 0)}
        {/each}
      {/if}
    </div>
  </div>
</div>
