<script lang="ts">
  import { openViewPanel } from '#lib/browse/viewPanel.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiViewGridOutline } from '@mdi/js';

  let { class: className = '' }: { class?: string } = $props();

  function toggle(): void {
    if (browseView.viewOpen) browseView.setViewOpen(false);
    else void openViewPanel();
  }
</script>

<IconButton
  id="browse-view-toggle"
  size="medium"
  variant="ghost"
  color={browseView.viewOpen ? 'primary' : 'secondary'}
  shape="round"
  class="size-11 {className}"
  icon={mdiViewGridOutline}
  title="View"
  aria-label="View"
  aria-expanded={browseView.viewOpen}
  onclick={toggle}
/>
