<script lang="ts">
  import { page } from '$app/state';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { editorHref } from '#lib/editor/navigation.ts';
  import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
  import { BAR_CONTAINER, NAME_WIDTH, ONE_ROW } from '#lib/browse/bulk/bulkBarFit.ts';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import { multiDetail } from '#lib/browse/bulk/bulkRules.ts';
  import { NOT_OWNER_TRASH } from '#lib/browse/bulk/bulkMarks.ts';
  import DisabledReason from '#lib/components/DisabledReason.svelte';
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import BulkActionsDialog from './BulkActionsDialog.svelte';
  import BulkMarkTools from './BulkMarkTools.svelte';
  import BulkOrganise from './BulkOrganise.svelte';
  import BulkTagBand from './BulkTagBand.svelte';
  import BulkAlbumBand from './BulkAlbumBand.svelte';
  import { Button, IconButton } from '@immich/ui';
  import type { MultiMode } from '#lib/browse/compareEntry.ts';
  import { compare } from '#lib/stores/compare.svelte.ts';
  import { mergeProps } from 'bits-ui';
  import {
    mdiClose,
    mdiCompare,
    mdiImageEditOutline,
    mdiTrashCanOutline,
    mdiViewGridOutline
  } from '@mdi/js';

  let band = $state<'tags' | 'albums' | null>(null);
  let bulkActionsOpen = $state(false);

  const scope = bulkScope;
  const actions = bulkActions;
  const photos = $derived(scope.targetCount === 1 ? 'this photo' : 'selected photos');

  function multiTitle(mode: MultiMode): string {
    const detail = multiDetail(mode, scope.count, compare.paneLimit);
    const label = mode === 'compare' ? 'Compare' : 'Survey';
    return hint(
      detail ? `${label} the ${detail}` : `${label} selected`,
      mode === 'compare' ? 'enterCompare' : 'enterSurvey'
    );
  }

  async function selectAll(): Promise<void> {
    band = null;
    bulkActionsOpen = false;
    await actions.selectAll();
  }
</script>

{#snippet divider()}
  <span class="mx-1 h-5 w-px shrink-0 bg-dark/10 {ONE_ROW.divider}" aria-hidden="true"></span>
{/snippet}

{#snippet multiButton(mode: MultiMode)}
  <IconButton
    size="medium"
    variant="ghost"
    color="secondary"
    class="shrink-0"
    icon={mode === 'compare' ? mdiCompare : mdiViewGridOutline}
    title={multiTitle(mode)}
    aria-label={mode === 'compare' ? 'Compare selected' : 'Survey selected'}
    disabled={scope.selectingAll}
    onclick={() => actions.multi(mode)}
  />
{/snippet}

<div
  class="pointer-events-none absolute inset-x-0 bottom-[calc(1rem+var(--chrome-inset,0px))] z-40 flex justify-center px-4 {BAR_CONTAINER}"
>
  <div
    {@attach actions.barRef}
    role="toolbar"
    aria-label="Selection actions"
    class="pointer-events-auto flex max-w-full flex-col overflow-hidden rounded-xl bg-light-100 shadow-2xl ring-1 ring-hairline"
  >
    <div class="items-center gap-1 py-1.5 {ONE_ROW.row}">
      <div class="flex min-w-0 items-center gap-1 {ONE_ROW.head}">
        <div class="flex shrink-0 flex-col justify-center pe-1 leading-tight" aria-live="polite">
          {#if scope.single}
            <span class="sr-only">1 selected</span>
            <TruncationTooltip text={scope.single.originalFileName}>
              {#snippet child({ props })}
                <span
                  {...mergeProps(props, { tabindex: -1 })}
                  data-truncate
                  aria-hidden="true"
                  class="block truncate text-sm font-semibold {NAME_WIDTH}"
                >
                  {scope.single?.originalFileName}
                </span>
              {/snippet}
            </TruncationTooltip>
          {:else}
            <span class="text-sm font-semibold whitespace-nowrap">{scope.targetCount} selected</span
            >
          {/if}
          {#if scope.scope}
            <span class="text-[11px] whitespace-nowrap text-dark/65">{scope.scope}</span>
          {/if}
        </div>
        {#if scope.canSelectAll}
          <Button
            size="tiny"
            variant="ghost"
            color="primary"
            class="shrink-0"
            title={hint('Load and select all photos', 'gridSelectAll')}
            loading={scope.selectingAll}
            disabled={actions.busy || scope.loadingMore}
            onclick={selectAll}
          >
            Select all
          </Button>
        {/if}
      </div>
      <div class="flex min-w-0 items-center justify-center {ONE_ROW.tools}">
        {@render divider()}
        {#if scope.single}
          <IconButton
            size="medium"
            variant="ghost"
            color="secondary"
            class="shrink-0"
            icon={mdiImageEditOutline}
            title={hint('Open in editor', 'openEditor')}
            aria-label="Open in editor"
            href={editorHref(scope.single.id, `${page.url.pathname}${page.url.search}`)}
          />
        {:else}
          {@render multiButton('compare')}
          {@render multiButton('survey')}
        {/if}
        {@render divider()}
        <BulkMarkTools />
        {@render divider()}
        <BulkOrganise
          {band}
          onBand={(next) => (band = next)}
          onEditExport={() => (bulkActionsOpen = true)}
        />
        {@render divider()}
        <DisabledReason locked={!scope.trashable} reason={NOT_OWNER_TRASH} class="flex shrink-0">
          <IconButton
            size="medium"
            variant="ghost"
            color="secondary"
            icon={mdiTrashCanOutline}
            title={hint(`Move ${photos} to the Immich trash`, 'trash')}
            aria-label="Move to Immich trash"
            disabled={actions.metaBusy || !scope.trashable}
            onclick={actions.trash}
          />
        </DisabledReason>
      </div>
      <IconButton
        size="medium"
        variant="ghost"
        color="secondary"
        class="shrink-0 {ONE_ROW.close}"
        icon={mdiClose}
        title={hint(scope.closeLabel, 'gridClearSelection')}
        aria-label={scope.closeLabel}
        disabled={scope.selectingAll}
        onclick={actions.clear}
      />
    </div>

    {#if band === 'tags' && scope.allOwned}
      <BulkTagBand />
    {:else if band === 'albums'}
      <BulkAlbumBand />
    {/if}
  </div>
</div>

{#if bulkActionsOpen}
  <BulkActionsDialog onClose={() => (bulkActionsOpen = false)} />
{/if}
