<script lang="ts">
  import { page } from '$app/state';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import ExifSummary from './ExifSummary.svelte';
  import SoftProofControl from './SoftProofControl.svelte';
  import Popover from '$lib/components/Popover.svelte';
  import { hint } from '$lib/keybinds';
  import { copyIndex, isCopy } from '$lib/assetKey';
  import { backToGrid } from '$lib/backToGrid';
  import { createVirtualCopy } from '$lib/copies';
  import { Button, IconButton } from '@immich/ui';
  import {
    mdiArrowLeft,
    mdiUndo,
    mdiRedo,
    mdiEyeOutline,
    mdiCompare,
    mdiContentDuplicate,
    mdiTriangleOutline,
    mdiDotsVertical,
    mdiKeyboardOutline
  } from '@mdi/js';

  let moreOpen = $state(false);

  const assetId = $derived(editor.assetId);
  const copyBadge = $derived(
    assetId && isCopy(assetId) ? (editor.asset?.copyLabel ?? `Copy ${copyIndex(assetId)}`) : null
  );

  function goBack(): void {
    void backToGrid(assetId, page.url.searchParams.get('from'));
  }

  function holdOriginal(down: boolean): void {
    editor.showingOriginal = down;
    if (down) {
      editor.showOriginal();
    } else {
      editor.onLive();
    }
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
      <IconButton
        size="small"
        variant="ghost"
        color={editor.splitMode ? 'primary' : 'secondary'}
        icon={mdiCompare}
        title={hint('Before / after split', 'beforeAfter')}
        aria-label={hint('Before / after split', 'beforeAfter')}
        aria-pressed={editor.splitMode}
        disabled={!!editor.geometrySession}
        onclick={editor.toggleSplit}
      />
    </div>

    <div class="mx-1 h-5 w-px shrink-0 bg-hairline"></div>

    {#if editor.assetId}
      <ExifSummary />
    {/if}
    <IconButton
      class="shrink-0"
      size="small"
      variant="ghost"
      color="secondary"
      icon={mdiKeyboardOutline}
      title={hint('Keyboard shortcuts', 'help')}
      aria-label={hint('Keyboard shortcuts', 'help')}
      onclick={ui.toggleKeybindsHelp}
    />
    <Popover
      open={moreOpen}
      anchor="bottom"
      align="end"
      onOpenChange={(open) => (moreOpen = open)}
      contentClass="w-60 p-2"
    >
      {#snippet trigger(props)}
        <IconButton
          size="small"
          variant="ghost"
          color={editor.isProofing || ui.clipWarn ? 'primary' : 'secondary'}
          icon={mdiDotsVertical}
          title="More editor actions"
          aria-label="More editor actions"
          {...props}
        />
      {/snippet}
      <div class="flex flex-col gap-1">
        <Button
          size="tiny"
          variant="ghost"
          color="secondary"
          class="w-full justify-start"
          leadingIcon={mdiContentDuplicate}
          disabled={!assetId}
          onclick={() => {
            if (assetId)
              void createVirtualCopy(assetId, {
                returnPath: page.url.searchParams.get('from')
              });
            moreOpen = false;
          }}>Create virtual copy</Button
        >
        <div class="contents xl:hidden">
          <Button
            size="tiny"
            variant="ghost"
            color="secondary"
            class="w-full justify-start"
            leadingIcon={mdiEyeOutline}
            onpointerdown={() => holdOriginal(true)}
            onpointerup={() => holdOriginal(false)}
            onpointerleave={() => {
              if (editor.showingOriginal) holdOriginal(false);
            }}>View original</Button
          >
          <Button
            size="tiny"
            variant="ghost"
            color={editor.splitMode ? 'primary' : 'secondary'}
            class="w-full justify-start"
            leadingIcon={mdiCompare}
            aria-pressed={editor.splitMode}
            disabled={!!editor.geometrySession}
            onclick={() => {
              editor.toggleSplit();
              moreOpen = false;
            }}>Before/After split</Button
          >
        </div>
        <Button
          size="tiny"
          variant="ghost"
          color={ui.clipWarn ? 'primary' : 'secondary'}
          class="w-full justify-start"
          leadingIcon={mdiTriangleOutline}
          aria-pressed={ui.clipWarn}
          onclick={() => {
            editor.toggleClipWarn();
            moreOpen = false;
          }}>Clipping overlay</Button
        >
        <SoftProofControl />
      </div>
    </Popover>
  </div>
</nav>
