<script lang="ts">
  import { page } from '$app/state';
  import { runEditorRoute } from '#lib/editor/editorRoute.svelte.ts';
  import DesktopEditorShell from '#lib/components/editor/desktop/DesktopEditorShell.svelte';

  const id = $derived(page.params.id as string);

  runEditorRoute(() => id);
</script>

<DesktopEditorShell />
