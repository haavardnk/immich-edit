<script lang="ts">
  import { page } from '$app/state';
  import { runEditorRoute } from '#lib/editor/editorRoute.svelte.ts';
  import DesktopEditorShell from '#lib/components/editor/desktop/DesktopEditorShell.svelte';
  import TouchEditorShell from '#lib/components/editor/touch/TouchEditorShell.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { provideInfoSource } from '#lib/browse/info/infoSource.ts';
  import { EDITOR_INFO } from '#lib/editor/editorInfo.ts';

  const id = $derived(page.params.id as string);

  runEditorRoute(() => id);
  provideInfoSource(EDITOR_INFO);
</script>

{#if layout.mode === 'touch'}
  <TouchEditorShell />
{:else}
  <DesktopEditorShell />
{/if}
