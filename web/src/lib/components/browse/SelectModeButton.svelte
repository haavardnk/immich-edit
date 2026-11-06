<script lang="ts">
  import { selection } from '#lib/stores/selection.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiCheckboxMarkedOutline } from '@mdi/js';

  let { size = 'medium', class: className = '' }: { size?: 'small' | 'medium'; class?: string } =
    $props();
</script>

<IconButton
  {size}
  variant="ghost"
  color="secondary"
  shape="round"
  class={className}
  icon={mdiCheckboxMarkedOutline}
  title="Select photos"
  aria-label="Select photos"
  onclick={selection.enterMode}
/>
