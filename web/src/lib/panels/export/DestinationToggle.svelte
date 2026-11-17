<script lang="ts">
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import type { Destination } from './settings';
  import { mdiCellphone, mdiCloudOutline } from '@mdi/js';

  let { value = $bindable<Destination>() }: { value: Destination } = $props();

  const touch = $derived(layout.mode === 'touch');
  const options = $derived<{ value: Destination; label: string; icon: string }[]>([
    { value: 'download', label: touch ? 'Device' : 'Download', icon: mdiCellphone },
    { value: 'immich', label: touch ? 'Immich' : 'To Immich', icon: mdiCloudOutline }
  ]);
</script>

<SegmentedChoice
  label="Export destination"
  fill
  {value}
  options={options.map((option) => ({ ...option, icon: touch ? option.icon : undefined }))}
  onChange={(v) => (value = v)}
/>
