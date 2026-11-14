<script lang="ts">
  import Chip from '#lib/components/chips/Chip.svelte';
  import IconToggle from '#lib/components/chips/IconToggle.svelte';
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import { structuredRoute } from '#lib/browse/browseRoutes.ts';
  import {
    ratingOp,
    ratingStars,
    starRating,
    STARS,
    type RatingOp,
    type Star
  } from '#lib/browse/rating.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { page } from '$app/state';
  import { Icon } from '@immich/ui';
  import {
    mdiEqual,
    mdiGreaterThanOrEqual,
    mdiLessThanOrEqual,
    mdiStar,
    mdiStarOffOutline,
    mdiStarOutline
  } from '@mdi/js';

  const OPS: readonly { value: RatingOp; label: string; icon: string }[] = [
    { value: 'gte', label: 'At least', icon: mdiGreaterThanOrEqual },
    { value: 'lte', label: 'At most', icon: mdiLessThanOrEqual },
    { value: 'eq', label: 'Exactly', icon: mdiEqual }
  ];

  let hover = $state(0);
  let opChoice = $state<RatingOp>('gte');

  const rating = $derived(browseControls.filters.rating);
  const ranges = $derived(structuredRoute(page.url));
  const stars = $derived(ratingStars(rating));
  const op = $derived(ranges ? (ratingOp(rating) ?? opChoice) : 'eq');
  const lit = $derived(hover || stars);

  function pickStar(n: Star, pressed: boolean): void {
    browseControls.filters.rating = pressed ? starRating(n, op) : 'any';
  }

  function pickOp(next: RatingOp): void {
    opChoice = next;
    if (stars !== 0) browseControls.filters.rating = starRating(stars, next);
  }
</script>

<div class="flex items-center gap-2">
  <div
    role="group"
    aria-label="Rating"
    class="-ml-1 flex text-[17px] touch:-ml-2.5 touch:text-[26px]"
  >
    {#each STARS as n (n)}
      <IconToggle
        pressed={n === stars}
        onPressedChange={(pressed) => pickStar(n, pressed)}
        label={countOf(n, 'star')}
        onpointerenter={() => (hover = n)}
        onpointerleave={() => (hover = 0)}
      >
        <Icon
          icon={n <= lit ? mdiStar : mdiStarOutline}
          size="1em"
          class={n <= lit ? 'text-primary' : 'text-dark/40'}
          aria-hidden="true"
        />
      </IconToggle>
    {/each}
  </div>
  <span class="flex-1"></span>
  <Chip
    pressed={rating === 'unrated'}
    onPressedChange={(pressed) => (browseControls.filters.rating = pressed ? 'unrated' : 'any')}
    icon={mdiStarOffOutline}
  >
    Unrated
  </Chip>
</div>
{#if ranges}
  <SegmentedChoice
    appearance="view"
    fill
    options={OPS}
    value={op}
    label="Rating match"
    onChange={pickOp}
  />
{/if}
