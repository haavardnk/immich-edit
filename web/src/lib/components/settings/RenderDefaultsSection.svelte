<script lang="ts">
  import { onMount } from 'svelte';
  import { getRenderDefaults, setRenderDefaults, type RenderDefaults } from '$lib/api/admin';
  import { editedThumbs } from '$lib/stores/editedThumbs.svelte';
  import { errorMessage } from '$lib/utils/errors';
  import Notice from '$lib/components/Notice.svelte';
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

<section class="space-y-3 py-2">
  {#if error}
    <Notice message={error} />
  {/if}
  {#if defaults}
    <div class="flex items-center gap-3 text-xs">
      <div class="min-w-0 flex-1">
        <div>Automatic lens corrections</div>
        <div class="text-dark/65">
          Apply matched lens profiles to raw files that have no lens choice of their own.
        </div>
      </div>
      <Switch
        checked={defaults.lens_auto}
        aria-label="Automatic lens corrections"
        onCheckedChange={(checked) => void toggleLensAuto(checked)}
        disabled={busy}
      />
    </div>
  {/if}
</section>
