<script lang="ts">
  import MenuRow from '#lib/components/MenuRow.svelte';
  import ActionSheet from '#lib/components/sheet/ActionSheet.svelte';
  import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import { fileStem, multiDetail, TOUCH_PANES } from '#lib/browse/bulk/bulkRules.ts';
  import { favoriteMark } from '#lib/browse/bulk/bulkMarks.ts';
  import type { MultiMode } from '#lib/browse/compareEntry.ts';
  import { clipboard, NOTHING_COPIED } from '#lib/stores/clipboard.svelte.ts';
  import {
    mdiCompare,
    mdiContentDuplicate,
    mdiContentPaste,
    mdiRestore,
    mdiTagOutline,
    mdiTrashCanOutline,
    mdiViewGridOutline,
    mdiViewGridPlusOutline
  } from '@mdi/js';

  let {
    open,
    onOpenChange,
    onPick
  }: {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onPick: (sheet: 'tags' | 'preset') => void;
  } = $props();

  const scope = bulkScope;
  const actions = bulkActions;
  const empty = $derived(scope.count === 0);
  const busy = $derived(empty || actions.metaBusy);
  const shared = $derived(!scope.allOwned);
  const favorite = $derived(favoriteMark(scope.favorite));
  const pasteDetail = $derived(
    clipboard.has
      ? clipboard.source
        ? `from ${fileStem(clipboard.source)}`
        : null
      : NOTHING_COPIED
  );

  function owned(run: () => void): () => void {
    return () => {
      if (actions.ownedOrExplain()) run();
    };
  }
</script>

{#snippet detail(text: string | null)}
  {#if text}
    <span class="shrink-0 text-[13px] text-dark/65">{text}</span>
  {/if}
{/snippet}

{#snippet multiRow(mode: MultiMode)}
  <MenuRow
    icon={mode === 'compare' ? mdiCompare : mdiViewGridOutline}
    label={mode === 'compare' ? 'Compare' : 'Survey'}
    disabled={scope.count < 2}
    onclick={() => actions.multi(mode, TOUCH_PANES)}
  >
    {@render detail(multiDetail(mode, scope.count, TOUCH_PANES))}
  </MenuRow>
{/snippet}

<ActionSheet {open} {onOpenChange} title="More actions">
  <MenuRow
    icon={mdiTagOutline}
    label="Tags"
    class="wide:hidden {shared ? 'opacity-50' : ''}"
    aria-disabled={shared || undefined}
    disabled={busy}
    onclick={owned(() => onPick('tags'))}
  />
  <MenuRow
    icon={mdiViewGridPlusOutline}
    label="Apply preset"
    disabled={busy}
    onclick={() => onPick('preset')}
  />
  <MenuRow
    icon={mdiContentPaste}
    label="Paste edits"
    disabled={busy || !clipboard.has}
    onclick={() => void actions.paste()}
  >
    {@render detail(pasteDetail)}
  </MenuRow>
  <MenuRow
    icon={mdiRestore}
    label="Reset edits"
    disabled={busy}
    onclick={() => void actions.resetEdits()}
  />
  {@render multiRow('compare')}
  {@render multiRow('survey')}
  <MenuRow
    icon={favorite.icon}
    label={favorite.label}
    class={shared ? 'opacity-50' : ''}
    aria-disabled={shared || undefined}
    disabled={busy}
    onclick={owned(() => void actions.setFavorite(scope.favorite !== 'all'))}
  />
  <MenuRow
    icon={mdiContentDuplicate}
    label="Create virtual copy"
    disabled={busy}
    onclick={() => void actions.createCopies()}
  />
  <MenuRow
    icon={mdiTrashCanOutline}
    label="Move to trash"
    class="text-red-400 {scope.trashReason ? 'opacity-50' : ''}"
    aria-disabled={scope.trashReason !== null || undefined}
    disabled={busy}
    onclick={actions.trash}
  >
    {@render detail(scope.trashReason && !empty ? 'Owner only' : null)}
  </MenuRow>
</ActionSheet>
