<script lang="ts">
  import type { EditSnapshot } from '#lib/api/snapshots.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { tick, untrack } from 'svelte';
  import NewSnapshotRow from './NewSnapshotRow.svelte';
  import SnapshotRow from './SnapshotRow.svelte';

  let { oncompare = () => {} }: { oncompare?: () => void } = $props();

  const store = editor.snapshots;

  let list = $state<HTMLElement | null>(null);
  let editingId = $state<string | null>(null);

  $effect(() => {
    const assetId = editor.assetId;
    const items = store.items;
    if (layout.mode !== 'touch' || !assetId) return;
    untrack(() => store.thumbs.request(assetId, items));
  });

  function created(snapshot: EditSnapshot, named: boolean): void {
    if (!named) editingId = snapshot.id;
  }

  async function removed(index: number): Promise<void> {
    await tick();
    const rows = list?.querySelectorAll<HTMLElement>('[data-snapshot-restore]') ?? [];
    const next = rows[Math.min(index, rows.length - 1)];
    (next ?? list?.querySelector<HTMLElement>('[data-snapshot-new]'))?.focus();
  }
</script>

<section aria-label="Snapshots" class="flex flex-col" bind:this={list}>
  <span class="px-3 pt-2 pb-1 text-[11px] uppercase tracking-wider text-dark/65 touch:hidden">
    Snapshots
  </span>
  {#if store.items.length > 0}
    <ul class="flex flex-col">
      {#each store.items as snapshot, index (snapshot.id)}
        <SnapshotRow
          {snapshot}
          restored={store.restoredId === snapshot.id}
          editing={editingId === snapshot.id}
          onrename={() => (editingId = snapshot.id)}
          onendrename={() => (editingId = null)}
          onremoved={() => void removed(index)}
          {oncompare}
        />
      {/each}
    </ul>
  {/if}
  <div class="touch:order-first">
    <NewSnapshotRow oncreated={created} />
  </div>
</section>
