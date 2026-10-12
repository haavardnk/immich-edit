<script lang="ts">
  import {
    segmentedControlClass,
    segmentedRadioItemClass
  } from '$lib/components/editor/controls/segmentedControl';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { structuredRoute } from '$lib/browse/browseRoutes';
  import { page } from '$app/state';
  import { ratingOp, ratingStars, starRating, STARS, type RatingOp } from '$lib/browse/rating';
  import { Icon, Tooltip } from '@immich/ui';
  import { mergeProps, RadioGroup } from 'bits-ui';
  import { mdiStar, mdiStarOffOutline, mdiStarOutline } from '@mdi/js';

  const OPS: Record<RatingOp, { symbol: string; name: string; next: RatingOp }> = {
    gte: { symbol: '≥', name: 'at least', next: 'lte' },
    lte: { symbol: '≤', name: 'at most', next: 'eq' },
    eq: { symbol: '=', name: 'exactly', next: 'gte' }
  };

  const starItemClass =
    'flex h-6 flex-1 items-center justify-center rounded-sm transition-colors hover:bg-ghost';

  let hover = $state(0);
  let opChoice = $state<RatingOp>('gte');

  const ranges = $derived(structuredRoute(page.url));
  const stars = $derived(ratingStars(browseControls.filters.rating));
  const op = $derived(ranges ? (ratingOp(browseControls.filters.rating) ?? opChoice) : 'eq');
  const lit = $derived(hover || stars);

  const starName = (n: number): string => `${n} star${n > 1 ? 's' : ''}`;

  function pickStar(value: string): void {
    const n = STARS.find((star) => String(star) === value);
    if (n) browseControls.filters.rating = starRating(n, op);
  }

  function cycleOp(): void {
    opChoice = OPS[op].next;
    if (stars !== 0) browseControls.filters.rating = starRating(stars, opChoice);
  }
</script>

<div class="flex flex-col gap-1">
  <span class="editor-compact-label select-none">Rating</span>
  <div class={segmentedControlClass}>
    <RadioGroup.Root
      bind:value={
        () => (stars === 0 ? String(browseControls.filters.rating) : ''),
        (v) => (browseControls.filters.rating = v === 'unrated' ? 'unrated' : 'any')
      }
      orientation="horizontal"
      aria-label="Rating presets"
      class="flex shrink-0 gap-0.5"
    >
      <RadioGroup.Item value="any" class="{segmentedRadioItemClass} px-2">Any</RadioGroup.Item>
      <Tooltip text="Unrated">
        {#snippet child({ props })}
          <RadioGroup.Item
            {...props}
            value="unrated"
            aria-label="Unrated"
            class="{segmentedRadioItemClass} px-2"
          >
            <Icon icon={mdiStarOffOutline} size="14px" aria-hidden="true" />
          </RadioGroup.Item>
        {/snippet}
      </Tooltip>
    </RadioGroup.Root>
    <span class="my-1 w-px shrink-0 bg-gray-300 dark:bg-gray-600" aria-hidden="true"></span>
    {#if ranges}
      <Tooltip text="Match ratings {OPS[op].name} the selected stars">
        {#snippet child({ props })}
          <button
            {...mergeProps(props, {
              type: 'button' as const,
              class:
                'flex h-6 w-6 shrink-0 items-center justify-center rounded-sm text-sm font-semibold text-primary transition-colors hover:bg-ghost',
              'aria-label': `Rating match: ${OPS[op].name}`,
              onclick: cycleOp
            })}
          >
            {OPS[op].symbol}
          </button>
        {/snippet}
      </Tooltip>
    {/if}
    <RadioGroup.Root
      bind:value={() => (stars === 0 ? '' : String(stars)), pickStar}
      orientation="horizontal"
      aria-label="Rating"
      class="flex flex-1 gap-0.5"
      onmouseleave={() => (hover = 0)}
    >
      {#each STARS as n (n)}
        <Tooltip text={starName(n)}>
          {#snippet child({ props })}
            <RadioGroup.Item
              {...props}
              value={String(n)}
              aria-label={starName(n)}
              class={starItemClass}
              onmouseenter={() => (hover = n)}
            >
              <Icon
                icon={n <= lit ? mdiStar : mdiStarOutline}
                size="14px"
                class={n <= lit ? 'text-primary' : 'text-dark/45'}
                aria-hidden="true"
              />
            </RadioGroup.Item>
          {/snippet}
        </Tooltip>
      {/each}
    </RadioGroup.Root>
  </div>
</div>
