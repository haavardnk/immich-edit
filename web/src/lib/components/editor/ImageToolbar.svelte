<script lang="ts">
  import { page } from '$app/state';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import ExifSummary from './ExifSummary.svelte';
  import SoftProofControl from './SoftProofControl.svelte';
  import VersionsList from './VersionsList.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import MenuItem from '#lib/components/MenuItem.svelte';
  import { hint, keysFor } from '#lib/shortcuts/labels.ts';
  import { copyIndex, isCopy } from '#lib/browse/assetKey.ts';
  import { backToGrid } from '#lib/browse/backToGrid.ts';
  import { trashLabel } from '#lib/browse/trash.ts';
  import { trash } from '#lib/stores/trash.svelte.ts';
  import { IconButton } from '@immich/ui';
  import {
    mdiArrowLeft,
    mdiUndo,
    mdiRedo,
    mdiEyeOutline,
    mdiCompare,
    mdiTriangleOutline,
    mdiDotsVertical,
    mdiKeyboardOutline,
    mdiCircleHalfFull,
    mdiFullscreen,
    mdiInformationOutline,
    mdiTrashCanOutline
  } from '@mdi/js';

  let moreOpen = $state(false);
  let moreAnchor = $state<HTMLElement | null>(null);

  const assetId = $derived(editor.assetId);
  const copyBadge = $derived(
    assetId && isCopy(assetId) ? (editor.asset?.copyLabel ?? `Copy ${copyIndex(assetId)}`) : null
  );

  function goBack(): void {
    void backToGrid(assetId, page.url.searchParams.get('from'));
  }

  async function moveToTrash(): Promise<void> {
    const asset = editor.asset;
    if (!asset) return;
    if (await trash.request([asset])) goBack();
  }

  function holdOriginal(down: boolean): void {
    editor.showingOriginal = down;
    editor.onLive();
  }
</script>

{#snippet historyControls()}
  <IconButton
    size="small"
    variant="ghost"
    color="secondary"
    icon={mdiUndo}
    title={hint('Undo', 'undo')}
    aria-label={hint('Undo', 'undo')}
    disabled={!editor.canUndo}
    onclick={editor.undo}
  />
  <IconButton
    size="small"
    variant="ghost"
    color="secondary"
    icon={mdiRedo}
    title={hint('Redo', 'redo')}
    aria-label={hint('Redo', 'redo')}
    disabled={!editor.canRedo}
    onclick={editor.redo}
  />
{/snippet}

<nav
  aria-label="Editor toolbar"
  class="relative z-40 grid h-12 shrink-0 grid-cols-[minmax(0,1fr)_auto] items-center border-b border-hairline bg-editor-chrome px-2 xl:grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] {ui.rightCollapsed
    ? 'md:w-[calc(100%+3.5rem)]'
    : ''}"
