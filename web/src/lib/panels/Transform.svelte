<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiRotateLeft, mdiRotateRight, mdiFlipHorizontal, mdiFlipVertical } from '@mdi/js';

  function rotateLeft(): void {
    editor.edits.rotate = ((editor.edits.rotate + 270) % 360) as 0 | 90 | 180 | 270;
    void editor.onCommit();
  }
  function rotateRight(): void {
    editor.edits.rotate = ((editor.edits.rotate + 90) % 360) as 0 | 90 | 180 | 270;
    void editor.onCommit();
  }
  function toggleFlipH(): void {
    editor.edits.flip_h = !editor.edits.flip_h;
    void editor.onCommit();
  }
  function toggleFlipV(): void {
    editor.edits.flip_v = !editor.edits.flip_v;
    void editor.onCommit();
  }
</script>

<div class="flex flex-col gap-2">
  <div class="grid grid-cols-2 gap-1.5">
    <button
      class="flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors"
      onclick={rotateLeft}
    >
      <Icon path={mdiRotateLeft} size={16} />
      90°
    </button>
    <button
      class="flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors"
      onclick={rotateRight}
    >
      <Icon path={mdiRotateRight} size={16} />
      90°
    </button>
    <button
      class="flex items-center justify-center gap-1.5 py-1.5 rounded-lg transition-colors text-xs {editor.edits.flip_h ? 'bg-immich-dark-primary/20 text-immich-dark-primary' : 'bg-white/5 hover:bg-white/10'}"
      onclick={toggleFlipH}
    >
      <Icon path={mdiFlipHorizontal} size={16} />
      Flip H
    </button>
    <button
      class="flex items-center justify-center gap-1.5 py-1.5 rounded-lg transition-colors text-xs {editor.edits.flip_v ? 'bg-immich-dark-primary/20 text-immich-dark-primary' : 'bg-white/5 hover:bg-white/10'}"
      onclick={toggleFlipV}
    >
      <Icon path={mdiFlipVertical} size={16} />
      Flip V
    </button>
  </div>
  <div class="text-[10px] text-immich-dark-fg/30 font-mono text-center">rotate {editor.edits.rotate}°</div>
</div>
