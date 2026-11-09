<script lang="ts">
  import { page } from '$app/state';
  import { runEditorRoute } from '#lib/editor/editorRoute.svelte.ts';
  import DesktopEditorShell from '#lib/components/editor/desktop/DesktopEditorShell.svelte';
  import TouchEditorShell from '#lib/components/editor/touch/TouchEditorShell.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';

  const id = $derived(page.params.id as string);

  runEditorRoute(() => id);
</script>

{#if layout.mode === 'touch'}
  <TouchEditorShell />
{:else}
  <DesktopEditorShell />
{/if}
