<script lang="ts">
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import { SORT_DIRECTION_LABELS, SORT_FIELD_LABELS, sortFields } from '#lib/browse/sortOptions.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';
  import { Label } from '@immich/ui';

  const fields = $derived(sortFields(immichCapabilities.structuredSearch));
  const directions = $derived(SORT_DIRECTION_LABELS[browseControls.sortField]);
</script>

<div class="flex flex-col gap-1">
  <Label label="Sort by" size="small" />
  <SegmentedChoice
    label="Sort by"
    fill
    value={browseControls.sortField}
    options={fields.map((field) => ({ value: field, label: SORT_FIELD_LABELS[field] }))}
    onChange={(v) => browseControls.setSortField(v)}
  />
</div>
<div class="flex flex-col gap-1">
  <Label label="Order" size="small" />
  <SegmentedChoice
    label="Order"
    fill
    value={browseControls.sortDir}
    options={[
      { value: 'desc', label: directions.desc },
      { value: 'asc', label: directions.asc }
    ]}
    onChange={(v) => browseControls.setSortDir(v)}
  />
</div>
