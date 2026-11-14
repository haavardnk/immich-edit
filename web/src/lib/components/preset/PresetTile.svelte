<script lang="ts">
  import type { Preset } from '#lib/api/presets.ts';
  import MenuItem from '#lib/components/MenuItem.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { clickGuard } from '#lib/gestures/clickGuard.ts';
  import { gestures } from '#lib/gestures/gestures.ts';
  import { longPressRecognizer } from '#lib/gestures/longPress.ts';
  import type { Point } from '#lib/gestures/point.ts';
  import { presetStrip } from '#lib/stores/presetStrip.svelte.ts';
  import { presets } from '#lib/stores/presets.svelte.ts';
  import { Icon, IconButton } from '@immich/ui';
  import {
    mdiCheck,
    mdiClose,
    mdiDeleteOutline,
    mdiDotsHorizontal,
    mdiDownload,
    mdiDownloadMultiple,
    mdiPencilOutline,
    mdiStar,
    mdiStarOutline
  } from '@mdi/js';

  const HOLD_MS = 250;

  let {
    preset,
    src,
    width,
    disabled,
    onedit
  }: {
    preset: Preset;
    src: string | null;
    width: number;
    disabled: boolean;
    onedit: () => void;
  } = $props();

  let menuOpen = $state(false);
  let confirmingDelete = $state(false);
  let failedSrc = $state<string | null>(null);
  let button = $state<HTMLButtonElement>();
  let holding = false;
  const touch = $derived(layout.mode === 'touch');
  const applied = $derived(presetStrip.appliedId === preset.id);
  const guard = clickGuard();

  function release(): void {
    if (!holding) return;
    holding = false;
    presetStrip.endPreview();
  }

  function inside({ x, y }: Point): boolean {
    const box = button?.getBoundingClientRect();
    return !!box && x >= box.left && x <= box.right && y >= box.top && y <= box.bottom;
  }

  const hold = gestures({
    recognizers: [
      longPressRecognizer({
        thresholds: { longPressMs: HOLD_MS },
        enabled: () => touch && !disabled,
        start: () => {
          guard.arm();
          holding = true;
          presetStrip.preview(preset, true);
        },
        move: (point) => {
          if (!inside(point)) release();
        },
        end: release,
        cancel: release
      })
    ],
    touchAction: null
  });

  $effect(() => release);

  function hover(event: PointerEvent): void {
    if (event.pointerType !== 'touch') presetStrip.preview(preset);
  }

  function unhover(event: PointerEvent): void {
    if (event.pointerType !== 'touch') presetStrip.endPreview();
  }

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
  class="group/tile relative flex-none touch:w-22"
  onpointerenter={hover}
  onpointerleave={unhover}
  oncontextmenu={(event) => {
    event.preventDefault();
    if (!touch) setMenu(true);
  }}
>
  <button
    bind:this={button}
    type="button"
    class="no-callout relative flex h-full flex-col rounded-sm outline-none select-none focus-visible:ring-2 focus-visible:ring-primary disabled:opacity-50 touch:h-auto touch:w-full touch:rounded-lg"
    style:width={touch ? null : `${width}px`}
    title="Apply {preset.name}"
    aria-label="Apply {preset.name}"
    aria-current={applied || undefined}
    {disabled}
    onfocus={() => {
      if (!touch) presetStrip.preview(preset);
    }}
    onblur={() => {
      if (!touch) presetStrip.endPreview();
    }}
    onclick={() => presetStrip.apply(preset)}
    {@attach hold}
    {@attach guard.attachment}
  >
    <span
      class="relative flex min-h-0 w-full flex-1 items-center justify-center overflow-hidden rounded-sm border bg-light-100 text-lg text-dark/40 touch:h-22 touch:flex-none touch:rounded-lg {applied
        ? 'border-primary ring-1 ring-primary'
        : 'border-hairline group-hover/tile:border-light-950/40'}"
    >
      {#if src && src !== failedSrc}
        <img
          {src}
          alt=""
          loading="lazy"
          draggable="false"
          class="size-full object-cover"
          onerror={() => (failedSrc = src)}
        />
      {:else}
        <span aria-hidden="true">{preset.name.slice(0, 1).toUpperCase()}</span>
      {/if}
      {#if applied}
        <span
          data-applied-badge
          class="absolute top-1 left-1 grid size-4 place-items-center rounded-full bg-primary text-light touch:size-5"
          aria-hidden="true"
        >
          <Icon icon={mdiCheck} size="75%" />
        </span>
      {/if}
    </span>
    <span
      class="pointer-events-none absolute inset-x-px bottom-px truncate rounded-b-sm bg-linear-to-t from-black/80 to-transparent px-1 pt-3 pb-0.5 text-left text-[11px] text-white/90 touch:static touch:bg-none touch:px-0.5 touch:pt-1 touch:pb-0 touch:text-center touch:text-[12px] touch:text-dark/80"
      >{preset.name}</span
    >
  </button>
  {#if preset.favorite}
    <span
      role="img"
      aria-label="Favourite"
      class="pointer-events-none absolute top-1 right-1 grid size-4 place-items-center rounded-full bg-black/55 text-warning touch:top-16 touch:right-1.5 touch:size-5 {menuOpen
        ? 'opacity-0'
        : 'group-hover/tile:opacity-0 group-has-focus-visible/tile:opacity-0 touch:opacity-100'}"
    >
      <Icon icon={mdiStar} size="75%" />
    </span>
  {/if}
  <Popover
    open={menuOpen}
    anchor="top"
    align="end"
    onOpenChange={setMenu}
    contentClass="w-52 py-1"
    title={preset.name}
  >
    {#snippet trigger(props)}
      <IconButton
        size="tiny"
        variant="ghost"
        color="secondary"
        icon={mdiDotsHorizontal}
        title=""
        aria-label="Actions for {preset.name}"
        class="absolute top-0.5 right-0.5 bg-black/55 touch:top-1 touch:right-1 touch:size-7 touch:rounded-full {menuOpen
          ? ''
          : 'opacity-0 group-hover/tile:opacity-100 group-has-focus-visible/tile:opacity-100 pointer-coarse:opacity-100 touch:opacity-100'}"
        {...props}
      />
    {/snippet}
    <div class="flex flex-col">
      <MenuItem
        icon={preset.favorite ? mdiStar : mdiStarOutline}
        label={preset.favorite ? 'Remove from favourites' : 'Add to favourites'}
        onclick={() => act(() => void presets.setFavorite(preset.id, !preset.favorite))}
      />
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
        <MenuItem
          icon={mdiClose}
          label="Cancel"
          data-menu-stay
          onclick={() => (confirmingDelete = false)}
        />
      {:else}
        <MenuItem
          icon={mdiDeleteOutline}
          label="Delete"
          data-menu-stay
          onclick={() => (confirmingDelete = true)}
        />
      {/if}
    </div>
  </Popover>
</li>
