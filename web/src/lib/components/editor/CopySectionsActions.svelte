<script lang="ts">
  import { copyDialog } from '#lib/stores/copyDialog.svelte.ts';
  import { Button } from '@immich/ui';
</script>

<Button
  size="tiny"
  variant="ghost"
  color="secondary"
  class="touch:h-9 touch:text-[15px] touch:text-primary"
  aria-label="Check all"
  onclick={() => copyDialog.setAll(true)}
>
  <span class="touch:hidden">Check all</span>
  <span class="hidden touch:inline">All</span>
</Button>
<Button
  size="tiny"
  variant="ghost"
  color="secondary"
  class="touch:h-9 touch:text-[15px] touch:text-primary"
  aria-label="Check none"
  onclick={() => copyDialog.setAll(false)}
>
  <span class="touch:hidden">Check none</span>
  <span class="hidden touch:inline">None</span>
</Button>
