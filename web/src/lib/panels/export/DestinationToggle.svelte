<script lang="ts">
  import { segmented } from '#lib/components/segmented.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import type { Destination } from './settings';
  import { Icon } from '@immich/ui';
  import { mdiCellphone, mdiCloudOutline } from '@mdi/js';
  import { RadioGroup } from 'bits-ui';

  let { value = $bindable<Destination>() }: { value: Destination } = $props();

  const touch = $derived(layout.mode === 'touch');
  const options = $derived<{ value: Destination; label: string; icon: string }[]>([
    { value: 'download', label: touch ? 'Device' : 'Download', icon: mdiCellphone },
    { value: 'immich', label: touch ? 'Immich' : 'To Immich', icon: mdiCloudOutline }
  ]);
</script>

<RadioGroup.Root
  bind:value={() => value, (v) => (value = v as Destination)}
  orientation="horizontal"
  aria-label="Export destination"
  class={segmented.text.group}
>
  {#each options as option (option.value)}
    <RadioGroup.Item value={option.value} class="{segmented.text.radio} flex-1 gap-1.5">
      {#if touch}
        <Icon icon={option.icon} size="1.15em" aria-hidden="true" />
      {/if}
      {option.label}
    </RadioGroup.Item>
  {/each}
</RadioGroup.Root>
