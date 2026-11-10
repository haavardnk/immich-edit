<script lang="ts">
  import type { HistoryDetailGroup } from '#lib/utils/history.ts';

  let { id, details }: { id: string; details: HistoryDetailGroup[] } = $props();
</script>

<div {id} class="space-y-2 bg-light-100 px-3 pt-1 pb-2.5">
  {#each details as group (group.key)}
    <div>
      <div class="mb-0.5 text-[9px] uppercase tracking-wider text-dark/65">
        {group.label}
      </div>
      <div class="space-y-0.5">
        {#each group.items as item, i (i)}
          {#if item.kind === 'value'}
            <div class="flex items-baseline justify-between gap-2 text-[10px]">
              <span class="text-dark/65 truncate">{item.label}</span>
              <span class="whitespace-nowrap font-mono text-dark tabular-nums">
                {item.before} → {item.after}
              </span>
            </div>
          {:else}
            <div class="text-[10px] text-dark/65">{item.text}</div>
          {/if}
        {/each}
      </div>
    </div>
  {/each}
</div>
