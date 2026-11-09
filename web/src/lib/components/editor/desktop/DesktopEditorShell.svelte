<script lang="ts">
  import RightSidebar from '#lib/components/editor/desktop/RightSidebar.svelte';
  import Filmstrip from '#lib/components/shell/Filmstrip.svelte';
  import PresetStrip from '#lib/components/preset/PresetStrip.svelte';
  import EditorToolRail from '#lib/components/editor/desktop/EditorToolRail.svelte';
  import Viewer from '#lib/components/editor/Viewer.svelte';
  import ImageToolbar from '#lib/components/editor/ImageToolbar.svelte';
  import BottomBar from '#lib/components/editor/BottomBar.svelte';
  import Notice from '#lib/components/Notice.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { IconButton } from '@immich/ui';
  import { mdiFullscreenExit } from '@mdi/js';
  import CommandPalette from './CommandPalette.svelte';
</script>

<div class="app-frame flex overflow-hidden">
  <div class="relative flex min-h-0 min-w-0 flex-1">
    <div class="flex min-h-0 min-w-0 flex-1 flex-col">
      <main class="flex-1 min-h-0 flex flex-col bg-light">
        {#if editor.error}
          <Notice message={editor.error} class="mx-4 my-2 text-xs" />
        {/if}
        {#if !ui.fullscreen}<ImageToolbar />{/if}
        <Viewer />
        {#if ui.fullscreen}
          <IconButton
            size="small"
            variant="filled"
            color="primary"
            shape="round"
            class="fixed top-[calc(0.75rem+var(--safe-top))] right-[calc(0.75rem+var(--safe-right))] z-40 shadow-lg"
            icon={mdiFullscreenExit}
            title={hint('Exit fullscreen', 'fullscreen')}
            aria-label={hint('Exit fullscreen', 'fullscreen')}
            onclick={ui.toggleFullscreen}
          />
        {:else}
          <BottomBar />
        {/if}
      </main>
      {#if editor.assetId && !ui.fullscreen}
        {#if ui.presetStrip}
          <PresetStrip />
        {:else}
          <Filmstrip resizable collapsed={ui.editorFilmstripCollapsed} />
        {/if}
      {/if}
    </div>
    {#if !ui.fullscreen}
      <RightSidebar />
      <EditorToolRail />
    {/if}
  </div>
</div>
<CommandPalette />
