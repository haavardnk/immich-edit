<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { geometryIsIdentity } from '$lib/types/edits';
  import { hint } from '$lib/shortcuts/labels';
  import { Button, IconButton } from '@immich/ui';
  import {
    mdiFlipHorizontal,
    mdiFlipVertical,
    mdiRestore,
    mdiRotateLeft,
    mdiRotateRight
  } from '@mdi/js';

  const flipH = $derived(editor.geometrySession?.draftFlipH ?? editor.edits.geometry.flip_h);
  const flipV = $derived(editor.geometrySession?.draftFlipV ?? editor.edits.geometry.flip_v);
  const iconClass = 'bg-transparent hover:bg-white/6';
</script>

<div class="flex min-w-0 flex-1 items-center gap-0.5">
  <Button
    size="tiny"
    variant="ghost"
    color="secondary"
    class="h-8 min-w-0 flex-1 justify-center bg-transparent hover:bg-white/6"
    leadingIcon={mdiRestore}
    title={hint('Reset geometry', 'resetEdits')}
    aria-label="Reset geometry"
    disabled={geometryIsIdentity(editor.edits.geometry) || editor.saving}
    onclick={() => void editor.resetGeometry()}
  >
    Reset
  </Button>
</div>
<div class="h-5 w-px shrink-0 bg-hairline"></div>
<div class="flex shrink-0 items-center gap-0.5">
  <IconButton
    size="small"
    variant="ghost"
    color="secondary"
    class={iconClass}
    icon={mdiRotateLeft}
    title="Rotate left 90°"
    aria-label="Rotate left 90°"
    onclick={() => editor.rotateStep(270)}
  />
  <IconButton
    size="small"
    variant="ghost"
    color="secondary"
    class={iconClass}
    icon={mdiRotateRight}
    title="Rotate right 90°"
    aria-label="Rotate right 90°"
    onclick={() => editor.rotateStep(90)}
  />
  <IconButton
    size="small"
    variant="ghost"
    color={flipH ? 'primary' : 'secondary'}
    class={iconClass}
    icon={mdiFlipHorizontal}
    title="Flip horizontal"
    aria-label="Flip horizontal"
    aria-pressed={flipH}
    onclick={() => editor.flipStep('h')}
  />
  <IconButton
    size="small"
    variant="ghost"
    color={flipV ? 'primary' : 'secondary'}
    class={iconClass}
    icon={mdiFlipVertical}
    title="Flip vertical"
    aria-label="Flip vertical"
    aria-pressed={flipV}
    onclick={() => editor.flipStep('v')}
  />
</div>
