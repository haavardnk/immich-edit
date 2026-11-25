<script lang="ts">
  import LabelSwatches from '#lib/components/LabelSwatches.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import RatingSwatches from '#lib/components/RatingSwatches.svelte';
  import BarAction from '#lib/components/browse/BarAction.svelte';
  import BulkAlbumBand from '#lib/components/browse/bulk/BulkAlbumBand.svelte';
  import BulkApplyPreset from '#lib/components/browse/bulk/BulkApplyPreset.svelte';
  import BulkLocationRows from '#lib/components/browse/bulk/BulkLocationRows.svelte';
  import BulkMoreSheet from '#lib/components/browse/bulk/BulkMoreSheet.svelte';
  import BulkTagBand from '#lib/components/browse/bulk/BulkTagBand.svelte';
  import TouchBulkExportSheet from '#lib/components/browse/bulk/TouchBulkExportSheet.svelte';
  import ActionSheet from '#lib/components/sheet/ActionSheet.svelte';
  import { presence } from '#lib/components/sheet/presence.svelte.ts';
  import { TAB_BAR_BOTTOM } from '#lib/components/shell/touch/touchChrome.ts';
  import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import { favoriteMark, labelLook, ratingLook, rejectMark } from '#lib/browse/bulk/bulkMarks.ts';
  import type { LabelColor } from '#lib/stores/labels.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { mdiDotsHorizontal, mdiExportVariant, mdiImageAlbum, mdiTagOutline } from '@mdi/js';

  type Sheet = 'rate' | 'label' | 'album' | 'tags' | 'location' | 'export' | 'preset';

  const OWNER_ONLY: readonly Sheet[] = ['rate', 'label', 'tags', 'location'];

  let sheet = $state<Sheet | null>(null);
  let more = $state(false);
  let next: Sheet | null = null;

  const scope = bulkScope;
  const actions = bulkActions;
  const empty = $derived(scope.count === 0);
  const busy = $derived(empty || actions.metaBusy);
  const shared = $derived(!scope.allOwned);
  const reject = $derived(rejectMark(scope.rejected));
  const favorite = $derived(favoriteMark(scope.favorite));
  const rating = $derived(ratingLook(scope.rating));
  const labelDot = $derived(labelLook(scope.label));
  const photos = $derived(countOf(scope.targetCount, 'photo'));
  const exportSheet = presence(() => sheet === 'export');

  function opener(name: Sheet): (open: boolean) => void {
    return (open) => {
      if (open && OWNER_ONLY.includes(name) && !actions.ownedOrExplain()) return;
      if (open) sheet = name;
      else if (sheet === name) sheet = null;
    };
  }

  function toggleReject(): void {
    if (actions.ownedOrExplain()) void actions.setRejected(scope.rejected !== 'all');
  }

  function toggleFavorite(): void {
    if (actions.ownedOrExplain()) void actions.setFavorite(scope.favorite !== 'all');
  }

  function closeMore(open: boolean): void {
    more = open;
    if (open || !next) return;
    sheet = next;
    next = null;
  }

  function rate(value: number | null): void {
    sheet = null;
    void actions.setRating(value);
  }

  function label(color: LabelColor | null): void {
    sheet = null;
    void actions.setLabel(color);
  }
</script>

<div
  {@attach actions.barRef}
  role="toolbar"
  aria-label="Selection actions"
  data-bulk-bar
  class="flex shrink-0 items-start justify-around border-t border-hairline bg-light-50 px-1 pt-0.5 pb-(--tab-bar-pad)"
  style:height={TAB_BAR_BOTTOM}
>
  <Popover open={sheet === 'rate'} onOpenChange={opener('rate')} appearance="control" title="Rate">
    {#snippet trigger(props)}
      <BarAction {...props} icon={rating.icon} label="Rate" blocked={shared} disabled={busy} />
    {/snippet}
    <RatingSwatches rating={scope.rating} size="large" onpick={rate} />
  </Popover>
  <Popover
    open={sheet === 'label'}
    onOpenChange={opener('label')}
    appearance="control"
    title="Colour label"
  >
    {#snippet trigger(props)}
      <BarAction
        {...props}
        icon={labelDot.icon}
        iconClass={labelDot.tone}
        label="Label"
        blocked={shared}
        disabled={busy}
      />
    {/snippet}
    <LabelSwatches label={scope.label} size="large" onpick={label} />
  </Popover>
  <BarAction
    icon={favorite.icon}
    iconClass={favorite.tone}
    label={favorite.label}
    aria-pressed={favorite.pressed}
    blocked={shared}
    disabled={busy}
    onclick={toggleFavorite}
  />
  <BarAction
    icon={reject.icon}
    iconClass={reject.tone}
    label={reject.label}
    aria-pressed={reject.pressed}
    blocked={shared}
    disabled={busy}
    onclick={toggleReject}
  />
  <Popover
    open={sheet === 'album'}
    onOpenChange={opener('album')}
    appearance="panel"
    title="Albums"
    subtitle={photos}
    sheetPicker
    sheetClose="icon"
    sheetClass="flex min-h-full flex-col px-4"
  >
    {#snippet trigger(props)}
      <BarAction
        {...props}
        icon={mdiImageAlbum}
        label="Albums"
        class="hidden wide:flex"
        disabled={busy}
      />
    {/snippet}
    <BulkAlbumBand onDone={() => (sheet = null)} />
  </Popover>
  <Popover
    open={sheet === 'tags'}
    onOpenChange={opener('tags')}
    appearance="panel"
    title="Tags"
    subtitle={photos}
    sheetPicker
    sheetClose="icon"
    sheetClass="flex min-h-full flex-col px-4"
  >
    {#snippet trigger(props)}
      <BarAction
        {...props}
        icon={mdiTagOutline}
        label="Tags"
        class="hidden wide:flex"
        blocked={shared}
        disabled={busy}
      />
    {/snippet}
    <BulkTagBand onDone={() => (sheet = null)} />
  </Popover>
  <BarAction
    icon={mdiExportVariant}
    label="Export"
    aria-haspopup="dialog"
    aria-expanded={sheet === 'export'}
    disabled={empty || scope.selectingAll}
    onclick={() => opener('export')(sheet !== 'export')}
  />
  <BarAction
    icon={mdiDotsHorizontal}
    label="More"
    aria-haspopup="dialog"
    aria-expanded={more}
    onclick={() => (more = true)}
  />
</div>

<Popover
  open={sheet === 'preset'}
  onOpenChange={opener('preset')}
  appearance="panel"
  title="Apply preset"
  subtitle={photos}
  sheetClass="pb-3"
>
  <BulkApplyPreset onApplied={() => (sheet = null)} />
</Popover>

<ActionSheet
  open={sheet === 'location'}
  onOpenChange={opener('location')}
  title="Location"
  subtitle={photos}
>
  <BulkLocationRows onDone={() => (sheet = null)} />
</ActionSheet>

{#if exportSheet.mounted}
  <TouchBulkExportSheet
    open={sheet === 'export'}
    title="Export {photos}"
    onOpenChange={opener('export')}
    onExited={exportSheet.exited}
  />
{/if}

{#if more}
  <BulkMoreSheet open={more} onOpenChange={closeMore} onPick={(name) => (next = name)} />
{/if}
