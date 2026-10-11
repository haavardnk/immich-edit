<script lang="ts">
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { DatePicker, Field } from '@immich/ui';
  import { DateTime } from 'luxon';
</script>

<Field label="Taken after" size="tiny" class="editor-compact-input-field">
  <DatePicker
    bind:value={
      () => (browseControls.takenAfter ? DateTime.fromISO(browseControls.takenAfter) : undefined),
      (date) => (browseControls.takenAfter = date?.toISODate() ?? '')
    }
  />
</Field>

<Field label="Taken before" size="tiny" class="editor-compact-input-field">
  <DatePicker
    bind:value={
      () => (browseControls.takenBefore ? DateTime.fromISO(browseControls.takenBefore) : undefined),
      (date) => (browseControls.takenBefore = date?.toISODate() ?? '')
    }
  />
</Field>
