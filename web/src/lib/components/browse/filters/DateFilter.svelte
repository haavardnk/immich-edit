<script lang="ts">
  import { page } from '$app/state';
  import Chip from '#lib/components/chips/Chip.svelte';
  import ChipList from '#lib/components/chips/ChipList.svelte';
  import DateRangeFields from '#lib/components/browse/filters/DateRangeFields.svelte';
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import { loadsWhole, structuredRoute, timelineBase } from '#lib/browse/browseRoutes.ts';
  import {
    DATE_BASES,
    DATE_BASIS_LABELS,
    DATE_KEYS,
    MONTHS,
    calendarYears,
    datePeriod,
    dateRange,
    loadedMonths,
    monthName,
    monthOpen,
    monthRange,
    moveDateRange,
    selectedBasis,
    withoutDates,
    yearRange,
    type DateBasis,
    type DateRange
  } from '#lib/browse/datePeriods.ts';
  import { serverFilter } from '#lib/browse/serverFilter.ts';
  import { browseControls, type BrowseFilters } from '#lib/stores/browseControls.svelte.ts';
  import { browsing } from '#lib/stores/browsing.svelte.ts';
  import { MonthCountsStore } from '#lib/stores/monthCounts.svelte.ts';
  import { mdiCalendarOutline } from '@mdi/js';

  const BASIS_OPTIONS = DATE_BASES.map((value) => ({ value, label: DATE_BASIS_LABELS[value] }));
  const NO_RANGE: DateRange = { after: '', before: '' };

  const months = new MonthCountsStore();
  const today = new Date();

  let basisChoice = $state<DateBasis>('taken');
  let customFor = $state.raw<BrowseFilters | null>(null);

  const filters = $derived(browseControls.filters);
  const basis = $derived(selectedBasis(filters, basisChoice));
  const period = $derived(datePeriod(dateRange(filters, basis)));
  const custom = $derived(period.kind === 'custom' || customFor === filters);
  const year = $derived(
    !custom && (period.kind === 'year' || period.kind === 'month') ? period.year : null
  );
  const month = $derived(!custom && period.kind === 'month' ? period.month : null);
  const base = $derived(timelineBase(page.url));
  const monthFilter = $derived(
    base && serverFilter(withoutDates(filters), base, structuredRoute(page.url))
  );
  const counts = $derived.by(() => {
    if (monthFilter) return months.loaded ? months.months : [];
    return loadsWhole(page.url) ? loadedMonths(browsing.assets) : null;
  });
  const years = $derived(calendarYears(counts, basis, today, year));
  const known = $derived(years.find((entry) => entry.year === year)?.months ?? null);

  $effect(() => {
    if (monthFilter) months.ensure(monthFilter);
  });

  function setRange(range: DateRange): void {
    const keys = DATE_KEYS[basis];
    browseControls.filters[keys.after] = range.after;
    browseControls.filters[keys.before] = range.before;
  }

  function pickBasis(next: DateBasis): void {
    const moved = moveDateRange(filters, basis, next);
    basisChoice = next;
    for (const keys of Object.values(DATE_KEYS)) {
      browseControls.filters[keys.after] = moved[keys.after];
      browseControls.filters[keys.before] = moved[keys.before];
    }
  }

  function pickYear(value: number, pressed: boolean): void {
    customFor = null;
    setRange(pressed ? yearRange(value) : NO_RANGE);
  }

  function pickMonth(value: number, pressed: boolean): void {
    if (year === null) return;
    setRange(pressed ? monthRange(year, value) : yearRange(year));
  }

  function pickCustom(pressed: boolean): void {
    customFor = pressed ? filters : null;
    if (!pressed) setRange(NO_RANGE);
  }
</script>

<SegmentedChoice
  appearance="view"
  options={BASIS_OPTIONS}
  value={basis}
  label="Date"
  onChange={pickBasis}
/>
<ChipList
  items={years}
  key={(entry) => String(entry.year)}
  selected={(entry) => entry.year === year}
  limit={7}
  label="Year"
>
  {#snippet chip(entry)}
    <Chip
      pressed={entry.year === year}
      onPressedChange={(pressed) => pickYear(entry.year, pressed)}
    >
      {entry.year}
    </Chip>
  {/snippet}
  <Chip pressed={custom} onPressedChange={pickCustom} icon={mdiCalendarOutline}>Custom</Chip>
</ChipList>
{#if year !== null}
  <div role="group" aria-label="Month" class="grid grid-cols-6 gap-1 touch:gap-1.5">
    {#each MONTHS as value (value)}
      <Chip
        class="w-full px-0 touch:px-0"
        pressed={value === month}
        disabled={value !== month && !monthOpen(year, value, known, today)}
        label="{monthName(value)} {year}"
        onPressedChange={(pressed) => pickMonth(value, pressed)}
      >
        {monthName(value)}
      </Chip>
    {/each}
  </div>
{:else if custom}
  <DateRangeFields {basis} />
{/if}
