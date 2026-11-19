<script lang="ts">
  import { curveEditor } from '#lib/stores/curveEditor.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiOpenInNew } from '@mdi/js';
</script>

<IconButton
  size="tiny"
  variant="ghost"
  color={curveEditor.poppedOut ? 'primary' : 'secondary'}
  class="size-6"
  icon={mdiOpenInNew}
  title="Open a larger curve editor"
  aria-label="Pop out curve editor"
  aria-pressed={curveEditor.poppedOut}
  onclick={() => (curveEditor.poppedOut = !curveEditor.poppedOut)}
/>
