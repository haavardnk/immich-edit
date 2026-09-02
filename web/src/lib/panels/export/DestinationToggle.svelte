<script lang="ts">
  import {
    segmentedControlClass,
    segmentedRadioItemClass
  } from '$lib/components/editor/controls/segmentedControl';
  import type { Destination } from './settings';
  import { RadioGroup } from 'bits-ui';

  let {
    value = $bindable<Destination>(),
    downloadLabel = 'Download',
    immichLabel = 'To Immich'
  }: {
    value: Destination;
    downloadLabel?: string;
    immichLabel?: string;
  } = $props();
</script>

<RadioGroup.Root
  bind:value={() => value, (v) => (value = v as Destination)}
  orientation="horizontal"
  aria-label="Export destination"
  class={segmentedControlClass}
>
  <RadioGroup.Item value="download" class="{segmentedRadioItemClass} flex-1">
    {downloadLabel}
  </RadioGroup.Item>
  <RadioGroup.Item value="immich" class="{segmentedRadioItemClass} flex-1">
    {immichLabel}
  </RadioGroup.Item>
</RadioGroup.Root>