>
  <div class="flex min-w-0 items-center gap-2 overflow-hidden">
    <IconButton
      class="shrink-0"
      size="small"
      variant="ghost"
      color="secondary"
      shape="round"
      icon={mdiArrowLeft}
      title={hint('Back', 'backToGrid')}
      aria-label={hint('Back', 'backToGrid')}
      onclick={goBack}
    />
    {#if editor.asset}
      <div class="min-w-0 border-l border-hairline pl-2">
        <div class="flex min-w-0 items-center gap-2">
          <span class="truncate whitespace-nowrap text-xs font-semibold text-white/90">
            {editor.asset.originalFileName}
          </span>
          {#if copyBadge}
            <span
              class="max-w-32 shrink-0 truncate rounded bg-hairline px-1.5 py-0.5 text-[9px] font-medium text-white/60"
              >{copyBadge}</span
            >
          {/if}
        </div>
      </div>
    {/if}
  </div>

  <div class="hidden items-center gap-0.5 xl:flex">
    {@render historyControls()}
  </div>

  <div class="flex min-w-0 items-center justify-end gap-0.5">
    <div class="flex shrink-0 items-center gap-0.5 xl:hidden">
      {@render historyControls()}
    </div>
    <div class="hidden shrink-0 items-center gap-0.5 xl:flex">
      <IconButton
        size="small"
        variant="ghost"
        color={editor.showingOriginal ? 'primary' : 'secondary'}
        icon={mdiEyeOutline}
        title={hint('Hold for original', 'holdOriginal')}
        aria-label={hint('Hold for original', 'holdOriginal')}
        onpointerdown={() => holdOriginal(true)}
        onpointerup={() => holdOriginal(false)}
        onpointercancel={() => holdOriginal(false)}
        onpointerleave={() => {
          if (editor.showingOriginal) holdOriginal(false);
        }}
      />
    </div>

    {#if editor.assetId}
      <ExifSummary anchor={moreAnchor} />
    {/if}
    <div class="flex shrink-0" bind:this={moreAnchor}>
      <Popover
        open={moreOpen}
        anchor="bottom"
        align="end"
        onOpenChange={(open) => (moreOpen = open)}
        contentClass="w-64 py-1"
        tooltip="More editor actions"
      >
        {#snippet trigger(props)}
          <IconButton
            size="small"
            variant="ghost"
            color={editor.isProofing || ui.clipWarn || editor.splitMode ? 'primary' : 'secondary'}
            icon={mdiDotsVertical}
            title=""
            aria-label="More editor actions"
            {...props}
          />
        {/snippet}
        <div class="flex flex-col">
          <VersionsList onnavigate={() => (moreOpen = false)} />
          <div class="my-1 border-t border-dark/10"></div>
          <span class="px-3 pt-1.5 pb-1 text-[10px] tracking-wider text-dark/65 uppercase"
            >View</span
          >
          <MenuItem
            icon={mdiInformationOutline}
            label="Info"
            keys={keysFor('toggleInfo')}
            pressed={ui.metaPopover === 'exif'}
            disabled={!editor.asset?.exifInfo}
            onclick={() => {
              moreOpen = false;
              ui.togglePopover('exif');
            }}
          />
          <MenuItem
            icon={mdiFullscreen}
            label="Fullscreen"
            keys={keysFor('fullscreen')}
            onclick={() => {
              moreOpen = false;
              ui.toggleFullscreen();
            }}
          />
          <div class="contents xl:hidden">
            <MenuItem
              icon={mdiEyeOutline}
              label="View original"
              keys={keysFor('holdOriginal')}
              onpointerdown={() => holdOriginal(true)}
              onpointerup={() => holdOriginal(false)}
              onpointercancel={() => holdOriginal(false)}
              onpointerleave={() => {
                if (editor.showingOriginal) holdOriginal(false);
              }}
            />
          </div>
          <MenuItem
            icon={mdiCompare}
            label="Before/After split"
            keys={keysFor('beforeAfter')}
            pressed={editor.splitMode}
            disabled={!!editor.geometrySession}
            onclick={() => {
              editor.toggleSplit();
              moreOpen = false;
            }}
          />
          <MenuItem
            icon={mdiTriangleOutline}
            label="Clipping overlay"
            keys={keysFor('clipWarn')}
            pressed={ui.clipWarn}
            onclick={() => {
              editor.toggleClipWarn();
              moreOpen = false;
            }}
          />
          <MenuItem
            icon={mdiCircleHalfFull}
            label="Grey background"
            pressed={ui.greyCanvas}
            onclick={() => {
              ui.toggleGreyCanvas();
              moreOpen = false;
            }}
          />
          <div class="my-1 border-t border-dark/10"></div>
          <SoftProofControl />
          <div class="my-1 border-t border-dark/10"></div>
          <MenuItem
            icon={mdiTrashCanOutline}
            label={trashLabel(copyBadge !== null)}
            disabled={!editor.asset}
            onclick={() => {
              moreOpen = false;
              void moveToTrash();
            }}
          />
          <div class="my-1 border-t border-dark/10"></div>
          <MenuItem
            icon={mdiKeyboardOutline}
            label="Keyboard shortcuts"
            keys={keysFor('help')}
            onclick={() => {
              moreOpen = false;
              ui.toggleKeybindsHelp();
            }}
          />
        </div>
      </Popover>
    </div>
  </div>
</nav>
