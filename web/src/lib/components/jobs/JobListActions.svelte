<script lang="ts">
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import { Button } from '@immich/ui';
  import { mdiTrashCanOutline } from '@mdi/js';
</script>

<Button
  size="tiny"
  variant="ghost"
  color="secondary"
  fullWidth
  leadingIcon={mdiTrashCanOutline}
  onclick={jobs.clear}
>
  Clear finished
</Button>
