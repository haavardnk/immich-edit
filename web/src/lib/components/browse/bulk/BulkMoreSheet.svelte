<script lang="ts">
  import MenuRow from '#lib/components/MenuRow.svelte';
  import ActionSheet from '#lib/components/sheet/ActionSheet.svelte';
  import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import { bulkEdits, type BulkEditAction } from '#lib/browse/bulk/bulkEdits.svelte.ts';
  import { multiDetail } from '#lib/browse/bulk/bulkRules.ts';
  import { editableViewedAlbum } from '#lib/browse/viewedAlbum.ts';
  import type { MultiMode } from '#lib/browse/compareEntry.ts';
  import { compare } from '#lib/stores/compare.svelte.ts';
  import {
    mdiCompare,
    mdiContentDuplicate,
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
    onPick: (sheet: 'album' | 'tags' | 'preset') => void;
  } = $props();

  const scope = bulkScope;
  const actions = bulkActions;
  const empty = $derived(scope.count === 0);
  const busy = $derived(empty || actions.metaBusy);
  const shared = $derived(!scope.allOwned);
  const removable = $derived(editableViewedAlbum());

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

{#snippet editRow(action: BulkEditAction)}
  <MenuRow
    icon={action.icon}
    label={action.label}
    aria-label={action.label}
    disabled={action.disabled}
    onclick={action.run}
  >
    {@render detail(action.detail)}
  </MenuRow>
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
    icon={bulkEdits.applyPreset.icon}
    label={bulkEdits.applyPreset.label}
    disabled={bulkEdits.applyPreset.disabled}
    onclick={() => onPick('preset')}
  />
  {@render editRow(bulkEdits.paste)}
  {@render editRow(bulkEdits.pasteOnly)}
  {@render editRow(bulkEdits.reset)}
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
