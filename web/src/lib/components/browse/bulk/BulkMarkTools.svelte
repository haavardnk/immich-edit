<script lang="ts">
  import DisabledReason from '#lib/components/DisabledReason.svelte';
  import FavoriteButton from '#lib/components/FavoriteButton.svelte';
  import LabelPicker from '#lib/components/LabelPicker.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import RatingSwatches from '#lib/components/RatingSwatches.svelte';
  import RejectButton from '#lib/components/RejectButton.svelte';
  import StarRating from '#lib/components/StarRating.svelte';
  import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
  import { ROOMY_ONLY, TIGHT_ONLY } from '#lib/browse/bulk/bulkBarFit.ts';
  import { ratingLook } from '#lib/browse/bulk/bulkMarks.ts';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { IconButton } from '@immich/ui';

  let rateOpen = $state(false);

  const scope = bulkScope;
  const actions = bulkActions;
  const locked = $derived(actions.metaBusy || !scope.allOwned);
  const rating = $derived(ratingLook(scope.rating));

  function rate(value: number | null): void {
    rateOpen = false;
    void actions.setRating(value);
  }
</script>

<DisabledReason
  locked={!scope.allOwned}
  reason={scope.notOwnedReason}
  class="flex shrink-0 items-center gap-1"
>
  <FavoriteButton
    size="medium"
    isFavorite={scope.favorite === 'all'}
    mixed={scope.favorite === 'some'}
    disabled={locked}
    ontoggle={() => void actions.setFavorite(scope.favorite !== 'all')}
  />
  <div class="{ROOMY_ONLY} shrink-0 items-center">
    <StarRating
      size={20}
      rating={scope.rating ?? 0}
      mixed={scope.rating === null}
      disabled={locked}
      onchange={(value) => void actions.setRating(value)}
    />
  </div>
  <Popover
    open={rateOpen}
    onOpenChange={(open) => (rateOpen = open)}
    appearance="control"
    anchor="top"
    title="Rate"
    tooltip={hint('Rate', 'rate')}
  >
    {#snippet trigger(popoverProps)}
      <IconButton
        {...popoverProps}
        size="medium"
        variant="ghost"
        color="secondary"
        class="shrink-0 {TIGHT_ONLY}"
        icon={rating.icon}
        title=""
        aria-label="Rate"
        aria-expanded={rateOpen}
        disabled={locked}
      />
    {/snippet}
    <RatingSwatches rating={scope.rating} onpick={rate} />
  </Popover>
  <RejectButton
    size="medium"
    isRejected={scope.rejected === 'all'}
    mixed={scope.rejected === 'some'}
    disabled={locked}
    ontoggle={() => void actions.setRejected(scope.rejected !== 'all')}
  />
  <LabelPicker
    size="medium"
    label={scope.label}
    disabled={locked}
    onchange={(color) => void actions.setLabel(color)}
  />
</DisabledReason>
