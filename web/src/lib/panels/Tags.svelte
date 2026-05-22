<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { library } from '$lib/stores/library.svelte';
  import { listTags } from '$lib/api/tags';
  import { mdiClose, mdiPlus } from '@mdi/js';

  let input = $state('');
  let open = $state(false);

  $effect(() => {
    if (library.tags.length === 0) {
      void listTags()
        .then((t) => (library.tags = t))
        .catch(() => {});
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
    open = false;
  }

  async function create(): Promise<void> {
    const v = input.trim();
    if (!v) return;
    await editor.createAndAddTag(v);
    input = '';
    open = false;
  }

  function onKey(e: KeyboardEvent): void {
    if (e.key === 'Enter') {
      e.preventDefault();
      if (suggestions.length > 0) {
        void pick(suggestions[0].id);
      } else if (canCreate) {
        void create();
      }
    } else if (e.key === 'Escape') {
      input = '';
      open = false;
    }
  }
</script>

<div class="flex flex-col gap-2">
  <div class="flex flex-wrap gap-1">
    {#each tags as t (t.id)}
      <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] bg-white/10">
        {t.value}
        <button
          class="hover:text-red-400"
          title="Remove"
          onclick={() => editor.removeTag(t.id)}
        >
          <Icon path={mdiClose} size={12} />
        </button>
      </span>
    {/each}
    {#if tags.length === 0}
      <span class="text-[11px] text-immich-dark-fg/30 italic">No tags</span>
    {/if}
  </div>

  <div class="relative">
    <input
      type="text"
      placeholder="Add tag…"
      class="input input-xs input-bordered w-full text-[11px]"
      bind:value={input}
      onfocus={() => (open = true)}
      onblur={() => setTimeout(() => (open = false), 150)}
      onkeydown={onKey}
    />
    {#if open && (suggestions.length > 0 || canCreate)}
      <div
        class="absolute z-20 left-0 right-0 mt-1 bg-immich-dark-bg border border-white/10 rounded-md shadow-lg max-h-48 overflow-y-auto"
      >
        {#each suggestions as s (s.id)}
          <button
            type="button"
            class="block w-full text-left px-2 py-1 text-[11px] hover:bg-white/10"
            onmousedown={() => pick(s.id)}
          >
            {s.value}
          </button>
        {/each}
        {#if canCreate}
          <button
            type="button"
            class="flex items-center gap-1 w-full text-left px-2 py-1 text-[11px] hover:bg-white/10 border-t border-white/5"
            onmousedown={() => create()}
          >
            <Icon path={mdiPlus} size={12} />
            Create "{input.trim()}"
          </button>
        {/if}
      </div>
    {/if}
  </div>
</div>
