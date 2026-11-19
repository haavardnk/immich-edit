<script lang="ts">
  import { onMount } from 'svelte';
  import { getRenderDefaults, setRenderDefaults, type RenderDefaults } from '#lib/api/admin.ts';
  import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';
  import { errorMessage } from '#lib/utils/errors.ts';
  import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
  import { Switch } from '@immich/ui';

  let defaults = $state<RenderDefaults | null>(null);
  let busy = $state(false);
  let error = $state<string | null>(null);

  async function load(): Promise<void> {
    try {
      defaults = await getRenderDefaults();
    } catch (e) {
      error = errorMessage(e);
    }
  }

  async function toggleLensAuto(enabled: boolean): Promise<void> {
    if (busy) return;
    busy = true;
    error = null;
    try {
      defaults = await setRenderDefaults({ lens_auto: enabled });
      await editedThumbs.refresh();
    } catch (e) {
      error = errorMessage(e);
    } finally {
      busy = false;
    }
  }

  onMount(() => {
    void load();
  });
</script>

<SettingsRow
  label="Automatic lens corrections"
  hint={error ?? 'For raw files without a lens choice'}
  anchor="lens-auto"
>
  {#if defaults}
    <Switch
      checked={defaults.lens_auto}
      aria-label="Automatic lens corrections"
      onCheckedChange={(checked) => void toggleLensAuto(checked)}
      disabled={busy}
    />
  {/if}
</SettingsRow>
