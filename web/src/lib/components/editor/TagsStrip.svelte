<script lang="ts">
  import TagPicker from '$lib/components/TagPicker.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';

  const tags = $derived(editor.asset?.tags ?? []);
</script>

<TagPicker
  {tags}
  open={ui.metaPopover === 'tags'}
  onToggle={() => ui.togglePopover('tags')}
  onClose={ui.closePopover}
  onAdd={editor.addTag}
  onRemove={editor.removeTag}
  onCreate={editor.createAndAddTag}
/>
