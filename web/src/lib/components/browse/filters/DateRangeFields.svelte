<script lang="ts">
  import { DATE_KEYS, type DateBasis } from '#lib/browse/datePeriods.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { DatePicker, Field } from '@immich/ui';
  import { DateTime } from 'luxon';

  let { basis }: { basis: DateBasis } = $props();

  const fields = $derived([
    { key: DATE_KEYS[basis].after, label: 'From' },
    { key: DATE_KEYS[basis].before, label: 'To' }
  ]);
</script>

{#key browseControls.filters}
  <div class="grid grid-cols-2 gap-2">
    {#each fields as { key, label } (key)}
      <Field {label} size="tiny" class="compact-date-field min-w-0">
        <DatePicker
          bind:value={
            () =>
              browseControls.filters[key]
                ? DateTime.fromISO(browseControls.filters[key])
                : undefined,
            (date) => (browseControls.filters[key] = date?.toISODate() ?? '')
          }
        />
      </Field>
    {/each}
  </div>
{/key}
