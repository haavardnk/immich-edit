<script lang="ts">
  import { getInstance, type InstanceInfo } from '#lib/api/admin.ts';
  import RebindDialog from '#lib/components/settings/RebindDialog.svelte';
  import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
  import { errorMessage } from '#lib/utils/errors.ts';
  import { Button } from '@immich/ui';
  import { onMount } from 'svelte';

  let instance = $state<InstanceInfo | null>(null);
  let loadError = $state<string | null>(null);
  let showRebind = $state(false);

  async function load(): Promise<void> {
    try {
      instance = await getInstance();
      loadError = null;
    } catch (e) {
      loadError = errorMessage(e);
    }
  }

  onMount(() => {
    void load();
  });
</script>

<SettingsRow label="Immich instance" anchor="instance">
  {#snippet detail()}
    {#if loadError}
      <span class="text-danger-800">{loadError}</span>
    {:else if instance}
      <span
        class="me-1.5 inline-block size-1.5 rounded-full bg-success align-middle"
        aria-hidden="true"
      ></span><span class="font-mono">{instance.immich_url}</span>
    {/if}
  {/snippet}
  <Button
    size="small"
    variant="ghost"
    color="secondary"
    class="shrink-0 bg-light-200/60 touch:min-h-11"
    onclick={() => (showRebind = true)}
  >
    Change server…
  </Button>
</SettingsRow>

{#if showRebind}
  <RebindDialog onClose={() => (showRebind = false)} />
{/if}
