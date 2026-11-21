<script lang="ts">
  import { albumActions } from '#lib/stores/albumActions.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiPlus } from '@mdi/js';
</script>

<IconButton
  size="small"
  variant="ghost"
  color="secondary"
  class="touch:size-11"
  icon={mdiPlus}
  title="Create album"
  aria-label="Create album"
  onclick={() => void albumActions.create()}
/>
