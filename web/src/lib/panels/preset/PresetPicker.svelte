<script lang="ts">
  import Popover from '$lib/components/Popover.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { presets } from '$lib/stores/presets.svelte';
  import { mdiChevronDown, mdiMagnify } from '@mdi/js';

  let {
    selectedId = $bindable(),
    disabled = false,
    placeholder = 'Select a preset…',
  }: {
    selectedId: string | null;
    disabled?: boolean;
    placeholder?: string;
  } = $props();

  let open = $state(false);
  let query = $state('');

  const selected = $derived(presets.presets.find((p) => p.id === selectedId) ?? null);
  const showSearch = $derived(presets.presets.length > 8);

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return presets.grouped;
    return presets.grouped
      .map(({ group, items }) => ({
        group,
        items: items.filter((p) => p.name.toLowerCase().includes(q)),
      }))
      .filter(({ items }) => items.length > 0);
  });

  function pick(id: string): void {
    selectedId = id;
    open = false;
    query = '';
  }

  function close(): void {
    open = false;
    query = '';
  }
</script>

<Popover {open} onClose={close} rootClass="w-full" contentClass="w-full max-h-72 overflow-hidden flex flex-col">
  {#snippet trigger()}
    <button
      class="flex w-full items-center gap-2 px-2.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
      {disabled}
      onclick={() => (open = !open)}
    >
      <span class="flex-1 text-left truncate {selected ? '' : 'text-immich-dark-fg/40'}">
        {selected ? selected.name : placeholder}
      </span>
      <Icon path={mdiChevronDown} size={14} class="opacity-50 flex-none" />
    </button>
  {/snippet}

  {#if showSearch}
    <div class="p-1.5 border-b border-white/10">
      <div class="flex items-center gap-1.5 px-2 py-1 rounded-lg bg-white/5">
        <Icon path={mdiMagnify} size={14} class="opacity-40 flex-none" />
        <input
          class="flex-1 bg-transparent text-xs outline-none placeholder:text-immich-dark-fg/30"
          placeholder="Search presets"
          bind:value={query}
        />
      </div>
    </div>
  {/if}

  <div class="overflow-y-auto py-1">
    {#if filtered.length === 0}
      <div class="px-3 py-2 text-xs text-immich-dark-fg/30">No matches.</div>
    {:else}
      {#each filtered as { group, items } (group)}
        {#if group}
          <div class="px-3 pt-1.5 pb-0.5 text-[10px] uppercase tracking-wider text-immich-dark-fg/40">
            {group}
          </div>
        {/if}
        {#each items as p (p.id)}
          <button
            class="block w-full text-left px-3 py-1.5 text-xs truncate transition-colors {selectedId ===
            p.id
              ? 'bg-immich-dark-primary/20 text-immich-dark-primary'
              : 'hover:bg-white/10'}"
            onclick={() => pick(p.id)}
          >
            {p.name}
          </button>
        {/each}
      {/each}
    {/if}
  </div>
</Popover>
