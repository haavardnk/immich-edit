<script lang="ts">
  import AccentPicker from '#lib/components/settings/AccentPicker.svelte';
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import ThemePicker from '#lib/components/settings/ThemePicker.svelte';
  import { appearance } from '#lib/stores/appearance.svelte.ts';
  import type { ThemeMode } from '#lib/theme/types.ts';

  const MODES: { value: ThemeMode; label: string }[] = [
    { value: 'system', label: 'System' },
    { value: 'light', label: 'Light' },
    { value: 'dark', label: 'Dark' }
  ];
</script>

<div class="space-y-5 py-2">
  <section class="space-y-2">
    <h2 class="text-xs font-medium">Mode</h2>
    <SegmentedChoice
      options={MODES}
      value={appearance.current.mode}
      label="Mode"
      onChange={(mode) => appearance.setMode(mode)}
    />
  </section>
  <div class="space-y-4">
    <ThemePicker scheme="light" label="Light theme" />
    <AccentPicker scheme="light" label="Light accent" />
  </div>
  <div class="space-y-4">
    <ThemePicker scheme="dark" label="Dark theme" />
    <AccentPicker scheme="dark" label="Dark accent" />
  </div>
</div>
