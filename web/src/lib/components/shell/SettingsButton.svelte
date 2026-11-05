<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { IconButton } from '@immich/ui';
  import { mdiClose, mdiCogOutline } from '@mdi/js';

  let { shape = 'rectangle', class: className }: { shape?: 'rectangle' | 'round'; class?: string } =
    $props();

  const onSettings = $derived(page.url.pathname.startsWith('/settings'));

  function toggleSettings(): void {
    if (onSettings) {
      window.history.back();
    } else {
      void goto('/settings');
    }
  }
</script>

<IconButton
  size="medium"
  variant="ghost"
  color={onSettings ? 'primary' : 'secondary'}
  {shape}
  class={className}
  icon={onSettings ? mdiClose : mdiCogOutline}
  title={onSettings ? 'Close settings' : 'Settings & diagnostics'}
  aria-label={onSettings ? 'Close settings' : 'Settings'}
  aria-pressed={onSettings}
  onclick={toggleSettings}
/>
