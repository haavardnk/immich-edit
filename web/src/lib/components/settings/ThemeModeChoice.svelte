<script lang="ts">
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import { appearance } from '#lib/stores/appearance.svelte.ts';
  import type { ThemeMode } from '#lib/theme/types.ts';
  import { mdiMonitor, mdiWeatherNight, mdiWhiteBalanceSunny } from '@mdi/js';

  let {
    label = 'Mode',
    fill = false,
    compact = false
  }: { label?: string; fill?: boolean; compact?: boolean } = $props();

  const MODES: { value: ThemeMode; label: string; icon: string }[] = [
    { value: 'system', label: 'System', icon: mdiMonitor },
    { value: 'light', label: 'Light', icon: mdiWhiteBalanceSunny },
    { value: 'dark', label: 'Dark', icon: mdiWeatherNight }
  ];

  const options = $derived(compact ? MODES.map(({ value, label }) => ({ value, label })) : MODES);
</script>

<SegmentedChoice
  {options}
  value={appearance.current.mode}
  {label}
  {fill}
  appearance={compact ? 'view' : 'panel'}
  onChange={(mode) => appearance.setMode(mode)}
/>
