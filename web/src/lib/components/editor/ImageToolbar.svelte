<script lang="ts">
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import ExifSummary from './ExifSummary.svelte';
  import EditorMoreMenu from './EditorMoreMenu.svelte';
  import CommandIconButton from '#lib/components/CommandIconButton.svelte';
  import { copyIndex, isCopy } from '#lib/browse/assetKey.ts';
  import { redo, undo } from '#lib/commands/editor/edits.ts';
  import { editorBack } from '#lib/commands/editor/navigation.ts';
  import { beforeAfter, holdOriginal } from '#lib/commands/editor/view.ts';

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
      <CommandIconButton command={beforeAfter} size="small" variant="ghost" />
    </div>

    {#if editor.assetId}
      <ExifSummary anchor={moreAnchor} />
    {/if}
    <div class="flex shrink-0" bind:this={moreAnchor}>
      <EditorMoreMenu />
    </div>
  </div>
</nav>
