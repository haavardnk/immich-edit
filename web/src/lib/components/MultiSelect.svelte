<script lang="ts" generics="T">
  import Icon from '$lib/components/Icon.svelte';
  import { mdiClose } from '@mdi/js';

  let {
    options,
    selected = $bindable(),
    getId,
    getLabel,
    placeholder = 'Search…'
  }: {
    options: T[];
    selected: string[];
    getId: (item: T) => string;
    getLabel: (item: T) => string;
    placeholder?: string;
  } = $props();

  let input = $state('');
  let open = $state(false);

  const selectedSet = $derived(new Set(selected));
  const selectedItems = $derived(options.filter((o) => selectedSet.has(getId(o))));
  const suggestions = $derived.by(() => {
    const q = input.trim().toLowerCase();
    return options
      .filter((o) => !selectedSet.has(getId(o)))
      .filter((o) => (q ? getLabel(o).toLowerCase().includes(q) : true));
  });

  function pick(id: string): void {
    if (!selected.includes(id)) selected = [...selected, id];
    input = '';
    open = false;
  }

  function remove(id: string): void {
    selected = selected.filter((s) => s !== id);
  }

  function onKey(e: KeyboardEvent): void {
    if (e.key === 'Enter' && suggestions.length > 0) {
      e.preventDefault();
      pick(getId(suggestions[0]));
    } else if (e.key === 'Escape') {
      input = '';
      open = false;
    }
  }
</script>

<div class="flex flex-col gap-1">
  {#if selectedItems.length > 0}
    <div class="flex flex-wrap gap-1">
      {#each selectedItems as item (getId(item))}
        <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] bg-white/10">
          {getLabel(item)}
          <button class="hover:text-red-400" title="Remove" onclick={() => remove(getId(item))}>
            <Icon path={mdiClose} size={12} />
          </button>
        </span>
      {/each}
    </div>
  {/if}
  <div class="relative">
    <input
      type="text"
      {placeholder}
      class="input w-full bg-immich-dark-bg/40 border-immich-dark-fg/10 text-xs h-auto py-2.5 min-h-0"
      bind:value={input}
      onfocus={() => (open = true)}
      onblur={() => setTimeout(() => (open = false), 150)}
      onkeydown={onKey}
    />
    {#if open && suggestions.length > 0}
      <div
        class="absolute z-20 left-0 right-0 mt-1 bg-immich-dark-bg border border-immich-dark-fg/10 rounded-md shadow-lg max-h-64 overflow-y-auto"
      >
        {#each suggestions as s (getId(s))}
          <button
            type="button"
            class="block w-full text-left px-2 py-1.5 text-xs hover:bg-white/10"
            onmousedown={() => pick(getId(s))}
          >
            {getLabel(s)}
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>
