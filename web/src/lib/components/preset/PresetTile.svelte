<script lang="ts">
  import MenuItem from '$lib/components/MenuItem.svelte';
  import Popover from '$lib/components/Popover.svelte';
  import type { Preset } from '$lib/api/presets';
  import { presets } from '$lib/stores/presets.svelte';
  import { IconButton } from '@immich/ui';
  import {
    mdiCheck,
    mdiClose,
    mdiDeleteOutline,
    mdiDotsHorizontal,
    mdiDownload,
    mdiDownloadMultiple,
    mdiPencilOutline
  } from '@mdi/js';

  let {
    preset,
    src,
    width,
    disabled,
    onpreview,
    onendpreview,
    onapply,
    onedit
  }: {
    preset: Preset;
    src: string | null;
    width: number;
    disabled: boolean;
    onpreview: () => void;
    onendpreview: () => void;
    onapply: () => void;
    onedit: () => void;
  } = $props();

  let menuOpen = $state(false);
  let confirmingDelete = $state(false);
  let failedSrc = $state<string | null>(null);

  function setMenu(open: boolean): void {
    menuOpen = open;
    confirmingDelete = false;
  }

  function act(action: () => void): void {
    setMenu(false);
    action();
  }
</script>

<li
  class="group/tile relative flex-none"
  onpointerenter={onpreview}
  onpointerleave={onendpreview}
  oncontextmenu={(event) => {
    event.preventDefault();
    setMenu(true);
  }}
>
  <button
    type="button"
    class="relative flex h-full items-center justify-center overflow-hidden rounded-sm border border-hairline bg-neutral-900 text-lg text-dark/40 outline-none group-hover/tile:border-white/40 focus-visible:ring-2 focus-visible:ring-primary disabled:opacity-50"
    style:width="{width}px"
    title="Apply {preset.name}"
    aria-label="Apply {preset.name}"
    {disabled}
    onfocus={onpreview}
    onblur={onendpreview}
    onclick={onapply}
  >
    {#if src && src !== failedSrc}
      <img
        {src}
        alt=""
        loading="lazy"
        class="size-full object-cover"
        onerror={() => (failedSrc = src)}
      />
    {:else}
      <span aria-hidden="true">{preset.name.slice(0, 1).toUpperCase()}</span>
    {/if}
    <span
      class="pointer-events-none absolute inset-x-0 bottom-0 truncate bg-linear-to-t from-black/80 to-transparent px-1 pt-3 pb-0.5 text-left text-[11px] text-white/90"
      >{preset.name}</span
    >
  </button>
  <Popover open={menuOpen} anchor="top" align="end" onOpenChange={setMenu} contentClass="w-52 py-1">
    {#snippet trigger(props)}
      <IconButton
        size="tiny"
        variant="ghost"
        color="secondary"
        icon={mdiDotsHorizontal}
        title=""
        aria-label="Actions for {preset.name}"
        class="absolute top-0.5 right-0.5 bg-black/55 {menuOpen
          ? ''
          : 'opacity-0 group-hover/tile:opacity-100 group-has-focus-visible/tile:opacity-100 pointer-coarse:opacity-100'}"
        {...props}
      />
    {/snippet}
    <div class="flex flex-col">
      <MenuItem icon={mdiPencilOutline} label="Edit…" onclick={() => act(onedit)} />
      <MenuItem
        icon={mdiDownload}
        label="Export"
        onclick={() => act(() => presets.exportOne(preset))}
      />
      <MenuItem
        icon={mdiDownloadMultiple}
        label="Export all presets"
        onclick={() => act(presets.exportAll)}
      />
      <div class="my-1 border-t border-dark/10"></div>
      {#if confirmingDelete}
        <MenuItem
          icon={mdiCheck}
          label="Confirm delete"
          onclick={() => act(() => void presets.remove(preset.id))}
        />
        <MenuItem icon={mdiClose} label="Cancel" onclick={() => (confirmingDelete = false)} />
      {:else}
        <MenuItem
          icon={mdiDeleteOutline}
          label="Delete"
          onclick={() => (confirmingDelete = true)}
        />
      {/if}
    </div>
  </Popover>
</li>
