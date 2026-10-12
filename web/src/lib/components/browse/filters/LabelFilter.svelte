<script lang="ts">
  import {
    segmentedControlClass,
    segmentedRadioItemClass
  } from '$lib/components/editor/controls/segmentedControl';
  import { browseControls, type LabelFilter } from '$lib/stores/browseControls.svelte';
  import { LABEL_COLORS, LABEL_NAMES, LABEL_TEXT } from '$lib/stores/labels';
  import { Icon, Tooltip } from '@immich/ui';
  import { RadioGroup } from 'bits-ui';
  import { mdiCircle, mdiCircleOffOutline } from '@mdi/js';
</script>

<div class="flex flex-col gap-1">
  <span class="editor-compact-label select-none">Label</span>
  <RadioGroup.Root
    bind:value={
      () => browseControls.filters.label, (v) => (browseControls.filters.label = v as LabelFilter)
    }
    orientation="horizontal"
    aria-label="Label"
    class={segmentedControlClass}
  >
    <div class="flex shrink-0 gap-0.5">
      <RadioGroup.Item value="any" class="{segmentedRadioItemClass} px-2">Any</RadioGroup.Item>
      <Tooltip text="No label">
        {#snippet child({ props })}
          <RadioGroup.Item
            {...props}
            value="none"
            aria-label="No label"
            class="{segmentedRadioItemClass} px-2"
          >
            <Icon icon={mdiCircleOffOutline} size="14px" aria-hidden="true" />
          </RadioGroup.Item>
        {/snippet}
      </Tooltip>
    </div>
    <span class="my-1 w-px shrink-0 bg-gray-300 dark:bg-gray-600" aria-hidden="true"></span>
    {#each LABEL_COLORS as color (color)}
      <Tooltip text={`${LABEL_NAMES[color]} label`}>
        {#snippet child({ props })}
          <RadioGroup.Item
            {...props}
            value={color}
            aria-label={`${LABEL_NAMES[color]} label`}
            class="{segmentedRadioItemClass} flex-1"
          >
            <Icon icon={mdiCircle} size="14px" class={LABEL_TEXT[color]} aria-hidden="true" />
          </RadioGroup.Item>
        {/snippet}
      </Tooltip>
    {/each}
  </RadioGroup.Root>
</div>
