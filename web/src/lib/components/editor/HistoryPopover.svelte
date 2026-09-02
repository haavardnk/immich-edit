<script lang="ts">
  import Popover from '$lib/components/Popover.svelte';
  import { mergeProps } from '$lib/utils/mergeProps';
  import { Button, IconButton, Tooltip } from '@immich/ui';
  import { mdiHistory, mdiClose, mdiChevronDown, mdiChevronRight } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';
  import { listEditHistory, type EditHistoryEntry } from '$lib/api/edits';
  import { historyDetails, historyLabel } from '$lib/utils/history';
  import { formatWhen } from '$lib/utils/datetime';

  let open = $state(false);
  let entries = $state<EditHistoryEntry[]>([]);
  let loading = $state(false);
  let busyHash = $state<string | null>(null);
  let expandedId = $state<number | null>(null);

  async function refresh(): Promise<void> {
    if (!editor.assetId) return;
    loading = true;
    try {
      entries = await listEditHistory(editor.assetId);
      if (expandedId !== null && !entries.some((entry) => entry.id === expandedId)) {
        expandedId = null;
      }
    } catch {
      entries = [];
      expandedId = null;
    } finally {
      loading = false;
    }
  }

  async function onOpenChange(next: boolean): Promise<void> {
    if (!next) {
      close();
      return;
    }
    open = true;
    await refresh();
  }

  async function restore(entryId: number): Promise<void> {
    if (!editor.assetId) return;
    busyHash = String(entryId);
    try {
      await editor.restoreHistoryEntry(entryId);
      await refresh();
      close();
    } finally {
      busyHash = null;
    }
  }

  function close(): void {
    open = false;
    expandedId = null;
  }

  function toggleDetails(entryId: number): void {
    expandedId = expandedId === entryId ? null : entryId;
  }
</script>

<Popover
  {open}
  align="end"
  onOpenChange={(v) => void onOpenChange(v)}
  contentClass="w-80 max-h-112 overflow-y-auto"
>
  {#snippet trigger(popoverProps)}
    <Tooltip text="Edit history">
      {#snippet child({ props })}
        <IconButton
          size="small"
          variant="ghost"
          color="secondary"
          class="bg-transparent hover:bg-white/6"
          icon={mdiHistory}
          title=""
          disabled={!editor.assetId}
          aria-label="Edit history"
          {...mergeProps(props, popoverProps)}
        />
      {/snippet}
    </Tooltip>
  {/snippet}
  <div class="flex items-center justify-between border-b border-dark/5 px-3 py-2">
    <span class="text-[11px] uppercase tracking-wider text-dark/65">History</span>
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      icon={mdiClose}
      aria-label="Close"
      onclick={close}
    />
  </div>
  {#if loading}
    <div class="px-3 py-4 text-xs text-dark/65">Loading…</div>
  {:else if entries.length === 0}
    <div class="px-3 py-4 text-xs text-dark/65">No history</div>
  {:else}
    <ul class="py-1">
      {#each entries as entry, i (entry.id)}
        {@const info = historyLabel(entry, entries[i + 1] ?? null, editor.meta?.is_raw ?? false)}
        {@const details = historyDetails(
          entry,
          entries[i + 1] ?? null,
          editor.meta?.is_raw ?? false
        )}
        {@const isCurrent = entry.manifest_hash === editor.savedHash}
        {@const expanded = expandedId === entry.id}
        <li class="border-b border-dark/5 last:border-b-0">
          <div class="flex {isCurrent ? 'bg-light-100' : ''}">
            <Button
              size="tiny"
              variant="ghost"
              color="secondary"
              class="min-w-0 flex-1 flex-col items-start gap-0.5 rounded-none py-2 pl-3 pr-1 text-left"
              disabled={busyHash !== null}
              onclick={() => void restore(entry.id)}
            >
              <span class="flex w-full items-center gap-1.5 truncate text-xs text-dark">
                <span class="truncate">{info.label}</span>
                {#if info.delta}
                  <span class="font-mono tabular-nums text-dark/65">{info.delta}</span>
                {/if}
                {#if busyHash === String(entry.id)}
                  <span class="ml-auto text-[9px] uppercase tracking-wider text-dark/65"
                    >restoring…</span
                  >
                {:else if isCurrent}
                  <span class="ml-auto text-[9px] uppercase tracking-wider text-dark/65"
                    >current</span
                  >
                {/if}
              </span>
              <span class="text-[10px] text-dark/65">{formatWhen(entry.created_at)}</span>
            </Button>
            <IconButton
              size="small"
              variant="ghost"
              color="secondary"
              class="w-9 shrink-0 rounded-none"
              icon={expanded ? mdiChevronDown : mdiChevronRight}
              disabled={busyHash !== null || details.length === 0}
              onclick={() => toggleDetails(entry.id)}
              aria-label={`${expanded ? 'Collapse' : 'Expand'} changes for ${info.label}`}
              aria-expanded={expanded}
              aria-controls={`history-details-${entry.id}`}
            />
          </div>
          {#if expanded}
            <div id={`history-details-${entry.id}`} class="space-y-2 bg-light-100 px-3 pt-1 pb-2.5">
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
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</Popover>
