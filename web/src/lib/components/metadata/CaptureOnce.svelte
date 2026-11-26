<script lang="ts">
  import CheckboxRow from '#lib/components/CheckboxRow.svelte';
  import WallTimeField from '#lib/components/metadata/WallTimeField.svelte';
  import { captureEditor } from '#lib/stores/captureEditor.svelte.ts';

  const editor = captureEditor;
</script>

<div class="flex items-center gap-3 touch:flex-col touch:items-stretch touch:gap-2">
  <span class="min-w-0 flex-1 text-sm text-light-950">Date and time</span>
  <WallTimeField
    label="Date and time"
    value={editor.onceWall}
    onChange={(wall) => (editor.onceWall = wall)}
  />
</div>
<CheckboxRow
  checked={editor.spaced}
  label="1 s apart, by file name"
  onChange={(checked) => (editor.spaced = checked)}
/>
