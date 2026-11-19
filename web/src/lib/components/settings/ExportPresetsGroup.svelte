<script lang="ts">
  import SettingsGroup from '#lib/components/settings/SettingsGroup.svelte';
  import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
  import { exportPresets } from '#lib/panels/export/exportPresets.svelte.ts';
  import { exportSettings } from '#lib/panels/export/exportSettings.svelte.ts';
  import { presetLine } from '#lib/panels/export/presetLine.ts';
  import { ensureLibraryLoaded } from '#lib/panels/export/settings.ts';
  import { IconButton } from '@immich/ui';
  import { mdiDeleteOutline } from '@mdi/js';
  import { onMount } from 'svelte';

  const rows = $derived(
    exportPresets.presets.map((preset) => ({ preset, line: presetLine(preset) }))
  );

  $effect(() => {
    if (rows.some(({ line }) => line.immich)) ensureLibraryLoaded(exportSettings.form);
  });

  onMount(() => {
    void exportPresets.load();
  });
</script>

<SettingsGroup title="Export presets" anchor="export-presets">
  {#each rows as { preset, line } (preset.id)}
    <SettingsRow label={preset.name}>
      {#snippet detail()}{line.summary}{/snippet}
      <IconButton
        size="small"
        variant="ghost"
        color="secondary"
        class="shrink-0 touch:size-11"
        icon={mdiDeleteOutline}
        title="Delete"
        aria-label="Delete export preset {preset.name}"
        onclick={() => void exportPresets.remove(preset)}
      />
    </SettingsRow>
  {:else}
    <SettingsRow label={exportPresets.loaded ? 'No saved presets' : 'Loading…'} muted />
  {/each}
</SettingsGroup>
