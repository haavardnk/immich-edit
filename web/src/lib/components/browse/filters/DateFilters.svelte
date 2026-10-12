<script lang="ts">
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { DatePicker, Field } from '@immich/ui';
  import { DateTime } from 'luxon';

  const FIELDS = [
    { key: 'takenAfter', label: 'Taken after' },
    { key: 'takenBefore', label: 'Taken before' },
    { key: 'addedAfter', label: 'Added after' },
    { key: 'addedBefore', label: 'Added before' }
  ] as const;
</script>

{#key browseControls.filters}
  {#each FIELDS as { key, label } (key)}
    <Field {label} size="tiny" class="editor-compact-input-field">
      <DatePicker
        bind:value={
          () =>
            browseControls.filters[key] ? DateTime.fromISO(browseControls.filters[key]) : undefined,
          (date) => (browseControls.filters[key] = date?.toISODate() ?? '')
        }
      />
    </Field>
  {/each}
{/key}
