<script lang="ts">
  import { page } from '$app/state';
  import { IconButton } from '@immich/ui';
  import { mdiClose } from '@mdi/js';

  let { shape = 'rectangle', class: className }: { shape?: 'rectangle' | 'round'; class?: string } =
    $props();

  const onSettings = $derived(page.url.pathname.startsWith('/settings'));
</script>

{#if onSettings}
  <IconButton
    size="medium"
    variant="ghost"
    color="primary"
    {shape}
    class={className}
    icon={mdiClose}
    title="Close settings"
    aria-label="Close settings"
    onclick={() => window.history.back()}
  />
{/if}
