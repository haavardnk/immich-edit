<script lang="ts">
  import { segmented } from '#lib/components/segmented.ts';
  import { SORT_DIRECTION_LABELS, SORT_FIELD_LABELS, sortFields } from '#lib/browse/sortOptions.ts';
  import {
    browseControls,
    type SortDir,
    type SortField
  } from '#lib/stores/browseControls.svelte.ts';
  import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';
  import { Label } from '@immich/ui';
  import { RadioGroup } from 'bits-ui';

  const fields = $derived(sortFields(immichCapabilities.structuredSearch));
  const directions = $derived(SORT_DIRECTION_LABELS[browseControls.sortField]);
</script>

<div class="flex flex-col gap-1">
  <Label label="Sort by" size="small" />
  <RadioGroup.Root
    bind:value={() => browseControls.sortField, (v) => browseControls.setSortField(v as SortField)}
    orientation="horizontal"
    aria-label="Sort by"
    class={segmented.row.group}
  >
    {#each fields as field (field)}
      <RadioGroup.Item value={field} class={segmented.row.radio}>
        {SORT_FIELD_LABELS[field]}
      </RadioGroup.Item>
    {/each}
  </RadioGroup.Root>
</div>
<div class="flex flex-col gap-1">
  <Label label="Order" size="small" />
  <RadioGroup.Root
    bind:value={() => browseControls.sortDir, (v) => browseControls.setSortDir(v as SortDir)}
    orientation="horizontal"
    aria-label="Order"
    class={segmented.row.group}
  >
    <RadioGroup.Item value="desc" class={segmented.row.radio}>{directions.desc}</RadioGroup.Item>
    <RadioGroup.Item value="asc" class={segmented.row.radio}>{directions.asc}</RadioGroup.Item>
  </RadioGroup.Root>
</div>
