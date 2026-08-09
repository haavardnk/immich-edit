<script lang="ts">
  import { goto } from '$app/navigation';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiCheck, mdiClose, mdiContentDuplicate, mdiDelete, mdiPencil } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';
  import { deleteCopy, listCopies, renameCopy, type CopyRecord } from '$lib/api/copies';
  import { isCopy, sourceId } from '$lib/assetKey';
  import { createVirtualCopy } from '$lib/copies';
  import { browsing } from '$lib/stores/browsing.svelte';

  let copies = $state<CopyRecord[]>([]);
  let busy = $state(false);
  let editingId = $state<string | null>(null);
  let editName = $state('');

  const assetId = $derived(editor.assetId);
  const master = $derived(assetId ? sourceId(assetId) : null);

  $effect(() => {
    const id = assetId;
    if (!id) {
      copies = [];
      return;
    }
    void listCopies(id).then((list) => {
      copies = list;
    });
  });

  async function create(): Promise<void> {
    if (!assetId || busy) return;
    busy = true;
    try {
      await createVirtualCopy(assetId);
    } finally {
      busy = false;
    }
  }

  function startRename(copy: CopyRecord): void {
    editingId = copy.id;
    editName = copy.name ?? '';
  }

  async function commitRename(): Promise<void> {
    if (!editingId) return;
    const updated = await renameCopy(editingId, editName.trim() || null);
    copies = copies.map((c) => (c.id === updated.id ? updated : c));
    browsing.patch(updated.id, { copyLabel: updated.name ?? undefined });
    editingId = null;
  }

  function onRenameKey(e: KeyboardEvent): void {
    if (e.key !== 'Enter' && e.key !== 'Escape') return;
    e.preventDefault();
    e.stopPropagation();
    if (e.key === 'Escape') {
      editingId = null;
      return;
    }
    void commitRename();
  }

  async function remove(copy: CopyRecord): Promise<void> {
    await deleteCopy(copy.id);
    copies = copies.filter((c) => c.id !== copy.id);
    browsing.remove(copy.id);
    if (assetId === copy.id && master) await goto(`/assets/${master}`);
  }

  function label(copy: CopyRecord): string {
    return copy.name ?? `Copy ${copy.id.split('_').at(-1)}`;
  }

  function focusOnMount(node: HTMLInputElement): void {
    node.focus();
    node.select();
  }
</script>

<div class="flex flex-col gap-2 pb-1">
  <button
    class="flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
    disabled={!assetId || busy}
    onclick={() => void create()}
  >
    <Icon path={mdiContentDuplicate} size={14} />
    Create virtual copy
  </button>

  <div class="flex flex-col gap-0.5">
    <a
      href={master ? `/assets/${master}` : '#'}
      class="px-2 py-1.5 rounded-lg text-xs hover:bg-white/10 transition-colors"
      class:text-immich-dark-primary={assetId != null && !isCopy(assetId)}
    >
      Original
    </a>
    {#each copies as copy (copy.id)}
      {#if editingId === copy.id}
        <div class="flex items-center gap-1 px-1 py-1">
          <input
            class="input bg-white/5 rounded-lg text-xs h-auto py-1 min-h-0 flex-1 min-w-0"
            placeholder="Name"
            bind:value={editName}
            onkeydown={onRenameKey}
            use:focusOnMount
          />
          <button
            class="p-1.5 rounded-lg hover:bg-white/10 transition-colors"
            onclick={() => void commitRename()}
            aria-label="Save name"
          >
            <Icon path={mdiCheck} size={13} />
          </button>
          <button
            class="p-1.5 rounded-lg hover:bg-white/10 transition-colors"
            onclick={() => (editingId = null)}
            aria-label="Cancel"
          >
            <Icon path={mdiClose} size={13} />
          </button>
        </div>
      {:else}
        <div class="flex items-center gap-1 group">
          <a
            href={`/assets/${copy.id}`}
            class="flex-1 min-w-0 truncate px-2 py-1.5 rounded-lg text-xs hover:bg-white/10 transition-colors"
            class:text-immich-dark-primary={assetId === copy.id}
          >
            {label(copy)}
          </a>
          <button
            class="p-1.5 rounded-lg text-immich-dark-fg/40 hover:text-immich-dark-fg hover:bg-white/10 transition-colors"
            onclick={() => startRename(copy)}
            aria-label="Rename copy"
            title="Rename"
          >
            <Icon path={mdiPencil} size={13} />
          </button>
          <button
            class="p-1.5 rounded-lg text-immich-dark-fg/40 hover:text-red-400 hover:bg-white/10 transition-colors"
            onclick={() => void remove(copy)}
            aria-label="Delete copy"
            title="Delete"
          >
            <Icon path={mdiDelete} size={13} />
          </button>
        </div>
      {/if}
    {/each}
  </div>
</div>
