<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiHistory, mdiClose } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';
  import { listEditHistory, restoreEdits, type EditHistoryEntry } from '$lib/api/edits';
  import { manifestToEdits } from '$lib/types/edits';

  let open = $state(false);
  let entries = $state<EditHistoryEntry[]>([]);
  let loading = $state(false);
  let busyHash = $state<string | null>(null);

  async function refresh(): Promise<void> {
    if (!editor.assetId) return;
    loading = true;
    try {
      entries = await listEditHistory(editor.assetId);
    } catch {
      entries = [];
    } finally {
      loading = false;
    }
  }

  async function toggle(): Promise<void> {
    open = !open;
    if (open) await refresh();
  }

  async function restore(hash: string): Promise<void> {
    if (!editor.assetId) return;
    busyHash = hash;
    try {
      const saved = await restoreEdits(editor.assetId, hash);
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
    } finally {
      busyHash = null;
    }
  }

  function close(): void {
    open = false;
  }

  function formatTime(s: string): string {
    const d = new Date(s);
    return d.toLocaleString();
  }

  function summary(entry: EditHistoryEntry): string {
    if (entry.deleted) return 'Reset to original';
    const e = entry.edits;
    if (!e) return entry.manifest_hash.slice(0, 8);
    const parts: string[] = [];
    if (Math.abs(e.basic.exposure_ev) > 1e-3) parts.push(`exp ${e.basic.exposure_ev.toFixed(2)}`);
    if (Math.abs(e.basic.contrast) > 1e-3) parts.push(`contrast ${e.basic.contrast.toFixed(2)}`);
    if (Math.abs(e.tone.highlights) > 1e-3) parts.push(`hi ${e.tone.highlights.toFixed(2)}`);
    if (Math.abs(e.tone.shadows) > 1e-3) parts.push(`sh ${e.tone.shadows.toFixed(2)}`);
    if (Math.abs(e.basic.saturation) > 1e-3) parts.push(`sat ${e.basic.saturation.toFixed(2)}`);
    if (parts.length === 0) return entry.manifest_hash.slice(0, 8);
    return parts.slice(0, 3).join(', ');
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
    class="fixed w-72 max-h-80 overflow-y-auto bg-immich-dark-gray border border-white/10 rounded-lg shadow-xl z-50"
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
          {#each entries as entry (entry.id)}
            <li>
              <button
                class="w-full text-left px-3 py-2 hover:bg-white/5 transition-colors disabled:opacity-40 flex flex-col gap-0.5"
                disabled={busyHash !== null}
                onclick={() => void restore(entry.manifest_hash)}
              >
                <span class="text-xs text-immich-dark-fg/90 truncate">{summary(entry)}</span>
                <span class="text-[10px] text-immich-dark-fg/40">{formatTime(entry.created_at)}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
