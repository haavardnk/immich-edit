<script lang="ts">
  import { tick } from 'svelte';
  import { hint } from '$lib/keybinds';
  import Icon from '$lib/components/Icon.svelte';
  import Popover from '$lib/components/Popover.svelte';
  import { library } from '$lib/stores/library.svelte';
  import { listTags, type TagSummary } from '$lib/api/tags';
  import { toasts } from '$lib/stores/toasts.svelte';
  import type { TagRef } from '$lib/types/asset';
  import { isManagedTag, MANAGED_TAG_PREFIX } from '$lib/reject';
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
    rootClass = 'flex items-center gap-1.5 min-w-0',
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

  let input = $state('');
  let inputEl = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (library.tags.length === 0) {
      void listTags()
        .then((t) => {
          library.tags = t;
        })
        .catch((e: unknown) => toasts.push('error', `tags: ${(e as Error).message}`));
    }
  });

  $effect(() => {
    if (open) {
      void tick().then(() => inputEl?.focus());
    } else {
      input = '';
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

  function rank(value: string, q: string): number {
    const v = value.toLowerCase();
    if (v === q) return 0;
    if (leafOf(v).startsWith(q)) return 1;
    if (v.startsWith(q)) return 2;
    return 3;
  }

  function toRef(t: TagSummary): TagRef {
    return { id: t.id, name: t.name, value: t.value, parentId: t.parentId, color: t.color };
  }

  const allTags = $derived((library.tags ?? []).filter((t) => !isManagedTag(toRef(t))));
  const visibleTags = $derived(tags.filter((t) => !isManagedTag(t)));
  const suggestions = $derived.by(() => {
    const q = input.trim().toLowerCase();
    if (!q) return [];
    const have = new Set(tags.map((t) => t.id));
    return allTags
      .filter((t) => !have.has(t.id))
      .filter((t) => t.value.toLowerCase().includes(q) || t.name.toLowerCase().includes(q))
      .sort((a, b) => {
        const ra = rank(a.value, q);
        const rb = rank(b.value, q);
        if (ra !== rb) return ra - rb;
        return a.value.localeCompare(b.value);
      })
      .slice(0, 8);
  });
  const canCreate = $derived(
    input.trim().length > 0 &&
      !isManagedTag({ id: '', name: '', value: input.trim() }) &&
      !allTags.some((t) => t.value.toLowerCase() === input.trim().toLowerCase())
  );

  async function pick(tag: TagSummary): Promise<void> {
    await onAdd(toRef(tag));
    input = '';
  }

  async function create(): Promise<void> {
    const v = input.trim();
    if (!v) return;
    await onCreate(v);
    input = '';
  }

  function onKey(e: KeyboardEvent): void {
    if (e.key === 'Enter') {
      e.preventDefault();
      if (suggestions.length > 0) {
        void pick(suggestions[0]);
      } else if (canCreate) {
        void create();
      }
    }
  }
</script>

<div class={rootClass}>
  <Icon path={mdiTagOutline} size={14} class="opacity-30 shrink-0" />
  <div class={pillsClass}>
    {#each visibleTags as t (t.id)}
      <span
        class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] bg-white/10 whitespace-nowrap shrink-0"
        title={t.value}
      >
        {#if t.color}
          <span class="w-2 h-2 rounded-full shrink-0" style="background-color: {t.color}"></span>
        {/if}
        {#if parentOf(t.value)}<span class="opacity-40">{parentOf(t.value)}</span>{/if}{leafOf(
          t.value
        )}
        <button
          class="opacity-50 hover:opacity-100 hover:text-red-400 transition"
          title="Remove"
          onclick={() => onRemove(t.id)}
        >
          <Icon path={mdiClose} size={11} />
        </button>
      </span>
    {/each}
  </div>
  <Popover {open} {anchor} {align} {onClose} contentClass="p-2 w-64">
    {#snippet trigger()}
      <button
        type="button"
        class="shrink-0 p-1 rounded hover:bg-white/10 text-immich-dark-fg/60 hover:text-immich-dark-fg transition-colors {open
          ? 'bg-white/10'
          : ''}"
        title={hint('Tags', 'toggleTags')}
        onclick={onToggle}
      >
        <Icon path={mdiPlus} size={14} />
      </button>
    {/snippet}
    {#snippet children()}
      <input
        bind:this={inputEl}
        type="text"
        placeholder="Add tag…"
        class="w-full bg-white/5 text-[11px] rounded px-2 py-1 outline-none focus:bg-white/10"
        value={input}
        oninput={(e) => (input = e.currentTarget.value)}
        onkeydown={onKey}
      />
      {#if suggestions.length > 0 || canCreate}
        <div class="mt-1 max-h-48 overflow-y-auto scrollbar-hidden">
          {#each suggestions as s (s.id)}
            <button
              type="button"
              class="flex items-center gap-1.5 w-full text-left px-2 py-1 text-[11px] rounded hover:bg-white/10"
              onmousedown={(e) => {
                e.preventDefault();
                void pick(s);
              }}
            >
              {#if s.color}
                <span
                  class="w-2 h-2 rounded-full shrink-0"
                  style="background-color: {s.color}"
                ></span>
              {/if}
              <span class="truncate">{s.value}</span>
            </button>
          {/each}
          {#if canCreate}
            <button
              type="button"
              class="flex items-center gap-1 w-full text-left px-2 py-1 text-[11px] rounded hover:bg-white/10 {suggestions.length >
              0
                ? 'border-t border-white/5 mt-1'
                : ''}"
              onmousedown={(e) => {
                e.preventDefault();
                void create();
              }}
            >
              <Icon path={mdiPlus} size={12} />
              Create "{input.trim()}"
            </button>
          {/if}
        </div>
      {/if}
    {/snippet}
  </Popover>
</div>
