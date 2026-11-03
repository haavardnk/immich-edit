<script lang="ts">
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import ExifSummary from './ExifSummary.svelte';
  import SoftProofControl from './SoftProofControl.svelte';
  import VersionsList from './VersionsList.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import CommandList from '#lib/components/CommandList.svelte';
  import CommandIconButton from '#lib/components/CommandIconButton.svelte';
  import { copyIndex, isCopy } from '#lib/browse/assetKey.ts';
  import { redo, undo } from '#lib/commands/editor/edits.ts';
  import { editorBack } from '#lib/commands/editor/navigation.ts';
  import { holdOriginal } from '#lib/commands/editor/view.ts';
  import { EDITOR_MORE_MENU, EDITOR_VIEW_MENU } from '#lib/commands/menus.ts';
  import { IconButton } from '@immich/ui';
  import { mdiDotsVertical } from '@mdi/js';

  let moreOpen = $state(false);
  let moreAnchor = $state<HTMLElement | null>(null);

  const assetId = $derived(editor.assetId);
  const copyBadge = $derived(
    assetId && isCopy(assetId) ? (editor.asset?.copyLabel ?? `Copy ${copyIndex(assetId)}`) : null
  );
</script>

{#snippet historyControls()}
  <CommandIconButton command={undo} size="small" variant="ghost" />
  <CommandIconButton command={redo} size="small" variant="ghost" />
{/snippet}

<nav
  aria-label="Editor toolbar"
  class="relative z-40 grid h-12 shrink-0 grid-cols-[minmax(0,1fr)_auto] items-center border-b border-hairline bg-editor-chrome px-2 xl:grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] {ui.rightCollapsed
    ? 'md:w-[calc(100%+3.5rem)]'
    : ''}"
>
  <div class="flex min-w-0 items-center gap-2 overflow-hidden">
    <CommandIconButton
      command={editorBack}
      class="shrink-0"
      size="small"
      variant="ghost"
      shape="round"
    />
    {#if editor.asset}
      <div class="min-w-0 border-l border-hairline pl-2">
        <div class="flex min-w-0 items-center gap-2">
          <span class="truncate whitespace-nowrap text-xs font-semibold text-light-950/90">
            {editor.asset.originalFileName}
          </span>
          {#if copyBadge}
            <span
              class="max-w-32 shrink-0 truncate rounded bg-hairline px-1.5 py-0.5 text-[9px] font-medium text-light-950/60"
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
      <CommandIconButton command={holdOriginal} size="small" variant="ghost" />
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
          <CommandList
            menu={EDITOR_VIEW_MENU}
            onrun={() => (moreOpen = false)}
            rowClass={(command) => (command === holdOriginal ? 'xl:hidden' : undefined)}
          />
          <div class="my-1 border-t border-dark/10"></div>
          <SoftProofControl />
          <div class="my-1 border-t border-dark/10"></div>
          <CommandList menu={EDITOR_MORE_MENU} onrun={() => (moreOpen = false)} />
        </div>
      </Popover>
    </div>
  </div>
</nav>
