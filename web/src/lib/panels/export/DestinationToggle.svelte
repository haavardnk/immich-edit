<script lang="ts">
  import { segmented } from '#lib/components/segmented.ts';
  import type { Destination } from './settings';
  import { Icon } from '@immich/ui';
  import { RadioGroup } from 'bits-ui';

  let {
    value = $bindable<Destination>(),
    downloadLabel = 'Download',
    immichLabel = 'To Immich',
    downloadIcon,
    immichIcon
  }: {
    value: Destination;
    downloadLabel?: string;
    immichLabel?: string;
    downloadIcon?: string;
    immichIcon?: string;
  } = $props();
</script>

<RadioGroup.Root
  bind:value={() => value, (v) => (value = v as Destination)}
  orientation="horizontal"
  aria-label="Export destination"
  class={segmented.text.group}
>
  <RadioGroup.Item value="download" class="{segmented.text.radio} flex-1 gap-1.5">
    {#if downloadIcon}
      <Icon icon={downloadIcon} size="1.15em" aria-hidden="true" />
    {/if}
    {downloadLabel}
  </RadioGroup.Item>
  <RadioGroup.Item value="immich" class="{segmented.text.radio} flex-1 gap-1.5">
    {#if immichIcon}
      <Icon icon={immichIcon} size="1.15em" aria-hidden="true" />
    {/if}
    {immichLabel}
  </RadioGroup.Item>
</RadioGroup.Root>
