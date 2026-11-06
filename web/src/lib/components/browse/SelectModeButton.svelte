<script lang="ts">
  import { selection } from '#lib/stores/selection.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiCheckboxMarkedOutline } from '@mdi/js';
</script>

<IconButton
  size="medium"
  variant="ghost"
  color="secondary"
  shape="round"
  class="size-11"
  icon={mdiCheckboxMarkedOutline}
  title="Select photos"
  aria-label="Select photos"
  onclick={selection.enterMode}
/>
