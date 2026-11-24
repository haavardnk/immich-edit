<script lang="ts">
  import DateBoundRow from '#lib/components/browse/filters/DateBoundRow.svelte';
  import DateRangeCalendar from '#lib/components/browse/filters/DateRangeCalendar.svelte';
  import { DATE_KEYS, type DateBasis } from '#lib/browse/datePeriods.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';

  let { basis }: { basis: DateBasis } = $props();

  let calendar: DateRangeCalendar | undefined = $state();

  const keys = $derived(DATE_KEYS[basis]);

  function focusCalendar(): void {
    calendar?.focus();
  }
</script>

{#key browseControls.filters}
  <div class="flex flex-col gap-1">
    <DateRangeCalendar bind:this={calendar} after={keys.after} before={keys.before} />
    <DateBoundRow key={keys.after} label="From" onCleared={focusCalendar} />
    <DateBoundRow key={keys.before} label="To" onCleared={focusCalendar} />
  </div>
{/key}
