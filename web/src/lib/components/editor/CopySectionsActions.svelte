<script lang="ts">
  import { copyDialog } from '#lib/stores/copyDialog.svelte.ts';
  import { Button } from '@immich/ui';
</script>

<Button size="tiny" variant="ghost" color="secondary" onclick={() => copyDialog.setAll(true)}>
  Check all
</Button>
<Button size="tiny" variant="ghost" color="secondary" onclick={() => copyDialog.setAll(false)}>
  Check none
</Button>
