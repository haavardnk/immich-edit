<script lang="ts">
  import { tick } from 'svelte';
  import Icon from '$lib/components/Icon.svelte';
  import Popover from '$lib/components/Popover.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { library } from '$lib/stores/library.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { listTags } from '$lib/api/tags';
  import { toasts } from '$lib/stores/toasts.svelte';
  import { mdiClose, mdiPlus, mdiTagOutline } from '@mdi/js';

  let input = $state('');
  let inputEl = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (library.tags.length === 0) {
      void listTags()
        .then((t) => (library.tags = t))
        .catch((e: unknown) => toasts.push('error', `tags: ${(e as Error).message}`));
    }
  });

  $effect(() => {
    if (ui.tagsPopoverOpen) {
      void tick().then(() => inputEl?.focus());
    } else {
      input = '';
    }
  });

  const tags = $derived(editor.asset?.tags ?? []);
  const allTags = $derived(library.tags ?? []);
  const suggestions = $derived.by(() => {
    const q = input.trim().toLowerCase();
    if (!q) return [];
    const have = new Set(tags.map((t) => t.id));
    return allTags
      .filter((t) => !have.has(t.id))
      .filter((t) => t.value.toLowerCase().includes(q) || t.name.toLowerCase().includes(q))
      .slice(0, 8);
  });
  const canCreate = $derived(
    input.trim().length > 0 &&
      !allTags.some((t) => t.value.toLowerCase() === input.trim().toLowerCase())
  );

  async function pick(tagId: string): Promise<void> {
    const tag = allTags.find((t) => t.id === tagId);
    if (!tag) return;
    await editor.addTag({ id: tag.id, name: tag.name, value: tag.value });
    input = '';
  }

  async function create(): Promise<void> {
    const v = input.trim();
    if (!v) return;
    await editor.createAndAddTag(v);
    input = '';
  }

  function onKey(e: KeyboardEvent): void {
    if (e.key === 'Enter') {
      e.preventDefault();
      if (suggestions.length > 0) {
        void pick(suggestions[0].id);
      } else if (canCreate) {
        void create();
      }
    }
  }
</script>

<div class="flex items-center gap-1.5 min-w-0">
  <Icon path={mdiTagOutline} size={14} class="opacity-30 shrink-0" />
  <div class="flex items-center gap-1 overflow-x-auto scrollbar-hidden min-w-0">
    {#each tags as t (t.id)}
      <span
        class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] bg-white/10 whitespace-nowrap shrink-0"
      >
        {t.value}
        <button
          class="opacity-50 hover:opacity-100 hover:text-red-400 transition"
          title="Remove"
          onclick={() => editor.removeTag(t.id)}
        >
          <Icon path={mdiClose} size={11} />
        </button>
      </span>
    {/each}
  </div>
  <Popover
    open={ui.tagsPopoverOpen}
    anchor="top"
    align="start"
    onClose={ui.closeTagsPopover}
    contentClass="p-2 w-64"
  >
    {#snippet trigger()}
      <button
        type="button"
        class="shrink-0 p-1 rounded hover:bg-white/10 text-immich-dark-fg/60 hover:text-immich-dark-fg transition-colors {ui.tagsPopoverOpen ? 'bg-white/10' : ''}"
        title="Tags (T)"
        onclick={ui.toggleTagsPopover}
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
        bind:value={input}
        onkeydown={onKey}
      />
      {#if suggestions.length > 0 || canCreate}
        <div class="mt-1 max-h-48 overflow-y-auto scrollbar-hidden">
          {#each suggestions as s (s.id)}
            <button
              type="button"
              class="block w-full text-left px-2 py-1 text-[11px] rounded hover:bg-white/10"
              onmousedown={(e) => {
                e.preventDefault();
                void pick(s.id);
              }}
            >
              {s.value}
            </button>
          {/each}
          {#if canCreate}
            <button
              type="button"
              class="flex items-center gap-1 w-full text-left px-2 py-1 text-[11px] rounded hover:bg-white/10 {suggestions.length > 0 ? 'border-t border-white/5 mt-1' : ''}"
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
