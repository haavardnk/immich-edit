<script lang="ts">
  import { segmented } from '#lib/components/segmented.ts';
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
  class={segmented.text.group}
>
  <RadioGroup.Item value="download" class="{segmented.text.radio} flex-1">
    {downloadLabel}
  </RadioGroup.Item>
  <RadioGroup.Item value="immich" class="{segmented.text.radio} flex-1">
    {immichLabel}
  </RadioGroup.Item>
</RadioGroup.Root>
