<script lang="ts">
  import {
    segmentedControlClass,
    segmentedRadioItemClass
  } from '#lib/components/editor/controls/segmentedControl.ts';
  import ThemePicker from '#lib/components/settings/ThemePicker.svelte';
  import { appearance } from '#lib/stores/appearance.svelte.ts';
  import type { ThemeMode } from '#lib/theme/types.ts';
  import { RadioGroup } from 'bits-ui';

  const MODES: { value: ThemeMode; label: string }[] = [
    { value: 'system', label: 'System' },
    { value: 'light', label: 'Light' },
    { value: 'dark', label: 'Dark' }
  ];
</script>

<div class="space-y-5 py-2">
  <section class="space-y-2">
    <h2 class="text-xs font-medium">Mode</h2>
    <RadioGroup.Root
      value={appearance.current.mode}
      onValueChange={(value) => {
        const picked = MODES.find((mode) => mode.value === value);
        if (picked) appearance.setMode(picked.value);
      }}
      orientation="horizontal"
      aria-label="Mode"
      class="{segmentedControlClass} grid w-fit grid-cols-3"
    >
      {#each MODES as mode (mode.value)}
        <RadioGroup.Item value={mode.value} class="{segmentedRadioItemClass} px-3">
          {mode.label}
        </RadioGroup.Item>
      {/each}
    </RadioGroup.Root>
  </section>
  <ThemePicker scheme="light" label="Light theme" />
  <ThemePicker scheme="dark" label="Dark theme" />
</div>
