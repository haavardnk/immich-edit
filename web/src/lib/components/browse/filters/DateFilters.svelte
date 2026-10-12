<script lang="ts">
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { DatePicker, Field } from '@immich/ui';
  import { DateTime } from 'luxon';
</script>

<Field label="Taken after" size="tiny" class="editor-compact-input-field">
  <DatePicker
    bind:value={
      () =>
        browseControls.filters.takenAfter
          ? DateTime.fromISO(browseControls.filters.takenAfter)
          : undefined,
      (date) => (browseControls.filters.takenAfter = date?.toISODate() ?? '')
    }
  />
</Field>

<Field label="Taken before" size="tiny" class="editor-compact-input-field">
  <DatePicker
    bind:value={
      () =>
        browseControls.filters.takenBefore
          ? DateTime.fromISO(browseControls.filters.takenBefore)
          : undefined,
      (date) => (browseControls.filters.takenBefore = date?.toISODate() ?? '')
    }
  />
</Field>
