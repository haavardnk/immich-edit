<script lang="ts">
  import { wideSidebar } from '#lib/components/shell/wide/wideSidebar.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiDockLeft } from '@mdi/js';
</script>

<IconButton
  size="small"
  variant="ghost"
  color={wideSidebar.open ? 'primary' : 'secondary'}
  class="size-11 shrink-0"
  icon={mdiDockLeft}
  title={wideSidebar.open ? 'Hide sidebar' : 'Show sidebar'}
  aria-label={wideSidebar.open ? 'Hide sidebar' : 'Show sidebar'}
  aria-expanded={wideSidebar.open}
  onclick={wideSidebar.toggle}
/>
