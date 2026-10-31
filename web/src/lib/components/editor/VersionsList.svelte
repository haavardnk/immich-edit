<script lang="ts">
  import EditableLabel from '#lib/components/EditableLabel.svelte';
  import DeleteConfirmation from '#lib/components/DeleteConfirmation.svelte';
  import MenuItem from '#lib/components/MenuItem.svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { mdiCheck, mdiClose, mdiContentDuplicate, mdiPencil } from '@mdi/js';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { deleteCopy, listCopies, renameCopy, type CopyRecord } from '#lib/api/copies.ts';
  import { isCopy, sourceId } from '#lib/browse/assetKey.ts';
  import { createVirtualCopy } from '#lib/editor/copies.ts';
  import { editorHref } from '#lib/editor/navigation.ts';
  import { keysFor } from '#lib/shortcuts/labels.ts';
  import { browsing } from '#lib/stores/browsing.svelte.ts';
  import { IconButton } from '@immich/ui';

  let { onnavigate }: { onnavigate: () => void } = $props();

  let copies = $state<CopyRecord[]>([]);
  let editingId = $state<string | null>(null);
  let editName = $state('');
  let deletingId = $state<string | null>(null);

  const assetId = $derived(editor.assetId);
  const master = $derived(assetId ? sourceId(assetId) : null);
  const from = $derived(page.url.searchParams.get('from'));
  const onOriginal = $derived(assetId != null && !isCopy(assetId));

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

  function startRename(copy: CopyRecord): void {
    deletingId = null;
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
    if (assetId !== copy.id || !master) return;
    onnavigate();
    await goto(editorHref(master, from));
  }

  function label(copy: CopyRecord): string {
    return copy.name ?? `Copy ${copy.id.split('_').at(-1)}`;
  }
</script>

{#snippet dot(active: boolean)}
  <span class="flex w-3.5 shrink-0 justify-center" aria-hidden="true">
    <span class="size-1.5 rounded-full {active ? 'bg-primary' : 'border border-dark/35'}"></span>
  </span>
{/snippet}

<nav aria-label="Versions" class="flex flex-col">
  <span class="px-3 pt-1.5 pb-1 text-[10px] tracking-wider text-dark/65 uppercase">Versions</span>
  <a
    href={master ? editorHref(master, from) : '#'}
    aria-current={onOriginal ? 'page' : undefined}
    class="flex h-7 items-center gap-2.5 px-3 text-xs transition-colors hover:bg-ghost"
    class:text-primary={onOriginal}
    onclick={onnavigate}
  >
    {@render dot(onOriginal)}
    <span class="truncate">Original</span>
  </a>
  {#each copies as copy (copy.id)}
    {#if editingId === copy.id}
      <div class="flex h-8 items-center gap-0.5 pr-1.5 pl-2">
        <div class="min-w-0 flex-1">
          <EditableLabel
            compact
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
      <div class="group flex h-7 items-center pr-1.5 transition-colors hover:bg-ghost">
        <a
          href={editorHref(copy.id, from)}
          aria-current={assetId === copy.id ? 'page' : undefined}
          class="flex h-full min-w-0 flex-1 items-center gap-2.5 pl-3 text-xs"
          class:text-primary={assetId === copy.id}
          onclick={onnavigate}
        >
          {@render dot(assetId === copy.id)}
          <span class="truncate">{label(copy)}</span>
        </a>
        <div
          class="flex items-center transition-opacity {deletingId === copy.id
            ? ''
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
          <DeleteConfirmation
            bind:pending={
              () => deletingId === copy.id,
              (value: boolean) => (deletingId = value ? copy.id : null)
            }
            label="Delete copy"
            title="Delete"
            confirmLabel="Confirm delete copy"
            onconfirm={() => remove(copy)}
          />
        </div>
      </div>
    {/if}
  {/each}
  <MenuItem
    icon={mdiContentDuplicate}
    label="Create virtual copy"
    keys={keysFor('createVirtualCopy')}
    disabled={!assetId}
    onclick={() => {
      if (assetId) void createVirtualCopy(assetId, { returnPath: from });
      onnavigate();
    }}
  />
</nav>
