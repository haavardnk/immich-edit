<script lang="ts">
  import { stepStates } from '#lib/editor/historySteps.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { onDestroy, untrack } from 'svelte';
  import HistoryRow from './HistoryRow.svelte';

  const steps = editor.historySteps;

  let expandedId = $state<number | null>(null);

  const states = $derived(stepStates(steps.entries, steps.hoveredId));
  const keepsNewer = $derived(states.some((state) => state.superseded));

  function toggle(entryId: number): void {
    expandedId = expandedId === entryId ? null : entryId;
  }

  $effect(() => {
    void steps.revision;
    untrack(() => void steps.refresh());
  });

  onDestroy(() => steps.leave());
</script>

{#if steps.loading}
  <div class="px-3 py-4 text-xs text-dark/65">Loading…</div>
{:else if steps.entries.length === 0}
  <div class="px-3 py-4 text-xs text-dark/65">No history</div>
{:else}
  <ul class="py-1 touch:py-0" onpointerleave={() => steps.leave()}>
    {#each steps.entries as entry, i (entry.id)}
      {@const step = states[i]}
      {#if step}
        <HistoryRow
          {entry}
          previous={steps.entries[i + 1] ?? null}
          isRaw={editor.isRaw}
          {step}
          busy={steps.busyId !== null}
          restoring={steps.busyId === entry.id}
          expanded={expandedId === entry.id}
          onrestore={() => void steps.restore(entry)}
          onhover={() => steps.hover(entry)}
          onleave={() => steps.leave()}
          ontoggle={() => toggle(entry.id)}
        />
      {/if}
    {/each}
  </ul>
  {#if keepsNewer}
    <p
      class="sticky bottom-0 border-t border-dark/5 bg-light-100 px-3 py-2 text-[11px] leading-snug text-dark/65"
    >
      <b class="font-semibold text-warning">Newer steps are kept</b> until your next edit.
    </p>
  {/if}
{/if}
