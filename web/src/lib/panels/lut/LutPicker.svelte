<script lang="ts">
  import Popover from '$lib/components/Popover.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import type { LutMeta } from '$lib/api/luts';
  import { mdiCheck, mdiChevronDown, mdiMagnify } from '@mdi/js';

  let {
    luts,
    selectedId,
    onSelect
  }: {
    luts: LutMeta[];
    selectedId: string | null;
    onSelect: (id: string | null) => void;
  } = $props();

  let open = $state(false);
  let query = $state('');

  const selected = $derived(luts.find((lut) => lut.id === selectedId) ?? null);
  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return luts;
    return luts.filter((lut) => lut.name.toLowerCase().includes(q));
  });

  function pick(id: string | null): void {
    onSelect(id);
    close();
  }

  function close(): void {
    open = false;
    query = '';
  }
</script>

<Popover
  {open}
  onClose={close}
  rootClass="w-full"
  contentClass="w-full max-h-72 overflow-hidden flex flex-col"
>
  {#snippet trigger()}
    <button
      type="button"
      class="flex w-full items-center gap-2 rounded-lg bg-white/5 px-2.5 py-1.5 text-xs transition-colors hover:bg-white/10"
      title={selected?.name ?? 'No LUT'}
      onclick={() => (open = !open)}
    >
      <span class="flex-1 truncate text-left {selected ? '' : 'text-immich-dark-fg/40'}">
        {selected?.name ?? 'No LUT'}
      </span>
      <Icon path={mdiChevronDown} size={14} class="flex-none opacity-50" />
    </button>
  {/snippet}

  <button
    type="button"
    class="flex w-full items-center gap-2 border-b border-white/10 px-3 py-1.5 text-left text-xs transition-colors {selectedId ===
    null
      ? 'bg-immich-dark-primary/20 text-immich-dark-primary'
      : 'hover:bg-white/10'}"
    onclick={() => pick(null)}
  >
    <span class="w-3.5 shrink-0"
      >{#if selectedId === null}<Icon path={mdiCheck} size={14} />{/if}</span
    >
    No LUT
  </button>

  {#if luts.length > 8}
    <div class="border-b border-white/10 p-1.5">
      <div class="flex items-center gap-1.5 rounded-lg bg-white/5 px-2 py-1">
        <Icon path={mdiMagnify} size={14} class="flex-none opacity-40" />
        <input
          class="min-w-0 flex-1 bg-transparent text-xs outline-none placeholder:text-immich-dark-fg/30"
          placeholder="Search LUTs"
          bind:value={query}
        />
      </div>
    </div>
  {/if}

  <div class="overflow-y-auto py-1">
    {#if luts.length === 0}
      <div class="px-3 py-2 text-xs text-immich-dark-fg/30">No imported LUTs.</div>
    {:else if filtered.length === 0}
      <div class="px-3 py-2 text-xs text-immich-dark-fg/30">No matching LUTs.</div>
    {:else}
      {#each filtered as lut (lut.id)}
        <button
          type="button"
          class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs transition-colors {selectedId ===
          lut.id
            ? 'bg-immich-dark-primary/20 text-immich-dark-primary'
            : 'hover:bg-white/10'}"
          title={`${lut.name} · ${lut.lut_size}³`}
          onclick={() => pick(lut.id)}
        >
          <span class="w-3.5 shrink-0"
            >{#if selectedId === lut.id}<Icon path={mdiCheck} size={14} />{/if}</span
          >
          <span class="min-w-0 flex-1 truncate">{lut.name}</span>
          <span class="text-[10px] text-immich-dark-fg/30">{lut.lut_size}³</span>
        </button>
      {/each}
    {/if}
  </div>
</Popover>
