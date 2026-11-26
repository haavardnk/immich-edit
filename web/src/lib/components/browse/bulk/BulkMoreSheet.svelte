<script lang="ts">
  import MenuRow from '#lib/components/MenuRow.svelte';
  import BulkActionRow from '#lib/components/browse/bulk/BulkActionRow.svelte';
  import ActionSheet from '#lib/components/sheet/ActionSheet.svelte';
  import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import { bulkEdits } from '#lib/browse/bulk/bulkEdits.svelte.ts';
  import type { BulkMenuAction } from '#lib/browse/bulk/bulkMenu.ts';
  import { multiDetail } from '#lib/browse/bulk/bulkRules.ts';
  import { editableViewedAlbum } from '#lib/browse/viewedAlbum.ts';
  import type { MultiMode } from '#lib/browse/compareEntry.ts';
  import { compare } from '#lib/stores/compare.svelte.ts';
  import {
    mdiCompare,
    mdiContentDuplicate,
    mdiFileDocumentEditOutline,
    mdiImageAlbum,
    mdiMinusBoxOutline,
    mdiTagOutline,
    mdiTrashCanOutline,
    mdiViewGridOutline
  } from '@mdi/js';

  let {
    open,
    onOpenChange,
    onPick
  }: {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onPick: (sheet: 'album' | 'tags' | 'metadata' | 'preset') => void;
  } = $props();

  const scope = bulkScope;
  const actions = bulkActions;
  const empty = $derived(scope.count === 0);
  const busy = $derived(empty || actions.metaBusy);
  const shared = $derived(!scope.allOwned);
  const removable = $derived(editableViewedAlbum());

  function runAction(action: BulkMenuAction): void {
    action.run();
  }

  function owned(run: () => void): () => void {
    return () => {
      if (actions.ownedOrExplain()) run();
    };
  }
</script>

{#snippet detail(text: string | null)}
  {#if text}
    <span class="shrink-0 text-label text-dark/65">{text}</span>
  {/if}
{/snippet}

{#snippet multiRow(mode: MultiMode)}
  <MenuRow
    icon={mode === 'compare' ? mdiCompare : mdiViewGridOutline}
    label={mode === 'compare' ? 'Compare' : 'Survey'}
    disabled={scope.count < 2}
    onclick={() => actions.multi(mode)}
  >
    {@render detail(multiDetail(mode, scope.count, compare.paneLimit))}
  </MenuRow>
{/snippet}

<ActionSheet {open} {onOpenChange} title="More actions">
  <MenuRow
    icon={mdiImageAlbum}
    label="Albums"
    class="wide:hidden"
    disabled={busy}
    onclick={() => onPick('album')}
  />
  <MenuRow
    icon={mdiTagOutline}
    label="Tags"
    class="wide:hidden {shared ? 'opacity-50' : ''}"
    aria-disabled={shared || undefined}
    disabled={busy}
    onclick={owned(() => onPick('tags'))}
  />
  <MenuRow
    icon={mdiFileDocumentEditOutline}
    label="Metadata"
    class={shared ? 'opacity-50' : ''}
    aria-disabled={shared || undefined}
    disabled={busy}
    onclick={owned(() => onPick('metadata'))}
  />
  <MenuRow
    icon={bulkEdits.applyPreset.icon}
    label={bulkEdits.applyPreset.label}
    disabled={bulkEdits.applyPreset.disabled}
    onclick={() => onPick('preset')}
  />
  <BulkActionRow action={bulkEdits.paste} onPick={runAction} />
  <BulkActionRow action={bulkEdits.pasteOnly} onPick={runAction} />
  <BulkActionRow action={bulkEdits.reset} onPick={runAction} />
  {@render multiRow('compare')}
  {#if compare.surveyAvailable}
    {@render multiRow('survey')}
  {/if}
  <MenuRow
    icon={mdiContentDuplicate}
    label="Create virtual copy"
    disabled={busy}
    onclick={() => void actions.createCopies()}
  />
  {#if removable}
    <MenuRow
      icon={mdiMinusBoxOutline}
      label="Remove from album"
      disabled={busy}
      onclick={() => void actions.removeFromAlbum(removable)}
    />
  {/if}
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
