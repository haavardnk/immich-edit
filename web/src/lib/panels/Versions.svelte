<script lang="ts">
  import EditableLabel from '$lib/components/EditableLabel.svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { mdiCheck, mdiClose, mdiContentDuplicate, mdiDelete, mdiPencil } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';
  import { deleteCopy, listCopies, renameCopy, type CopyRecord } from '$lib/api/copies';
  import { isCopy, sourceId } from '$lib/assetKey';
  import { createVirtualCopy } from '$lib/copies';
  import { editorHref } from '$lib/editorNavigation';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { Button, IconButton } from '@immich/ui';

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
      await createVirtualCopy(assetId, { returnPath: page.url.searchParams.get('from') });
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

  async function remove(copy: CopyRecord): Promise<void> {
    await deleteCopy(copy.id);
    copies = copies.filter((c) => c.id !== copy.id);
    browsing.remove(copy.id);
    if (assetId === copy.id && master)
      await goto(editorHref(master, page.url.searchParams.get('from')));
  }

  function label(copy: CopyRecord): string {
    return copy.name ?? `Copy ${copy.id.split('_').at(-1)}`;
  }
</script>

<div class="flex flex-col gap-1 pb-1">
  <Button
    size="tiny"
    variant="ghost"
    color="secondary"
    class="h-7 panel-action"
    leadingIcon={mdiContentDuplicate}
    disabled={!assetId || busy}
    onclick={() => void create()}
  >
    Create virtual copy
  </Button>

  <nav aria-label="Versions" class="flex flex-col gap-0.5">
    <a
      href={master ? editorHref(master, page.url.searchParams.get('from')) : '#'}
      aria-current={assetId != null && !isCopy(assetId) ? 'page' : undefined}
      class="flex h-7 items-center gap-2 rounded-sm px-1.5 text-[11px] transition-colors hover:bg-ghost"
      class:text-primary={assetId != null && !isCopy(assetId)}
    >
      <span
        class="size-1.5 shrink-0 rounded-full {assetId != null && !isCopy(assetId)
          ? 'bg-primary'
          : 'border border-dark/35'}"
      ></span>
      <span class="font-medium">Original</span>
      <span class="ml-auto text-[9px] text-dark/35">Source</span>
    </a>
    {#each copies as copy (copy.id)}
      {#if editingId === copy.id}
        <div class="flex h-7 items-center gap-1 px-1">
          <div class="flex-1 min-w-0">
            <EditableLabel
              round
              placeholder="Name"
              ariaLabel="Version name"
              bind:value={editName}
              oncommit={commitRename}
              oncancel={() => (editingId = null)}
            />
          </div>
          <IconButton
            size="tiny"
            variant="ghost"
            color="secondary"
            icon={mdiCheck}
            onclick={() => void commitRename()}
            aria-label="Save name"
          />
          <IconButton
            size="tiny"
            variant="ghost"
            color="secondary"
            icon={mdiClose}
            onclick={() => (editingId = null)}
            aria-label="Cancel"
          />
        </div>
      {:else}
        <div class="group flex h-7 items-center gap-0.5">
          <a
            href={editorHref(copy.id, page.url.searchParams.get('from'))}
            aria-current={assetId === copy.id ? 'page' : undefined}
            class="flex h-7 min-w-0 flex-1 items-center gap-2 truncate rounded-sm px-1.5 text-[11px] transition-colors hover:bg-ghost"
            class:text-primary={assetId === copy.id}
          >
            <span
              class="size-1.5 shrink-0 rounded-full {assetId === copy.id
                ? 'bg-primary'
                : 'border border-dark/35'}"
            ></span>
            <span class="truncate font-medium">{label(copy)}</span>
          </a>
          <div
            class="flex items-center gap-0.5 transition-opacity {assetId === copy.id
              ? 'opacity-100'
              : 'opacity-0 group-hover:opacity-100 group-focus-within:opacity-100'}"
          >
            <IconButton
              size="tiny"
              variant="ghost"
              color="secondary"
              icon={mdiPencil}
              title="Rename"
              aria-label="Rename copy"
              onclick={() => startRename(copy)}
            />
            <IconButton
              size="tiny"
              variant="ghost"
              color="secondary"
              icon={mdiDelete}
              title="Delete"
              aria-label="Delete copy"
              onclick={() => void remove(copy)}
            />
          </div>
        </div>
      {/if}
    {/each}
  </nav>
</div>
