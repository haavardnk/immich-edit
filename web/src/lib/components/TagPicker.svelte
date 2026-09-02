<script lang="ts">
  import { hint } from '$lib/keybinds';
  import SearchableSelect from '$lib/components/SearchableSelect.svelte';
  import Popover from '$lib/components/Popover.svelte';
  import { library } from '$lib/stores/library.svelte';
  import { listTags, type TagSummary } from '$lib/api/tags';
  import { toasts } from '$lib/stores/toasts.svelte';
  import type { TagRef } from '$lib/types/asset';
  import { isManagedTag } from '$lib/reject';
  import { mergeProps } from '$lib/utils/mergeProps';
  import { Icon, IconButton } from '@immich/ui';
  import { mdiClose, mdiPlus, mdiTagOutline } from '@mdi/js';

  type Anchor = 'top' | 'bottom';
  type Align = 'start' | 'end' | 'center';

  let {
    tags,
    open,
    onToggle,
    onClose,
    onAdd,
    onRemove,
    onCreate,
    anchor = 'top',
    align = 'start',
    rootClass = 'flex min-w-0 items-center gap-1.5 overflow-hidden',
    pillsClass = 'flex items-center gap-1 overflow-x-auto scrollbar-hidden min-w-0'
  }: {
    tags: TagRef[];
    open: boolean;
    onToggle: () => void;
    onClose: () => void;
    onAdd: (tag: TagRef) => void | Promise<void>;
    onRemove: (tagId: string) => void | Promise<void>;
    onCreate: (value: string) => TagRef | null | Promise<TagRef | null>;
    anchor?: Anchor;
    align?: Align;
    rootClass?: string;
    pillsClass?: string;
  } = $props();

  $effect(() => {
    if (library.tags.length === 0) {
      void listTags()
        .then((t) => {
          library.tags = t;
        })
        .catch((e: unknown) => toasts.push('error', `tags: ${(e as Error).message}`));
    }
  });

  function leafOf(value: string): string {
    const i = value.lastIndexOf('/');
    return i >= 0 ? value.slice(i + 1) : value;
  }

  function parentOf(value: string): string {
    const i = value.lastIndexOf('/');
    return i >= 0 ? value.slice(0, i + 1) : '';
  }

  function toRef(t: TagSummary): TagRef {
    return { id: t.id, name: t.name, value: t.value, parentId: t.parentId, color: t.color };
  }

  const allTags = $derived((library.tags ?? []).filter((t) => !isManagedTag(toRef(t))));
  const visibleTags = $derived(tags.filter((t) => !isManagedTag(t)));
  const selectedIds = $derived(visibleTags.map((tag) => tag.id));

  function updateSelection(next: string[]): void {
    const current = new Set(selectedIds);
    const addedId = next.find((id) => !current.has(id));
    if (addedId) {
      const tag = allTags.find((item) => item.id === addedId);
      if (tag) void onAdd(toRef(tag));
      return;
    }
    const nextSet = new Set(next);
    const removedId = selectedIds.find((id) => !nextSet.has(id));
    if (removedId) void onRemove(removedId);
  }

  async function create(value: string): Promise<void> {
    await onCreate(value);
  }
</script>

<div class={rootClass}>
  <Icon icon={mdiTagOutline} size="14px" class="opacity-30 shrink-0" aria-hidden="true" />
  <div class={pillsClass}>
    {#each visibleTags as t (t.id)}
      <span
        class="inline-flex h-6 shrink-0 items-center gap-1 rounded-full bg-primary-200 pl-2 text-[11px] whitespace-nowrap"
        title={t.value}
      >
        {#if t.color}
          <span class="w-2 h-2 rounded-full shrink-0" style="background-color: {t.color}"></span>
        {/if}
        {#if parentOf(t.value)}<span class="opacity-40">{parentOf(t.value)}</span>{/if}{leafOf(
          t.value
        )}
        <IconButton
          size="tiny"
          variant="ghost"
          color="secondary"
          class="opacity-50 hover:text-red-400 hover:opacity-100"
          icon={mdiClose}
          title="Remove"
          aria-label="Remove"
          onclick={() => onRemove(t.id)}
        />
      </span>
    {/each}
  </div>
  <Popover
    {open}
    {anchor}
    {align}
    onOpenChange={(v) => (v ? onToggle() : onClose())}
    contentClass="w-72 overflow-visible! border-0! bg-transparent! p-0 shadow-none! ring-0!"
  >
    {#snippet trigger(popoverProps)}
      <IconButton
        size="tiny"
        variant={open ? 'filled' : 'ghost'}
        color={open ? 'primary' : 'secondary'}
        class="shrink-0"
        icon={mdiPlus}
        title={hint('Tags', 'toggleTags')}
        aria-label={hint('Tags', 'toggleTags')}
        {...mergeProps(popoverProps)}
      />
    {/snippet}
    <SearchableSelect
      options={allTags}
      selected={selectedIds}
      getId={(tag) => tag.id}
      getLabel={(tag) => tag.value}
      getColor={(tag) => tag.color}
      placeholder="Add tags…"
      compact
      color="neutral"
      side="top"
      autofocus
      closeOnSelect={false}
      onSelectedChange={updateSelection}
      onCreate={create}
    />
  </Popover>
</div>
