<script lang="ts">
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import { appearance } from '#lib/stores/appearance.svelte.ts';
  import type { ThemeMode } from '#lib/theme/types.ts';

  let { label = 'Mode' }: { label?: string } = $props();

  const MODES: { value: ThemeMode; label: string }[] = [
    { value: 'system', label: 'System' },
    { value: 'light', label: 'Light' },
    { value: 'dark', label: 'Dark' }
  ];
</script>

<SegmentedChoice
  options={MODES}
  value={appearance.current.mode}
  {label}
  onChange={(mode) => appearance.setMode(mode)}
/>
