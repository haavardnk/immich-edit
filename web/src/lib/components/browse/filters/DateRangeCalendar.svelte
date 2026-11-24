<script lang="ts">
  import { boundDay, boundTime, joinBound, type DateBoundKey } from '#lib/browse/dateBounds.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { IconButton, getLocale } from '@immich/ui';
  import { parseDate } from '@internationalized/date';
  import { mdiChevronLeft, mdiChevronRight } from '@mdi/js';
  import { RangeCalendar, type DateRange } from 'bits-ui';
  import { untrack } from 'svelte';

  let { after, before }: { after: DateBoundKey; before: DateBoundKey } = $props();

  function day(key: DateBoundKey): DateRange['start'] {
    const picked = boundDay(browseControls.filters[key]);
    return picked ? parseDate(picked) : undefined;
  }

  let range = $state<DateRange>(untrack(() => ({ start: day(after), end: day(before) })));
  let root: HTMLDivElement | undefined = $state();

  $effect(() => {
    const picked = { start: day(after), end: day(before) };
    untrack(() => (range = picked));
  });

  export function focus(): void {
    const active = root?.querySelector<HTMLElement>('[role="gridcell"] [tabindex="0"]');
    (active ?? root?.querySelector<HTMLElement>('button'))?.focus();
  }

  function pick(next: DateRange): void {
    if (!next.start || !next.end) return;
    const filters = browseControls.filters;
    filters[after] = joinBound(next.start.toString(), boundTime(filters[after]));
    filters[before] = joinBound(next.end.toString(), boundTime(filters[before]));
  }

  const cellClass =
    'flex-1 p-0 data-range-middle:bg-primary/15 ' +
    'data-range-start:not-data-range-end:bg-linear-to-r data-range-start:not-data-range-end:from-transparent data-range-start:not-data-range-end:from-50% data-range-start:not-data-range-end:to-primary/15 data-range-start:not-data-range-end:to-50% ' +
    'data-range-end:not-data-range-start:bg-linear-to-l data-range-end:not-data-range-start:from-transparent data-range-end:not-data-range-start:from-50% data-range-end:not-data-range-start:to-primary/15 data-range-end:not-data-range-start:to-50%';

  const dayClass =
    'flex h-8 w-full items-center justify-center rounded-full text-sm tabular-nums outline-none ' +
    'hover:bg-light-200 focus-visible:ring-2 focus-visible:ring-primary touch:h-10 ' +
    'data-today:font-semibold data-today:text-primary ' +
    'data-outside-month:text-muted data-outside-month:opacity-50 ' +
    'data-highlighted:rounded-none data-highlighted:bg-primary/15 ' +
    'data-range-middle:rounded-none ' +
    'data-selection-start:bg-primary data-selection-start:text-light ' +
    'data-selection-end:bg-primary data-selection-end:text-light ' +
    'data-disabled:pointer-events-none data-disabled:opacity-40';
</script>

<div bind:this={root}>
  <RangeCalendar.Root
    bind:value={range}
    onValueChange={pick}
    locale={getLocale()}
    weekStartsOn={1}
    preventDeselect
    weekdayFormat="short"
    class="select-none"
  >
    {#snippet children({ months, weekdays })}
      <RangeCalendar.Header class="mb-1 flex items-center justify-between">
        <RangeCalendar.PrevButton>
          {#snippet child({ props })}
            <IconButton
              {...props}
              size="small"
              variant="ghost"
              color="secondary"
              shape="round"
              icon={mdiChevronLeft}
              title="Previous month"
              aria-label="Previous month"
            />
          {/snippet}
        </RangeCalendar.PrevButton>
        <RangeCalendar.Heading class="text-sm font-medium" />
        <RangeCalendar.NextButton>
          {#snippet child({ props })}
            <IconButton
              {...props}
              size="small"
              variant="ghost"
              color="secondary"
              shape="round"
              icon={mdiChevronRight}
              title="Next month"
              aria-label="Next month"
            />
          {/snippet}
        </RangeCalendar.NextButton>
      </RangeCalendar.Header>
      {#each months as month (month.value.toString())}
        <RangeCalendar.Grid class="w-full border-collapse">
          <RangeCalendar.GridHead>
            <RangeCalendar.GridRow class="flex w-full">
              {#each weekdays as weekday, index (index)}
                <RangeCalendar.HeadCell
                  class="flex h-6 flex-1 items-center justify-center text-xs font-normal text-muted"
                >
                  {weekday.slice(0, 2)}
                </RangeCalendar.HeadCell>
              {/each}
            </RangeCalendar.GridRow>
          </RangeCalendar.GridHead>
          <RangeCalendar.GridBody>
            {#each month.weeks as week (week[0]?.toString())}
              <RangeCalendar.GridRow class="flex w-full">
                {#each week as date (date.toString())}
                  <RangeCalendar.Cell {date} month={month.value} class={cellClass}>
                    <RangeCalendar.Day class={dayClass} />
                  </RangeCalendar.Cell>
                {/each}
              </RangeCalendar.GridRow>
            {/each}
          </RangeCalendar.GridBody>
        </RangeCalendar.Grid>
      {/each}
    {/snippet}
  </RangeCalendar.Root>
</div>
