<script lang="ts">
  import { segmented } from '#lib/components/segmented.ts';
  import AccentPicker from '#lib/components/settings/AccentPicker.svelte';
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
      class="{segmented.text.group} grid w-fit grid-cols-3"
    >
      {#each MODES as mode (mode.value)}
        <RadioGroup.Item value={mode.value} class="{segmented.text.radio} px-3">
          {mode.label}
        </RadioGroup.Item>
      {/each}
    </RadioGroup.Root>
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
