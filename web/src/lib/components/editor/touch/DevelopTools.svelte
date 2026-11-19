<script lang="ts">
  import { commandLabel, commandTitle, isEnabled, runCommand } from '#lib/commands/command.ts';
  import { autoAdjust, resetDevelop } from '#lib/commands/editor/edits.ts';
  import { IconButton } from '@immich/ui';
  import { mdiMagnify } from '@mdi/js';
  import { openTouchSearch } from './modes';

  let { size = 'medium' }: { size?: 'small' | 'medium' } = $props();

  function search(event: MouseEvent): void {
    event.stopPropagation();
    openTouchSearch();
  }
</script>

<IconButton
  {size}
  variant="ghost"
  color="secondary"
  shape="round"
  icon={autoAdjust.icon}
  title={commandTitle(autoAdjust)}
  aria-label="Auto"
  disabled={!isEnabled(autoAdjust)}
  onclick={() => runCommand(autoAdjust)}
/>
<IconButton
  {size}
  variant="ghost"
  color="secondary"
  shape="round"
  icon={resetDevelop.icon}
  title={commandTitle(resetDevelop)}
  aria-label={commandLabel(resetDevelop)}
  disabled={!isEnabled(resetDevelop)}
  onclick={() => runCommand(resetDevelop)}
/>
<IconButton
  {size}
  variant="ghost"
  color="secondary"
  shape="round"
  icon={mdiMagnify}
  aria-label="Search settings"
  onclick={search}
/>
