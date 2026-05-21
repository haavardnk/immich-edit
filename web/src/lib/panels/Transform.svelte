<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';

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
    <button class="btn btn-sm btn-soft" onclick={rotateLeft}>↺ 90°</button>
    <button class="btn btn-sm btn-soft" onclick={rotateRight}>↻ 90°</button>
    <button
      class="btn btn-sm btn-soft"
      class:btn-active={editor.edits.flip_h}
      onclick={toggleFlipH}
    >
      Flip H
    </button>
    <button
      class="btn btn-sm btn-soft"
      class:btn-active={editor.edits.flip_v}
      onclick={toggleFlipV}
    >
      Flip V
    </button>
  </div>
  <div class="text-[11px] opacity-50 font-mono">rotate {editor.edits.rotate}°</div>
</div>
