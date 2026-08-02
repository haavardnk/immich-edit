<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiHistory, mdiClose, mdiChevronRight } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';
  import { listEditHistory, restoreEdits, type EditHistoryEntry } from '$lib/api/edits';
  import { manifestToEdits } from '$lib/types/edits';
  import { historyDetails, historyLabel } from '$lib/utils/history';

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

  async function toggle(): Promise<void> {
    if (open) {
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
      const saved = await restoreEdits(editor.assetId, entryId);
      if (saved) {
        editor.edits = manifestToEdits(saved.manifest);
        editor.savedHash = saved.hash;
      } else {
        const { neutralEdits } = await import('$lib/types/edits');
        editor.edits = neutralEdits();
        editor.savedHash = '';
      }
      editor.onLive();
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

  function formatTime(s: string): string {
    const d = new Date(s);
    return d.toLocaleString();
  }

  let popoverEl: HTMLDivElement | null = $state(null);
  let buttonEl: HTMLButtonElement | null = $state(null);
  let popoverTop = $state(0);
  let popoverRight = $state(0);

  function updatePosition(): void {
    if (!buttonEl) return;
    const r = buttonEl.getBoundingClientRect();
    popoverTop = r.bottom + 4;
    popoverRight = window.innerWidth - r.right;
  }

  onMount(() => {
    function onDoc(e: MouseEvent): void {
      if (!open) return;
      const t = e.target as Node;
      if (popoverEl?.contains(t) || buttonEl?.contains(t)) return;
      close();
    }
    function onScroll(): void {
      if (open) updatePosition();
    }
    document.addEventListener('mousedown', onDoc);
    window.addEventListener('resize', onScroll);
    window.addEventListener('scroll', onScroll, true);
    return () => {
      document.removeEventListener('mousedown', onDoc);
      window.removeEventListener('resize', onScroll);
      window.removeEventListener('scroll', onScroll, true);
    };
  });

  async function toggleAndPosition(): Promise<void> {
    if (!open) updatePosition();
    await toggle();
  }
</script>

<button
  bind:this={buttonEl}
  class="flex items-center justify-center gap-1.5 py-1.5 px-2 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors disabled:opacity-40"
  disabled={!editor.assetId}
  onclick={() => void toggleAndPosition()}
  title="Edit history"
  aria-label="Edit history"
>
  <Icon path={mdiHistory} size={14} />
</button>
{#if open}
  <div
    bind:this={popoverEl}
    style="top: {popoverTop}px; right: {popoverRight}px;"
    class="fixed w-80 max-h-112 overflow-y-auto bg-immich-dark-gray border border-white/10 rounded-lg shadow-xl z-50"
  >
      <div class="flex items-center justify-between px-3 py-2 border-b border-white/5">
        <span class="text-[11px] uppercase tracking-wider text-immich-dark-fg/60">History</span>
        <button class="p-1 hover:bg-white/10 rounded" onclick={close} aria-label="Close">
          <Icon path={mdiClose} size={12} class="opacity-50" />
        </button>
      </div>
      {#if loading}
        <div class="px-3 py-4 text-xs text-immich-dark-fg/40">Loading…</div>
      {:else if entries.length === 0}
        <div class="px-3 py-4 text-xs text-immich-dark-fg/40">No history</div>
      {:else}
        <ul class="py-1">
          {#each entries as entry, i (entry.id)}
            {@const info = historyLabel(entry, entries[i + 1] ?? null)}
            {@const details = historyDetails(entry, entries[i + 1] ?? null)}
            {@const isCurrent = entry.manifest_hash === editor.savedHash}
            {@const expanded = expandedId === entry.id}
            <li class="border-b border-white/5 last:border-b-0">
              <div class="flex {isCurrent ? 'bg-white/5' : ''}">
                <button
                  class="min-w-0 flex-1 text-left pl-3 pr-1 py-2 hover:bg-white/5 transition-colors disabled:opacity-40 flex flex-col gap-0.5"
                  disabled={busyHash !== null}
                  onclick={() => void restore(entry.id)}
                >
                  <span class="text-xs text-immich-dark-fg/90 truncate flex items-center gap-1.5">
                    <span class="truncate">{info.label}</span>
                    {#if info.delta}
                      <span class="font-mono tabular-nums text-immich-dark-fg/50">{info.delta}</span>
                    {/if}
                    {#if isCurrent}
                      <span class="ml-auto text-[9px] uppercase tracking-wider text-immich-dark-fg/40">current</span>
                    {/if}
                  </span>
                  <span class="text-[10px] text-immich-dark-fg/40">{formatTime(entry.created_at)}</span>
                </button>
                <button
                  class="w-9 shrink-0 flex items-center justify-center hover:bg-white/5 transition-colors disabled:opacity-20"
                  disabled={busyHash !== null || details.length === 0}
                  onclick={() => toggleDetails(entry.id)}
                  aria-label={`${expanded ? 'Collapse' : 'Expand'} changes for ${info.label}`}
                  aria-expanded={expanded}
                  aria-controls={`history-details-${entry.id}`}
                >
                  <Icon
                    path={mdiChevronRight}
                    size={15}
                    class="opacity-50 transition-transform {expanded ? 'rotate-90' : ''}"
                  />
                </button>
              </div>
              {#if expanded}
                <div
                  id={`history-details-${entry.id}`}
                  class="px-3 pt-1 pb-2.5 bg-black/10 space-y-2"
                >
                  {#each details as group (group.key)}
                    <div>
                      <div class="text-[9px] uppercase tracking-wider text-immich-dark-fg/35 mb-0.5">
                        {group.label}
                      </div>
                      <div class="space-y-0.5">
                        {#each group.items as item}
                          {#if item.kind === 'value'}
                            <div class="flex items-baseline justify-between gap-2 text-[10px]">
                              <span class="text-immich-dark-fg/60 truncate">{item.label}</span>
                              <span class="font-mono tabular-nums text-immich-dark-fg/80 whitespace-nowrap">
                                {item.before} → {item.after}
                              </span>
                            </div>
                          {:else}
                            <div class="text-[10px] text-immich-dark-fg/65">{item.text}</div>
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
    </div>
  {/if}
