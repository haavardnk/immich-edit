<script lang="ts">
  import type { MarkerFilter } from '#lib/api/map.ts';
  import Chip from '#lib/components/chips/Chip.svelte';
  import { chipTone } from '#lib/components/chips/chipStyles.ts';
  import Popover from '#lib/components/Popover.svelte';
  import { Button, DatePicker, Field, Icon } from '@immich/ui';
  import { mdiCalendarRange, mdiHeartOutline } from '@mdi/js';
  import { DateTime } from 'luxon';

  let {
    filter,
    onChange
  }: {
    filter: MarkerFilter;
    onChange: (filter: MarkerFilter) => void;
  } = $props();

  let open = $state(false);

  const FIELDS = [
    { key: 'after', label: 'From' },
    { key: 'before', label: 'To' }
  ] as const;

  function day(value: string | undefined): string {
    return value ? (DateTime.fromISO(value).toLocaleString(DateTime.DATE_MED) ?? value) : '';
  }

  const dateLabel = $derived.by(() => {
    if (filter.after && filter.before) return `${day(filter.after)} – ${day(filter.before)}`;
    if (filter.after) return `From ${day(filter.after)}`;
    if (filter.before) return `Until ${day(filter.before)}`;
    return 'Any date';
  });
</script>

<div class="flex min-w-0 items-center gap-1.5">
  <Chip
    pressed={filter.favorite ?? false}
    onPressedChange={(favorite) => onChange({ ...filter, favorite })}
    icon={mdiHeartOutline}
  >
    Favorites
  </Chip>
  <Popover
    {open}
    onOpenChange={(next) => (open = next)}
    appearance="panel"
    title="Dates"
    contentClass="flex w-64 flex-col gap-3 p-3"
    sheetClass="flex flex-col gap-3 px-4 py-3"
  >
    {#snippet trigger(props)}
      <button
        {...props}
        type="button"
        aria-pressed={Boolean(filter.after || filter.before)}
        class={chipTone.value}
      >
        <Icon icon={mdiCalendarRange} size="1.15em" class="shrink-0" aria-hidden="true" />
        {dateLabel}
      </button>
    {/snippet}
    <div class="flex flex-col gap-3">
      {#each FIELDS as { key, label } (key)}
        <Field {label} size="tiny" class="compact-date-field min-w-0">
          <DatePicker
            bind:value={
              () => (filter[key] ? DateTime.fromISO(filter[key]) : undefined),
              (date) => onChange({ ...filter, [key]: date?.toISODate() ?? undefined })
            }
          />
        </Field>
      {/each}
    </div>
    {#if filter.after || filter.before}
      <Button
        size="small"
        variant="ghost"
        color="secondary"
        class="self-start touch:h-11"
        onclick={() => onChange({ ...filter, after: undefined, before: undefined })}
      >
        Clear dates
      </Button>
    {/if}
  </Popover>
</div>
